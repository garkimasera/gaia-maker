#![allow(clippy::type_complexity)]

extern crate tile_geom as geom;

#[macro_use]
mod tools;
#[macro_use]
mod text_assets;

mod action;
mod assets;
mod audio;
mod conf;
mod draw;
mod gz;
mod image_assets;
mod info;
mod overlay;
mod planet;
mod platform;
mod saveload;
mod screen;
mod sim;
mod text;
mod title_screen;
mod ui;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};
use clap::Parser;

const APP_NAME: &str = concat!("Gaia Maker v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Parser, Debug)]
#[clap(author, version)]
struct Args {}

fn main() {
    let _args = Args::parse();

    crate::platform::window_open();
    let window_size = crate::platform::preferred_window_size();

    App::new()
        .add_plugins(AssetPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: APP_NAME.into(),
                present_mode: PresentMode::Fifo,
                resolution: WindowResolution::new(window_size.0 as f32, window_size.1 as f32),
                canvas: Some("#game-screen".into()),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(gz::GzPlugin)
        .add_plugins(text_assets::TextAssetsPlugin)
        .add_plugins(image_assets::ImageAssetsPlugin)
        .init_state::<GameState>()
        .add_plugins(conf::ConfPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(overlay::OverlayPlugin)
        .add_plugins(screen::ScreenPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(audio::GameAudioPlugin)
        .add_plugins(title_screen::TitleScreenPlugin)
        .add_plugins(draw::DrawPlugin)
        .add_plugins(action::ActionPlugin)
        .add_plugins(sim::SimPlugin)
        .insert_resource(WinitSettings::game())
        .init_resource::<GameSpeed>()
        .run();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    MainMenu,
    Running,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, Resource)]
pub enum GameSpeed {
    #[default]
    Paused,
    Normal,
    Fast,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub enum GameSystemSet {
    Draw,
    StartSim,
    UpdateHoverTile,
}

struct AssetPlugin;

impl Plugin for AssetPlugin {
    #[cfg(feature = "asset_tar")]
    fn build(&self, app: &mut App) {
        let asset_file_path = option_env!("ASSET_FILE_PATH").unwrap_or("assets.tar.gz");
        app.add_plugins(bevy_asset_tar::AssetTarPlugin {
            archive_files: vec![std::path::PathBuf::from(asset_file_path)],
            addon_directories: platform::addon_directory(),
            ..Default::default()
        });
    }

    #[cfg(not(feature = "asset_tar"))]
    fn build(&self, _app: &mut App) {}
}
