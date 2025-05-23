#![allow(clippy::type_complexity, clippy::comparison_chain)]

extern crate tile_geom as geom;

#[macro_use]
mod tools;
#[macro_use]
mod text_assets;

mod achivement_save;
mod action;
mod assets;
mod audio;
mod conf;
mod draw;
mod gz;
mod image_assets;
mod manage_planet;
mod overlay;
mod planet;
mod platform;
mod saveload;
mod screen;
mod text;
mod title_screen;
mod tutorial;
mod ui;

use std::path::PathBuf;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};
use clap::Parser;

const APP_NAME: &str = concat!("Gaia Maker v", env!("CARGO_PKG_VERSION"));

#[derive(Clone, Parser, Debug)]
#[clap(author, about, version)]
struct Args {
    /// Specify the number of threads used to run this game
    #[arg(long, default_value_t = 4)]
    num_threads: u8,
    /// Log file path
    #[arg(long)]
    log_file: Option<PathBuf>,
    #[arg(long)]
    launcher_port: Option<u16>,
}

fn main() {
    let args = Args::parse();
    crate::platform::init_rayon(args.num_threads as usize);
    if let Some(log_file) = args.log_file {
        crate::platform::init_log_file(log_file);
    }
    if let Some(port) = args.launcher_port {
        crate::platform::client::run_client(port);
    }
    crate::platform::window_open();

    let mut window = Window {
        title: APP_NAME.into(),
        present_mode: PresentMode::Fifo,
        canvas: Some("#game-screen".into()),
        ..default()
    };
    match crate::platform::PreferredWindowResolution::get() {
        platform::PreferredWindowResolution::Size(w, h) => {
            window.resolution = WindowResolution::new(w as f32, h as f32);
        }
        platform::PreferredWindowResolution::Maximized => {
            window.set_maximized(true);
        }
    }

    App::new()
        .add_plugins(AssetPlugin)
        .add_plugins(
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    custom_layer: crate::platform::log_plugin_custom_layer,
                    ..default()
                })
                .set(TaskPoolPlugin {
                    task_pool_options: TaskPoolOptions::with_num_threads(args.num_threads as usize),
                })
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                }),
        )
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
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
        .add_plugins(manage_planet::ManagePlanetPlugin)
        .add_plugins(achivement_save::AchivementPlugin)
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
    Slow,
    Medium,
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
        use std::path::PathBuf;
        #[cfg(feature = "deb")]
        fn asset_file_path() -> PathBuf {
            let current_exe = std::env::current_exe().expect("cannot get current exe path");
            let usr_dir = current_exe
                .parent()
                .expect("cannot get usr directory path")
                .parent()
                .expect("cannot get usr directory path");
            usr_dir.join("share/games/gaia-maker/assets.tar.gz")
        }
        #[cfg(all(not(feature = "deb"), not(target_arch = "wasm32")))]
        fn asset_file_path() -> PathBuf {
            let current_exe = std::env::current_exe().expect("cannot get current exe path");
            let dir = current_exe.parent().expect("invalid current exe path");
            dir.join("assets.tar.gz")
        }
        #[cfg(all(not(feature = "deb"), target_arch = "wasm32"))]
        fn asset_file_path() -> PathBuf {
            "assets.tar".into()
        }

        app.add_plugins(bevy_asset_tar::AssetTarPlugin {
            archive_files: vec![asset_file_path()],
            addon_directories: platform::addon_directory(),
            ..Default::default()
        });
    }

    #[cfg(not(feature = "asset_tar"))]
    fn build(&self, _app: &mut App) {}
}
