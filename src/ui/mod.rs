mod edit_planet;
mod help;
mod main_menu;
mod new_planet;
mod orbit;
mod panels;
mod star_system;
mod stat;

use bevy::{app::AppExit, math::Rect, prelude::*};
use bevy_egui::{
    egui::{self, FontData, FontDefinitions, FontFamily},
    EguiContexts, EguiPlugin, EguiSettings,
};
use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::{
    assets::{UiAssets, UiTexture, UiTextures},
    conf::Conf,
    draw::UpdateMap,
    gz::GunzipBin,
    msg::MsgHolder,
    overlay::OverlayLayerKind,
    planet::*,
    screen::{CursorMode, OccupiedScreenSpace},
    sim::ManagePlanet,
    GameState,
};

use self::help::HelpItem;

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin;

#[derive(Clone, Default, Debug, Resource)]
pub struct WindowsOpenState {
    pub build: bool,
    pub orbit: bool,
    pub star_system: bool,
    pub layers: bool,
    pub stat: bool,
    pub message: bool,
    pub game_menu: bool,
    pub help: bool,
    pub edit_planet: bool,
}

#[derive(Clone, Default, Resource)]
pub struct EguiTextures(HashMap<UiTexture, (egui::TextureHandle, egui::Vec2)>);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct UiWindowsSystemSet;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .init_resource::<WindowsOpenState>()
            .init_resource::<OverlayLayerKind>()
            .add_systems((setup_fonts, load_textures).in_schedule(OnExit(GameState::AssetLoading)))
            .add_system(main_menu::set_main_menu_state.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(main_menu::main_menu.in_set(OnUpdate(GameState::MainMenu)))
            .add_system(
                panels::panels
                    .in_set(OnUpdate(GameState::Running))
                    .before(UiWindowsSystemSet),
            )
            .add_systems(
                (
                    build_window,
                    orbit::orbit_window,
                    star_system::star_system_window,
                    layers_window,
                    stat::stat_window,
                    msg_window,
                    game_menu_window,
                    help::help_window,
                    edit_planet::edit_planet_window,
                )
                    .in_set(OnUpdate(GameState::Running))
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
    egui_settings.scale_factor = conf.scale_factor.into();

    let font_data = gunzip_bin.get(&ui_assets.font).unwrap().clone();
    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_owned(font_data.0);
    font_data.tweak.scale = conf.font_scale;
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
    ui_textures: Res<UiTextures>,
) {
    let ctx = egui_ctxs.ctx_mut();

    let mut egui_textures = HashMap::new();

    for (k, handle) in ui_textures.textures.iter() {
        let image = images.get(handle).unwrap();
        let size = egui::Vec2::new(image.size().x, image.size().y);
        let color_image = egui::ColorImage {
            size: [size.x as usize, size.y as usize],
            pixels: image
                .data
                .windows(4)
                .step_by(4)
                .map(|rgba| {
                    egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
                })
                .collect(),
        };
        let texture_handle =
            ctx.load_texture(k.as_ref(), color_image, egui::TextureOptions::NEAREST);

        egui_textures.insert(*k, (texture_handle, size));
    }

    commands.insert_resource(EguiTextures(egui_textures));
}

fn build_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut cursor_mode: ResMut<CursorMode>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    params: Res<Params>,
) {
    if !wos.build {
        return;
    }

    let rect = egui::Window::new(t!("build"))
        .open(&mut wos.build)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            if ui.button(t!("demolition")).clicked() {
                *cursor_mode = CursorMode::Demolition;
            }
            ui.separator();
            for kind in &planet.player.buildable_structures {
                let s: &str = kind.as_ref();
                if ui
                    .button(t!(s))
                    .on_hover_ui(|ui| HelpItem::Structures(*kind).ui(ui, &params))
                    .clicked()
                {
                    *cursor_mode = CursorMode::Build(*kind);
                }
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn layers_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut current_layer: ResMut<OverlayLayerKind>,
    mut update_map: ResMut<UpdateMap>,
    conf: Res<Conf>,
) {
    if !wos.layers {
        return;
    }
    let mut new_layer = *current_layer;

    let rect = egui::Window::new(t!("layers"))
        .open(&mut wos.layers)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            for kind in OverlayLayerKind::iter() {
                ui.radio_value(&mut new_layer, kind, t!(kind.as_ref()));
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));

    if new_layer != *current_layer {
        *current_layer = new_layer;
        update_map.update();
    }
}

fn msg_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    msg_holder: Res<MsgHolder>,
    conf: Res<Conf>,
) {
    if !wos.message {
        return;
    }

    let (_msg_kind, s) = msg_holder.latest();

    let mut open = true;
    let rect = egui::Window::new(t!("messages"))
        .open(&mut wos.message)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.label(s);
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button(t!("close")).clicked() {
                    open = false;
                }
            });
        })
        .unwrap()
        .response
        .rect;
    wos.message = open && wos.message;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn game_menu_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut app_exit_events: EventWriter<AppExit>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    conf: Res<Conf>,
) {
    if !wos.game_menu {
        return;
    }

    let mut close = false;

    let rect = egui::Window::new(t!("menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .open(&mut wos.game_menu)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button(t!("save")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Save("test.planet".into()));
                    close = true;
                }

                if ui.button(t!("load")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Load("test.planet".into()));
                    close = true;
                }
                ui.separator();
                if ui.button(t!("exit")).clicked() {
                    app_exit_events.send(bevy::app::AppExit);
                }
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));

    if close {
        wos.game_menu = false;
    }
}

fn convert_rect(rect: bevy_egui::egui::Rect, scale_factor: f32) -> Rect {
    Rect {
        min: Vec2::new(rect.left() * scale_factor, rect.top() * scale_factor),
        max: Vec2::new(rect.right() * scale_factor, rect.bottom() * scale_factor),
    }
}

impl Conf {
    fn tex_size(&self, size: egui::Vec2) -> egui::Vec2 {
        let factor = 1.0;
        egui::Vec2::new(size.x * factor, size.y * factor)
    }
}
