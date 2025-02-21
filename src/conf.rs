use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

use crate::GameState;
use crate::{assets::UiAssets, text_assets::Lang};

pub const CONF_FILE_NAME: &str = "conf.toml";

#[derive(Clone, Copy, Debug)]
pub struct ConfPlugin;

impl Plugin for ConfPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ConfChange>()
            .add_plugins(bevy_common_assets::toml::TomlAssetPlugin::<Conf>::new(&[
                "conf.toml",
            ]))
            .add_systems(Update, on_change)
            .add_systems(OnExit(GameState::AssetLoading), load_conf);
    }
}

fn on_change(mut er_conf_change: EventReader<ConfChange>, conf: Option<Res<Conf>>) {
    if let Some(conf) = conf {
        let conf = toml::to_string(&*conf).unwrap();
        if er_conf_change.read().next().is_some() {
            if let Err(e) = crate::platform::write_data_file(CONF_FILE_NAME, &conf) {
                log::error!("cannot save conf: {}", e);
            }
            log::info!("conf saved");
        }
    }
}

fn load_conf(mut command: Commands, ui_assets: Res<UiAssets>, conf: Res<Assets<Conf>>) {
    let conf = match crate::platform::read_data_file(CONF_FILE_NAME)
        .and_then(|data| toml::from_str(&data).context("deserialize conf"))
    {
        Ok(conf) => conf,
        Err(e) => {
            log::info!("cannot load config: {}", e);
            conf.get(&ui_assets.default_conf).unwrap().clone()
        }
    };
    let conf = crate::platform::modify_conf(conf);
    crate::text_assets::set_lang(conf.lang);
    command.insert_resource(conf);
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Asset, Resource, TypePath)]
pub struct Conf {
    pub lang: Lang,
    pub camera_move_speed: f32,
    pub ui: UiConf,
    pub autosave_enabled: bool,
    pub autosave_cycle_duration: u64,
    pub autosave_max_files: usize,
    pub manual_max_files: usize,
    pub screen_refresh_rate: HighLow3,
    pub show_fps: bool,
    pub window: Option<WindowConf>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Reflect)]
pub struct UiConf {
    pub scale_factor: f32,
    pub font_scale: f32,
    pub messages_in_list: usize,
    pub min_sidebar_width: f32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Asset, Resource, TypePath)]
pub struct WindowConf {
    pub size: (u32, u32),
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, AsRefStr, EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum HighLow3 {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, Event)]
pub struct ConfChange;

impl Default for ConfChange {
    fn default() -> Self {
        ConfChange
    }
}
