#![warn(rust_2018_compatibility, future_incompatible, nonstandard_style)]
#![allow(clippy::type_complexity)]

extern crate tile_geom as geom;

use clap::Parser;

#[macro_use]
mod tools;
#[macro_use]
mod text;

mod action;
mod assets;
mod audio;
mod conf;
mod draw;
mod gz;
mod info;
mod overlay;
mod planet;
mod saveload;
mod screen;
mod sim;
mod ui;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};

const APP_NAME: &str = concat!("Gaia Maker v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Parser, Debug)]
#[clap(author, version)]
struct Args {}

fn main() {
    let _args = Args::parse();

    screen::window_open();
    let window_size = screen::preferred_window_size();

    App::new()
        .add_state::<GameState>()
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
        .add_plugins(text::TextPlugin)
        .add_plugins(conf::ConfPlugin)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(overlay::OverlayPlugin)
        .add_plugins(screen::ScreenPlugin)
        .add_plugins(ui::UiPlugin)
        .add_plugins(audio::GameAudioPlugin)
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
