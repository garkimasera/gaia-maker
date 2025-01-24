use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use serde::{Deserialize, Serialize};

use crate::GameState;
use crate::{assets::UiAssets, text_assets::Lang};

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
            if let Err(e) = crate::platform::conf_save(conf) {
                log::error!("cannot save conf: {}", e);
            }
            log::info!("conf saved");
        }
    }
}

fn set_conf(mut command: Commands, ui_assets: Res<UiAssets>, conf: Res<Assets<Conf>>) {
    let conf = match crate::platform::conf_load() {
        Ok(conf) => conf,
        Err(e) => {
            log::info!("cannot load config: {}", e);
            conf.get(&ui_assets.default_conf).unwrap().clone()
        }
    };
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
    #[serde(default)]
    pub max_simulation_speed: bool,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Reflect)]
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
