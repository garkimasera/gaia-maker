use anyhow::{anyhow, Result};
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

use crate::GameState;
use crate::{assets::UiAssets, text_assets::Lang};

const CONF_FILE_NAME: &str = "conf.ron";

#[cfg(not(target_arch = "wasm32"))]
use std::sync::LazyLock;

#[cfg(not(target_arch = "wasm32"))]
static DATA_DIR: LazyLock<Option<std::path::PathBuf>> =
    LazyLock::new(|| dirs::data_dir().map(|path| path.join(env!("CARGO_PKG_NAME"))));

#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir() -> Option<&'static std::path::Path> {
    DATA_DIR.as_ref().map(|path| path.as_ref())
}

#[cfg(not(target_arch = "wasm32"))]
fn conf_file() -> Option<std::path::PathBuf> {
    data_dir().map(|data_dir| data_dir.join(CONF_FILE_NAME))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "asset_tar"))]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    data_dir()
        .map(|data_dir| data_dir.join("addons"))
        .into_iter()
        .collect()
}

#[cfg(all(target_arch = "wasm32", feature = "asset_tar"))]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    Vec::new()
}

#[derive(Clone, Copy, Debug)]
pub struct ConfPlugin;

impl Plugin for ConfPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConfChange>()
            .add_plugins(RonAssetPlugin::<Conf>::new(&["conf.ron"]))
            .add_systems(Update, on_change)
            .add_systems(OnExit(GameState::AssetLoading), set_conf);
    }
}

fn on_change(mut er_conf_change: EventReader<ConfChange>, conf: Option<Res<Conf>>) {
    if let Some(conf) = &conf {
        if er_conf_change.read().next().is_some() {
            if let Err(e) = save(conf) {
                log::error!("cannot save conf: {}", e);
            }
        }
    }
}

fn set_conf(mut command: Commands, ui_assets: Res<UiAssets>, conf: Res<Assets<Conf>>) {
    let conf = match load() {
        Ok(conf) => conf,
        Err(e) => {
            log::info!("cannot load config: {}", e);
            conf.get(&ui_assets.default_conf).unwrap().clone()
        }
    };
    crate::text_assets::set_lang(conf.lang);
    command.insert_resource(conf);
}

#[derive(Clone, Debug, Serialize, Deserialize, Asset, Resource, TypePath)]
pub struct Conf {
    pub lang: Lang,
    pub camera_move_speed: f32,
    pub ui: UiConf,
    #[serde(with = "serde_with::rust::unwrap_or_skip")]
    pub autosave_cycle_duration: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Reflect)]
pub struct UiConf {
    pub scale_factor: f32,
    pub font_scale: f32,
    pub messages_in_list: usize,
    pub min_sidebar_width: f32,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ConfChange;

impl Default for ConfChange {
    fn default() -> Self {
        ConfChange
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load() -> Result<Conf> {
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let conf = ron::from_str(&std::fs::read_to_string(conf_file_path)?)?;
    Ok(conf)
}

#[cfg(not(target_arch = "wasm32"))]
fn save(conf: &Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    std::fs::write(conf_file_path, s)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn load() -> Result<Conf> {
    let s = get_storage()?
        .get_item(CONF_FILE_NAME)
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    let conf = ron::from_str(&s)?;
    Ok(conf)
}

#[cfg(target_arch = "wasm32")]
fn save(conf: &Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    get_storage()?
        .set_item(CONF_FILE_NAME, &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn get_storage() -> Result<web_sys::Storage> {
    let window = web_sys::window().ok_or_else(|| anyhow!("cannot get Window"))?;
    let storage = window
        .local_storage()
        .map_err(|e| anyhow!("cannot get local_storage {:?}", e))?
        .ok_or_else(|| anyhow!("cannot get local_storage"))?;
    Ok(storage)
}
