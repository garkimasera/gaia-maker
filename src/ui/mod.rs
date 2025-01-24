mod animals;
mod debug_tools;
mod dialog;
mod error_popup;
mod help;
mod main_menu;
mod map;
mod new_planet;
mod panels;
mod preferences;
mod saveload;
mod space_buildings;
mod stat;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, epaint, load::SizedTexture, FontData, FontDefinitions, FontFamily},
    EguiContextSettings, EguiContexts, EguiPlugin,
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
    sim::ManagePlanetError,
    GameState,
};

use self::dialog::MsgDialog;

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
    pub save: bool,
    pub load: bool,
    pub error_popup: Option<ManagePlanetError>,
    pub preferences: bool,
    pub debug_tools: bool,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Dialog {
    Msg(MsgDialog),
    _Dummy,
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
                (setup_fonts, load_textures, setup_style),
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
                    saveload::saveload_window,
                    error_popup::error_popup,
                    preferences::preferences_window,
                    debug_tools::debug_tools_window,
                )
                    .run_if(in_state(GameState::Running))
                    .in_set(UiWindowsSystemSet),
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
    fonts
        .font_data
        .insert("m+_font".to_owned(), font_data.into());
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

    let rounding = egui::Rounding::same(2.0);
    style.visuals.window_rounding = rounding;
    style.visuals.menu_rounding = rounding;
    style.visuals.widgets.noninteractive.rounding = rounding;
    style.visuals.widgets.inactive.rounding = rounding;
    style.visuals.widgets.hovered.rounding = rounding;
    style.visuals.widgets.active.rounding = rounding;
    style.visuals.widgets.open.rounding = rounding;
    style.visuals.widgets.noninteractive.fg_stroke.color = egui::Color32::from_rgb(180, 180, 180);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.interaction.tooltip_delay = 0.2;

    ctx.set_style(style);
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
    let textures = ui.chain(start_planets).chain(animal_imgs);

    let mut egui_textures = HashMap::new();
    for (path, handle) in textures {
        let image = images.get(&handle).unwrap();
        let (texture_handle, size) = bevy_image_to_egui_texture(ctx, image, &path);

        let Some(path) = path.strip_suffix(".png") else {
            continue;
        };

        egui_textures.insert(
            path.to_owned(),
            SizedTexture {
                id: texture_handle.id(),
                size,
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
) -> (egui::TextureHandle, egui::Vec2) {
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

    (
        ctx.load_texture(name, color_image, egui::TextureOptions::NEAREST),
        egui::Vec2::new(w as f32, h as f32),
    )
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
        if ui.radio_value(&mut new_layer, kind, t!(kind)).clicked() {
            ui.close_menu();
        }
    }
    if new_layer != *current_layer {
        *current_layer = new_layer;
        update_map.update();
    }
}

fn label_with_icon(
    ui: &mut egui::Ui,
    textures: &EguiTextures,
    icon: &str,
    s: impl Into<egui::WidgetText>,
) {
    let icon = textures.get(icon);
    ui.add(LabelWithIcon::new(icon, s));
}

struct LabelWithIcon {
    icon: SizedTexture,
    text: egui::WidgetText,
}

impl LabelWithIcon {
    fn new(icon: impl Into<SizedTexture>, text: impl Into<egui::WidgetText>) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
        }
    }
}

impl egui::Widget for LabelWithIcon {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let layout_job =
            self.text
                .into_layout_job(ui.style(), egui::FontSelection::Default, egui::Align::Min);
        let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

        let icon_size = self.icon.size;
        let galley_size = galley.rect.size();
        let desired_size =
            egui::Vec2::new(icon_size.x + galley_size.x, icon_size.y.max(galley_size.y));

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), galley.text())
        });

        let (icon_pos_y, galley_pos_y) = if icon_size.y > galley_size.y {
            (0.0, (icon_size.y - galley_size.y) / 2.0)
        } else {
            ((galley_size.y - icon_size.y) / 2.0, 0.0)
        };

        let icon_rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, icon_pos_y), icon_size)
            .translate(rect.left_top().to_vec2());
        let galley_pos = rect.left_top() + egui::Vec2::new(icon_size.x, galley_pos_y);

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();
            painter.add(epaint::RectShape {
                rect: icon_rect,
                rounding: egui::Rounding::ZERO,
                fill: egui::Color32::WHITE,
                stroke: egui::Stroke::NONE,
                blur_width: 0.0,
                fill_texture_id: self.icon.id,
                uv: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            });
            painter.add(epaint::TextShape::new(
                galley_pos,
                galley,
                ui.style().visuals.text_color(),
            ));
        }
        response
    }
}
