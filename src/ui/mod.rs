mod achivements;
mod animals;
mod control;
mod debug_tools;
mod dialogs;
mod help;
mod hover_tile_tooltip;
mod indicators;
mod main_menu;
mod map;
mod misc;
mod new_planet;
mod preferences;
mod reports;
mod saveload;
mod space_buildings;
mod stat;
mod toolbar;
mod tools_expander;
mod tutorial;

use bevy::prelude::*;
use bevy_egui::{
    EguiContextSettings, EguiContexts, EguiPlugin,
    egui::{self, FontData, FontDefinitions, FontFamily, load::SizedTexture},
};
use geom::Coords;
use std::collections::HashMap;

use crate::{
    GameState,
    assets::UiAssets,
    conf::Conf,
    gz::GunzipBin,
    manage_planet::{ManagePlanetError, SwitchPlanet},
    overlay::OverlayLayerKind,
    planet::{AnimalId, Params},
    screen::{CursorMode, OccupiedScreenSpace},
};

const HELP_TOOLTIP_WIDTH: f32 = 256.0;

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin;

#[derive(Clone, Debug, Resource)]
pub struct WindowsOpenState {
    pub space_building: bool,
    pub animals: bool,
    pub control: bool,
    pub map: bool,
    pub stat: bool,
    pub reports: bool,
    pub help: bool,
    pub save: bool,
    pub load: bool,
    pub achivements: bool,
    pub preferences: bool,
    pub debug_tools: bool,
    pub dialogs: Vec<Dialog>,
    pub error_popup: Option<ManagePlanetError>,
}

impl Default for WindowsOpenState {
    fn default() -> Self {
        Self {
            space_building: false,
            animals: false,
            control: false,
            map: true,
            stat: false,
            reports: true,
            help: false,
            save: false,
            load: false,
            achivements: false,
            preferences: false,
            debug_tools: false,
            dialogs: Vec::new(),
            error_popup: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Dialog {
    Civilize { p: Coords, id: AnimalId },
}

#[derive(Clone, Default, Resource)]
pub struct UiTextures(HashMap<String, SizedTexture>);

impl UiTextures {
    fn get(&self, path: impl AsRef<str>) -> SizedTexture {
        *self
            .0
            .get(path.as_ref())
            .unwrap_or_else(|| panic!("cannot get ui texture {}", path.as_ref()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct UiWindowsSystemSet;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct MainPanelsSystemSet;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .init_resource::<WindowsOpenState>()
            .init_resource::<OverlayLayerKind>()
            .init_resource::<map::NeedUpdate>()
            .add_systems(
                OnExit(GameState::AssetLoading),
                (setup_fonts, load_textures, setup_style).after(crate::assets::AssetsListSystemSet),
            )
            .add_systems(OnEnter(GameState::MainMenu), main_menu::set_main_menu_state)
            .add_systems(OnEnter(GameState::Running), map::update)
            .add_systems(
                Update,
                main_menu::main_menu.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), reset_ime)
            .add_systems(
                Update,
                toolbar::toolbar
                    .run_if(in_state(GameState::Running))
                    .in_set(MainPanelsSystemSet)
                    .before(UiWindowsSystemSet),
            )
            .add_systems(
                Update,
                (
                    indicators::indicators,
                    tools_expander::tools_expander,
                    space_buildings::space_buildings_window,
                    animals::animals_window,
                    control::control_window,
                    map::map_window,
                    stat::stat_window,
                    reports::reports_window,
                    help::help_window,
                    saveload::load_window,
                    tutorial::tutorial_popup,
                    achivements::achivements_window,
                    dialogs::error_popup,
                    dialogs::dialogs,
                    preferences::preferences_window,
                    debug_tools::debug_tools_window,
                    reset_window_open_state,
                )
                    .run_if(in_state(GameState::Running))
                    .in_set(UiWindowsSystemSet),
            )
            .add_systems(
                Update,
                hover_tile_tooltip::hover_tile_tooltip
                    .run_if(in_state(GameState::Running))
                    .after(UiWindowsSystemSet),
            )
            .add_systems(
                Update,
                check_windows_open_state.run_if(in_state(GameState::Running)),
            );
    }
}

fn reset_ime(mut windows: Query<&mut Window, With<bevy::window::PrimaryWindow>>) {
    windows.single_mut().ime_enabled = false;
}

fn setup_fonts(
    mut egui_ctxs: EguiContexts,
    mut egui_settings: Query<&mut EguiContextSettings, With<bevy::window::PrimaryWindow>>,
    conf: Res<Assets<Conf>>,
    ui_assets: Res<UiAssets>,
    gunzip_bin: Res<Assets<GunzipBin>>,
) {
    let conf = conf.get(&ui_assets.default_conf).unwrap().clone();
    egui_settings.single_mut().scale_factor = conf.ui.scale_factor;

    let font_data = gunzip_bin.get(&ui_assets.font).unwrap().clone();
    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_owned(font_data.0);
    font_data.tweak.scale = conf.ui.font_scale;
    fonts.font_data.insert("m+_font".to_owned(), font_data.into());
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

fn setup_style(mut egui_ctxs: EguiContexts) {
    let ctx = egui_ctxs.ctx_mut();
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style()).clone();

    style
        .text_styles
        .get_mut(&egui::TextStyle::Heading)
        .unwrap()
        .size = 14.0;

    let cr = egui::CornerRadius::same(2);
    style.visuals.window_corner_radius = cr;
    style.visuals.menu_corner_radius = cr;
    style.visuals.widgets.noninteractive.corner_radius = cr;
    style.visuals.widgets.inactive.corner_radius = cr;
    style.visuals.widgets.hovered.corner_radius = cr;
    style.visuals.widgets.active.corner_radius = cr;
    style.visuals.widgets.open.corner_radius = cr;
    let bg_color = egui::Color32::from_rgb(7, 12, 24);
    style.visuals.window_fill = bg_color;
    style.visuals.panel_fill = bg_color;
    let button_color = egui::Color32::from_rgb(53, 57, 78);
    style.visuals.widgets.inactive.weak_bg_fill = button_color;
    style.visuals.widgets.hovered.weak_bg_fill = button_color;
    style.visuals.widgets.open.weak_bg_fill = egui::Color32::from_rgb(33, 36, 52);
    let text_color = egui::Color32::from_rgb(208, 208, 208);
    style.visuals.widgets.noninteractive.fg_stroke.color = text_color;
    style.visuals.widgets.inactive.fg_stroke.color = text_color;
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.interaction.tooltip_delay = 0.04;

    style.visuals.window_stroke.color = egui::Color32::from_rgb(34, 48, 78);
    style.visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(36, 52, 84);
    style.visuals.widgets.hovered.bg_stroke.color = egui::Color32::from_rgb(158, 210, 245);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(98, 112, 152);
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(10, 80, 118);

    // style.debug.debug_on_hover = true;

    ctx.set_style(style);
}

fn load_textures(
    mut commands: Commands,
    mut egui_ctxs: EguiContexts,
    images: Res<Assets<Image>>,
    ui_assets: Res<UiAssets>,
    params: Res<Params>,
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
    let textures = ui.chain(start_planets).chain(animal_imgs);

    let mut egui_textures = HashMap::new();
    for (path, handle) in textures {
        let image = images.get(&handle).unwrap();
        let Some((path, _)) = path.split_once(".") else {
            continue;
        };
        let (texture_handle, size) = bevy_image_to_egui_texture(ctx, image, path, &params);

        egui_textures.insert(
            path.to_owned(),
            SizedTexture {
                id: texture_handle.id(),
                size,
            },
        );
        texture_handles.push(texture_handle);
    }

    commands.insert_resource(UiTextures(egui_textures));
}

fn bevy_image_to_egui_texture(
    ctx: &egui::Context,
    image: &bevy::prelude::Image,
    name: &str,
    params: &Params,
) -> (egui::TextureHandle, egui::Vec2) {
    let image = image
        .clone()
        .try_into_dynamic()
        .unwrap_or_else(|_| panic!("not supported image format: {}", name))
        .into_rgba8();
    let w = image.width();
    let h = image.height();

    let (w, h) = if let Some(animal) = name.strip_prefix("animals/") {
        let attr = &params.animals[&AnimalId::from(animal).unwrap()];
        let nw = if attr.civ.is_some() { 3 } else { 2 };
        (w / nw, h / 2)
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

    (
        ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST),
        egui::Vec2::new(w as f32, h as f32),
    )
}

pub fn reset_window_open_state(
    mut wos: ResMut<WindowsOpenState>,
    mut er_switch_planet: EventReader<SwitchPlanet>,
) {
    if er_switch_planet.read().last().is_some() {
        wos.dialogs.clear();
    }
}

fn check_windows_open_state(
    wos: Res<WindowsOpenState>,
    se_player: crate::audio::SoundEffectPlayer,
    mut prev_wos: Local<WindowsOpenState>,
) {
    let mut opened = false;
    let mut closed = false;

    for (current, prev) in wos.open_bools().into_iter().zip(prev_wos.open_bools()) {
        if !current && prev {
            closed = true;
        }
        if current && !prev {
            opened = true;
        }
    }

    if !se_player.is_playing() {
        if opened {
            se_player.play("window-open");
        } else if closed {
            se_player.play("window-close");
        }
    }

    *prev_wos = wos.clone();
}

impl WindowsOpenState {
    fn open_bools(&self) -> [bool; 12] {
        [
            self.space_building,
            self.animals,
            self.control,
            self.map,
            self.stat,
            self.reports,
            self.help,
            self.save,
            self.load,
            self.achivements,
            self.preferences,
            self.debug_tools,
        ]
    }
}
