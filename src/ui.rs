use bevy::{
    app::AppExit,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    sprite::Rect,
};
use bevy_egui::{
    egui::{self, FontData, FontDefinitions, FontFamily},
    EguiContext, EguiPlugin, EguiSettings,
};

use std::collections::{HashMap, VecDeque};

use crate::{
    assets::{UiTexture, UiTextures},
    msg::MsgKind,
    planet::*,
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::ManagePlanet,
    GameState,
};

#[derive(Clone, Copy, Debug)]
pub struct UiPlugin {
    pub edit_map: bool,
}

#[derive(Clone, Default, Debug)]
pub struct WindowsOpenState {
    edit_map: bool,
    build: bool,
    message: bool,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Default)]
pub struct EguiTextures(HashMap<UiTexture, (egui::TextureHandle, egui::Vec2)>);

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_startup_system(setup)
            .insert_resource(WindowsOpenState {
                edit_map: self.edit_map,
                message: true,
                ..default()
            })
            .init_resource::<UiConf>()
            .add_system_set(SystemSet::on_exit(GameState::AssetLoading).with_system(load_textures))
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(panels.label("ui_panels").before("ui_windows"))
                    .with_system(msg_window.label("ui_windows"))
                    .with_system(build_window.label("ui_windows"))
                    .with_system(edit_map_window.label("ui_windows")),
            )
            .add_system(exit_on_esc_system);
    }
}

fn setup(
    mut egui_ctx: ResMut<EguiContext>,
    mut egui_settings: ResMut<EguiSettings>,
    conf: Res<UiConf>,
) {
    egui_settings.scale_factor = conf.scale_factor.into();

    let mut fonts = FontDefinitions::default();
    let mut font_data = FontData::from_static(include_bytes!("../fonts/Mplus2-SemiBold.otf"));
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
    mut _app_exit_events: EventWriter<AppExit>,
) {
    for event in keyboard_input_events.iter() {
        if let Some(key_code) = event.key_code {
            if event.state == ButtonState::Pressed && key_code == KeyCode::Escape {
                std::process::exit(0);
                // app_exit_events.send(bevy::app::AppExit);
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
    ui.label(&format!("{}: {:.0}", t!("energy"), planet.player.energy));
    ui.label(&format!(
        "{}: {:.0}",
        t!("material"),
        planet.player.material
    ));

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
        .add(egui::Button::image_and_text(handle.id(), conf.tex_size(*size), t!("build")).small())
        .clicked()
    {
        wos.build = !wos.build;
    }

    let (handle, size) = textures.0.get(&UiTexture::IconMessage).unwrap();
    if ui
        .add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
        .clicked()
    {
        wos.message = !wos.message;
    }
}

fn build_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut cursor_mode: ResMut<CursorMode>,
    conf: Res<UiConf>,
    planet: Res<Planet>,
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
                        if ui.button(t!(s)).clicked() {
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

fn edit_map_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut cursor_mode: ResMut<CursorMode>,
    wos: Res<WindowsOpenState>,
    conf: Res<UiConf>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    (mut new_w, mut new_h): (Local<u32>, Local<u32>),
    mut biome: Local<Biome>,
    mut save_file_path: Local<String>,
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
                        use strum::IntoEnumIterator;
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

            ui.separator();
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut *save_file_path));
                if ui.button("Save").clicked() {
                    ew_manage_planet.send(ManagePlanet::Save(save_file_path.clone()));
                }
                if ui.button("Load").clicked() {
                    ew_manage_planet.send(ManagePlanet::Load(save_file_path.clone()));
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
