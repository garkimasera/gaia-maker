mod orbit;
mod star_system;
mod stat;

use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    math::Rect,
    prelude::*,
};
use bevy_egui::{
    egui::{self, FontData, FontDefinitions, FontFamily, RichText, Ui},
    EguiContext, EguiPlugin, EguiSettings,
};
use std::collections::{HashMap, VecDeque};
use strum::IntoEnumIterator;

use crate::{
    assets::{UiFonts, UiTexture, UiTextures},
    gz::GunzipBin,
    msg::MsgKind,
    overlay::OverlayLayerKind,
    planet::*,
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::ManagePlanet,
    GameState,
};

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin {
    pub edit_map: bool,
}

#[derive(Clone, Default, Debug, Resource)]
pub struct WindowsOpenState {
    build: bool,
    orbit: bool,
    star_system: bool,
    layers: bool,
    stat: bool,
    message: bool,
    game_menu: bool,
    edit_map: bool,
}

#[derive(Clone, Debug, Resource)]
pub struct UiConf {
    pub scale_factor: f32,
    pub font_scale: f32,
    pub max_message: usize,
}

impl Default for UiConf {
    fn default() -> Self {
        Self {
            scale_factor: 1.0,
            font_scale: 1.4,
            max_message: 20,
        }
    }
}

#[derive(Clone, Default, Resource)]
pub struct EguiTextures(HashMap<UiTexture, (egui::TextureHandle, egui::Vec2)>);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(WindowsOpenState {
                edit_map: self.edit_map,
                message: true,
                ..default()
            })
            .init_resource::<UiConf>()
            .init_resource::<OverlayLayerKind>()
            .add_system_set(
                SystemSet::on_exit(GameState::AssetLoading)
                    .with_system(setup_fonts)
                    .with_system(load_textures),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(panels.label("ui_panels").before("ui_windows"))
                    .with_system(build_window.label("ui_windows"))
                    .with_system(orbit::orbit_window.label("ui_windows"))
                    .with_system(star_system::star_system_window.label("ui_windows"))
                    .with_system(layers_window.label("ui_windows"))
                    .with_system(stat::stat_window.label("ui_windows"))
                    .with_system(msg_window.label("ui_windows"))
                    .with_system(game_menu_window.label("ui_windows"))
                    .with_system(edit_map_window.label("ui_windows")),
            )
            .add_system(exit_on_esc_system);
    }
}

fn setup_fonts(
    mut egui_ctx: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    conf: Res<UiConf>,
    fonts: Res<UiFonts>,
    gunzip_bin: Res<Assets<GunzipBin>>,
) {
    egui_settings.scale_factor = conf.scale_factor.into();

    let font_data = gunzip_bin.get(&fonts.font).unwrap().clone();
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
    egui_ctx.ctx_mut().set_fonts(fonts);
}

fn exit_on_esc_system(
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ButtonState::Pressed && key_code == KeyCode::Escape {
                app_exit_events.send(bevy::app::AppExit);
            }
        }
    }
}

fn load_textures(
    mut commands: Commands,
    mut egui_ctx: ResMut<EguiContext>,
    images: Res<Assets<Image>>,
    ui_textures: Res<UiTextures>,
) {
    let ctx = egui_ctx.ctx_mut();

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
            ctx.load_texture(k.as_ref(), color_image, egui::TextureFilter::Nearest);

        egui_textures.insert(*k, (texture_handle, size));
    }

    commands.insert_resource(EguiTextures(egui_textures));
}

fn panels(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    planet: Res<Planet>,
    textures: Res<EguiTextures>,
    conf: Res<UiConf>,
) {
    occupied_screen_space.window_rects.clear();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            sidebar(ui, &cursor_mode, &planet, hover_tile.get_single().unwrap());
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(ui, &mut cursor_mode, &mut wos, &textures, &conf);
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.scale_factor;
}

fn sidebar(ui: &mut egui::Ui, cursor_mode: &CursorMode, planet: &Planet, hover_tile: &HoverTile) {
    for (kind, v) in &planet.res.stock {
        ui.horizontal(|ui| {
            ui.label(&format!("{}: {:.1}", t!(kind.as_ref()), v,));
            ui.label(egui::RichText::new(&format!("({:+.1})", planet.res.diff[kind],)).small());
        });
    }

    ui.separator();

    // Information about selected tool
    ui.label(t!("selected-tool"));
    match cursor_mode {
        CursorMode::Normal => {
            ui.label(t!("none"));
        }
        CursorMode::Build(kind) => {
            ui.label(t!(kind.as_ref()));
        }
        CursorMode::EditBiome(biome) => {
            ui.label(format!("biome editing: {}", biome.as_ref()));
        }
    }

    ui.separator();

    // Information about the hovered tile
    if let Some(p) = hover_tile.0 {
        ui.label(format!("{}: [{}, {}]", t!("coordinates"), p.0, p.1));
        let tile = &planet.map[p];

        ui.label(format!(
            "{}: {:.1} °C",
            t!("air-temprature"),
            tile.temp - 273.15
        ));

        let s = match &tile.structure {
            Structure::None => None,
            Structure::Occupied { by } => {
                Some(crate::info::structure_info(&planet.map[*by].structure))
            }
            other => Some(crate::info::structure_info(other)),
        };

        if let Some(s) = s {
            ui.label(s);
        }
    } else {
        ui.label(format!("{}: -", t!("coordinates")));
    };
}

fn toolbar(
    ui: &mut egui::Ui,
    _cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    textures: &EguiTextures,
    conf: &UiConf,
) {
    let (handle, size) = textures.0.get(&UiTexture::IconBuild).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("build"))
        .clicked()
    {
        wos.build = !wos.build;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconOrbit).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("orbit"))
        .clicked()
    {
        wos.orbit = !wos.orbit;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconStarSystem).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("star-system"))
        .clicked()
    {
        wos.star_system = !wos.star_system;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconLayers).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("layers"))
        .clicked()
    {
        wos.layers = !wos.layers;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconStat).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("statistics"))
        .clicked()
    {
        wos.stat = !wos.stat;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let (handle, size) = textures.0.get(&UiTexture::IconMessage).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("messages"))
        .clicked()
    {
        wos.message = !wos.message;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconGameMenu).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .on_hover_text(t!("menu"))
        .clicked()
    {
        wos.game_menu = !wos.game_menu;
    }
}

fn build_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut cursor_mode: ResMut<CursorMode>,
    conf: Res<UiConf>,
    planet: Res<Planet>,
    params: Res<Params>,
) {
    if !wos.build {
        return;
    }

    let rect = egui::Window::new(t!("build"))
        .open(&mut wos.build)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    for kind in &planet.player.buildable_structures {
                        let s: &str = kind.as_ref();
                        if ui
                            .button(t!(s))
                            .on_hover_ui(build_button_tooltip(*kind, &params))
                            .clicked()
                        {
                            *cursor_mode = CursorMode::Build(*kind);
                        }
                    }
                });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn build_button_tooltip(kind: StructureKind, params: &Params) -> impl FnOnce(&mut Ui) + '_ {
    building_desc_tooltip(&params.structures[&kind].building)
}

fn building_desc_tooltip(attrs: &BuildingAttrs) -> impl FnOnce(&mut Ui) + '_ {
    move |ui| {
        if !attrs.cost.is_empty() {
            ui.label(RichText::new(t!("cost")).strong());
            let mut resources = attrs.cost.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .into_iter()
                .map(|(resource, value)| format!("{}: {}", t!(resource.as_ref()), value))
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
        if !attrs.upkeep.is_empty() {
            ui.label(RichText::new(t!("upkeep")).strong());
            let mut resources = attrs.upkeep.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .iter()
                .map(|(resource, value)| format!("{}: {}", t!(resource.as_ref()), value))
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
        if !attrs.produces.is_empty() {
            ui.label(RichText::new(t!("produces")).strong());
            let mut resources = attrs.produces.iter().collect::<Vec<_>>();
            resources.sort_by_key(|(resource, _)| *resource);
            let s = resources
                .iter()
                .map(|(resource, value)| format!("{}: {}", t!(resource.as_ref()), value))
                .fold(String::new(), |mut s0, s1| {
                    if !s0.is_empty() {
                        s0.push_str(", ");
                    }
                    s0.push_str(&s1);
                    s0
                });
            ui.label(s);
        }
    }
}

fn layers_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut current_layer: ResMut<OverlayLayerKind>,
    conf: Res<UiConf>,
    _planet: Res<Planet>,
) {
    if !wos.layers {
        return;
    }

    let rect = egui::Window::new(t!("layers"))
        .open(&mut wos.layers)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            for kind in OverlayLayerKind::iter() {
                ui.radio_value(&mut *current_layer, kind, t!(kind.as_ref()));
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn msg_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut msgs: Local<VecDeque<(MsgKind, String)>>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<UiConf>,
) {
    if !wos.message {
        return;
    }

    while let Some(msg) = crate::msg::pop_msg() {
        msgs.push_front(msg);
        if msgs.len() > conf.max_message {
            msgs.pop_back();
        }
    }

    let rect = egui::Window::new(t!("messages"))
        .open(&mut wos.message)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| {
                    for (_kind, msg) in msgs.iter() {
                        ui.label(msg);
                        ui.separator();
                    }
                });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn game_menu_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut app_exit_events: EventWriter<AppExit>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    conf: Res<UiConf>,
) {
    if !wos.game_menu {
        return;
    }

    let mut close = false;

    let rect = egui::Window::new(t!("menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .resizable(false)
        .open(&mut wos.game_menu)
        .show(egui_ctx.ctx_mut(), |ui| {
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

fn edit_map_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut cursor_mode: ResMut<CursorMode>,
    wos: Res<WindowsOpenState>,
    conf: Res<UiConf>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    (mut new_w, mut new_h): (Local<u32>, Local<u32>),
    mut biome: Local<Biome>,
) {
    if !wos.edit_map {
        return;
    }

    let rect = egui::Window::new("Map editing tools")
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.add(egui::Slider::new(&mut *new_w, 2..=100).text("width"));
            ui.horizontal(|ui| {
                ui.add(egui::Slider::new(&mut *new_h, 2..=100).text("height"));
                if ui.button("New").clicked() {
                    ew_manage_planet.send(ManagePlanet::New(*new_w, *new_h));
                }
            });

            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source(Biome::Ocean)
                    .selected_text(AsRef::<str>::as_ref(&*biome))
                    .show_ui(ui, |ui| {
                        for b in Biome::iter() {
                            ui.selectable_value(&mut *biome, b, AsRef::<str>::as_ref(&b));
                        }
                    });
                if ui.button("Edit biome").clicked()
                    || matches!(*cursor_mode, CursorMode::EditBiome(_))
                {
                    *cursor_mode = CursorMode::EditBiome(*biome);
                }
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn convert_rect(rect: bevy_egui::egui::Rect, scale_factor: f32) -> Rect {
    Rect {
        min: Vec2::new(rect.left() * scale_factor, rect.top() * scale_factor),
        max: Vec2::new(rect.right() * scale_factor, rect.bottom() * scale_factor),
    }
}

impl UiConf {
    fn tex_size(&self, size: egui::Vec2) -> egui::Vec2 {
        let factor = 1.0;
        egui::Vec2::new(size.x * factor, size.y * factor)
    }
}
