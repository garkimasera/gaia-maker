mod animals;
mod debug_tools;
mod dialog;
mod help;
mod main_menu;
mod map;
mod new_planet;
mod panels;
mod space_buildings;
mod stat;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, load::SizedTexture, FontData, FontDefinitions, FontFamily},
    EguiContexts, EguiPlugin, EguiSettings,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::{
    assets::UiAssets,
    conf::Conf,
    draw::UpdateMap,
    gz::GunzipBin,
    overlay::OverlayLayerKind,
    screen::{CursorMode, OccupiedScreenSpace},
    GameState,
};

use self::dialog::{CivilizeDialog, MsgDialog};

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin;

#[derive(Clone, Default, Debug, Resource)]
pub struct WindowsOpenState {
    pub space_building: bool,
    pub animals: bool,
    pub map: bool,
    pub layers: bool,
    pub stat: bool,
    pub dialogs: Vec<Dialog>,
    pub help: bool,
    pub debug_tools: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Dialog {
    Msg(MsgDialog),
    Civilize(CivilizeDialog),
}

#[derive(Clone, Default, Resource)]
pub struct EguiTextures(HashMap<String, SizedTexture>);

impl EguiTextures {
    fn get(&self, path: impl AsRef<str>) -> SizedTexture {
        *self
            .0
            .get(path.as_ref())
            .unwrap_or_else(|| panic!("cannot get ui texture {}", path.as_ref()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct UiWindowsSystemSet;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<WindowsOpenState>()
            .init_resource::<OverlayLayerKind>()
            .init_resource::<map::NeedUpdate>()
            .add_systems(
                OnExit(GameState::AssetLoading),
                (setup_fonts, load_textures),
            )
            .add_systems(OnEnter(GameState::MainMenu), main_menu::set_main_menu_state)
            .add_systems(OnEnter(GameState::Running), map::update)
            .add_systems(
                Update,
                main_menu::main_menu.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                Update,
                panels::panels
                    .run_if(in_state(GameState::Running))
                    .before(UiWindowsSystemSet),
            )
            .add_systems(
                Update,
                (
                    space_buildings::space_buildings_window,
                    animals::animals_window,
                    map::map_window,
                    stat::stat_window,
                    layers_window,
                    dialog::dialogs,
                    help::help_window,
                    debug_tools::debug_tools_window,
                )
                    .run_if(in_state(GameState::Running))
                    .in_set(UiWindowsSystemSet),
            );
    }
}

fn setup_fonts(
    mut egui_ctxs: EguiContexts,
    mut egui_settings: ResMut<EguiSettings>,
    conf: Res<Assets<Conf>>,
    ui_assets: Res<UiAssets>,
    gunzip_bin: Res<Assets<GunzipBin>>,
) {
    let conf = conf.get(&ui_assets.default_conf).unwrap().clone();
    egui_settings.scale_factor = conf.ui.scale_factor;

    let font_data = gunzip_bin.get(&ui_assets.font).unwrap().clone();
    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_owned(font_data.0);
    font_data.tweak.scale = conf.ui.font_scale;
    fonts.font_data.insert("m+_font".to_owned(), font_data);
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "m+_font".to_owned());
    fonts
        .families
        .get_mut(&FontFamily::Monospace)
        .unwrap()
        .push("m+_font".to_owned());
    egui_ctxs.ctx_mut().set_fonts(fonts);
}

fn load_textures(
    mut commands: Commands,
    mut egui_ctxs: EguiContexts,
    images: Res<Assets<Image>>,
    ui_assets: Res<UiAssets>,
    mut texture_handles: Local<Vec<egui::TextureHandle>>,
) {
    let ctx = egui_ctxs.ctx_mut();

    let ui = ui_assets
        .ui_imgs
        .iter()
        .map(|(path, handle)| (path.clone(), handle.clone()));
    let start_planets = ui_assets.start_planet.iter().filter_map(|(path, handle)| {
        if let Ok(handle) = handle.clone().try_typed::<Image>() {
            Some((path.clone(), handle))
        } else {
            None
        }
    });
    let animal_imgs = ui_assets.animal_imgs.iter().filter_map(|(path, handle)| {
        if let Ok(handle) = handle.clone().try_typed::<Image>() {
            Some((path.clone(), handle))
        } else {
            None
        }
    });
    let other_imgs = ui_assets
        .other_imgs
        .iter()
        .map(|(path, handle)| (path.clone(), handle.clone()));
    let textures = ui.chain(start_planets).chain(animal_imgs).chain(other_imgs);

    let mut egui_textures = HashMap::new();
    for (path, handle) in textures {
        let image = images.get(&handle).unwrap();
        let texture_handle = bevy_image_to_egui_texture(ctx, image, &path);

        let Some(path) = path.strip_suffix(".png") else {
            continue;
        };

        egui_textures.insert(
            path.to_owned(),
            SizedTexture {
                id: texture_handle.id(),
                size: egui::Vec2::new(image.size().x as f32, image.size().y as f32),
            },
        );
        texture_handles.push(texture_handle);
    }

    commands.insert_resource(EguiTextures(egui_textures));
}

fn bevy_image_to_egui_texture(
    ctx: &egui::Context,
    image: &bevy::prelude::Image,
    name: &str,
) -> egui::TextureHandle {
    let image = image
        .clone()
        .try_into_dynamic()
        .unwrap_or_else(|_| panic!("not supported image format: {}", name))
        .into_rgba8();
    let w = image.width();
    let h = image.height();

    let (w, h) = if name.starts_with("animals/") {
        (w / 2, h / 2)
    } else {
        (w, h)
    };

    let mut pixels = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let pixel = image.get_pixel(x, y).0;
            pixels.push(egui::Color32::from_rgba_unmultiplied(
                pixel[0], pixel[1], pixel[2], pixel[3],
            ));
        }
    }

    let color_image = egui::ColorImage {
        size: [w as usize, h as usize],
        pixels,
    };
    ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST)
}

fn layers_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut current_layer: ResMut<OverlayLayerKind>,
    mut update_map: ResMut<UpdateMap>,
) {
    if !wos.layers {
        return;
    }

    let rect = egui::Window::new(t!("layers"))
        .open(&mut wos.layers)
        .vscroll(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            layers_menu(ui, &mut current_layer, &mut update_map);
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn layers_menu(
    ui: &mut egui::Ui,
    current_layer: &mut OverlayLayerKind,
    update_map: &mut UpdateMap,
) {
    let mut new_layer = *current_layer;
    for kind in OverlayLayerKind::iter() {
        if ui
            .radio_value(&mut new_layer, kind, t!(kind.as_ref()))
            .clicked()
        {
            ui.close_menu();
        }
    }
    if new_layer != *current_layer {
        *current_layer = new_layer;
        update_map.update();
    }
}
