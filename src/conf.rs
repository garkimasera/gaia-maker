use crate::GameState;
use crate::{assets::UiAssets, text::Lang};
use anyhow::Result;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

#[cfg(not(target_arch = "wasm32"))]
static DATA_DIR: Lazy<Option<PathBuf>> =
    Lazy::new(|| dirs::data_dir().map(|path| path.join(env!("CARGO_PKG_NAME"))));

pub fn data_dir() -> Option<&'static Path> {
    DATA_DIR.as_ref().map(|path| path.as_ref())
}

fn conf_file() -> Option<PathBuf> {
    data_dir().map(|data_dir| data_dir.join("conf.ron"))
}

#[derive(Clone, Copy, Debug)]
pub struct ConfPlugin;

impl Plugin for ConfPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<Conf>::new(&["conf.ron"]))
            .add_system_set(SystemSet::on_exit(GameState::AssetLoading).with_system(set_conf));
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
    crate::text::set_lang(conf.lang);
    command.insert_resource(conf);
}

#[derive(Clone, Debug, Serialize, Deserialize, Resource, TypeUuid)]
#[uuid = "92795344-1b26-49fb-b352-e989043777c7"]
pub struct Conf {
    pub lang: Lang,
    pub scale_factor: f32,
    pub font_scale: f32,
    pub max_message: usize,
    pub camera_move_speed: f32,
}

#[cfg(not(target_arch = "wasm32"))]
fn load() -> Result<Conf> {
    let conf_file_path =
        conf_file().ok_or_else(|| anyhow::anyhow!("cannot get data directory path"))?;
    let conf = ron::from_str(&std::fs::read_to_string(conf_file_path)?)?;
    Ok(conf)
}

#[cfg(not(target_arch = "wasm32"))]
fn save(conf: &Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    let conf_file_path =
        conf_file().ok_or_else(|| anyhow::anyhow!("cannot get data directory path"))?;
    std::fs::write(conf_file_path, s)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn load() -> Result<Conf> {
    None
}
