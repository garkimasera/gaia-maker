#![warn(rust_2018_compatibility, future_incompatible, nonstandard_style)]
#![allow(clippy::type_complexity)]

extern crate tile_geom as geom;

use clap::Parser;

#[macro_use]
mod tools;
#[macro_use]
mod text;
#[macro_use]
mod msg;

mod action;
mod assets;
mod audio;
mod draw;
mod gz;
mod info;
mod overlay;
mod planet;
mod saveload;
mod screen;
mod sim;
mod ui;

use bevy::{prelude::*, window::PresentMode, winit::WinitSettings};

const APP_NAME: &str = concat!("Pixel Gaia v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Parser, Debug)]
#[clap(author, version)]
struct Args {
    /// Open map editing tools
    #[clap(long)]
    edit_map: bool,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: APP_NAME.into(),
                present_mode: PresentMode::Fifo,
                ..default()
            },
            ..default()
        }))
        .add_plugin(gz::GzPlugin)
        .add_plugin(text::TextPlugin)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(overlay::OverlayPlugin)
        .add_plugin(screen::ScreenPlugin)
        .add_plugin(ui::UiPlugin {
            edit_map: args.edit_map,
        })
        .add_plugin(audio::GameAudioPlugin)
        .add_plugin(InspectorPlugin)
        .add_plugin(draw::DrawPlugin)
        .add_plugin(action::ActionPlugin)
        .add_plugin(sim::SimPlugin)
        .insert_resource(WinitSettings::game())
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    MainMenu,
    Running,
}

#[derive(Clone, Copy, Debug)]
pub struct InspectorPlugin;

#[cfg(feature = "inspector")]
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
    }
}

#[cfg(not(feature = "inspector"))]
impl Plugin for InspectorPlugin {
    fn build(&self, _app: &mut App) {}
}
