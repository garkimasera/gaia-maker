use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use geom::Coords;
use strum::IntoEnumIterator;

use crate::{
    conf::Conf,
    manage_planet::ManagePlanet,
    planet::{Cost, Params, Planet, StructureKind, KELVIN_CELSIUS},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    text::WithUnitDisplay,
    GameSpeed, GameState,
};

use super::{help::HelpItem, misc::LabelWithIcon, UiTextures, WindowsOpenState};

pub fn panels(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    (mut app_exit_events, mut ew_manage_planet, mut next_game_state): (
        EventWriter<AppExit>,
        EventWriter<ManagePlanet>,
        ResMut<NextState<GameState>>,
    ),
    (planet, textures, params, conf): (Res<Planet>, Res<UiTextures>, Res<Params>, Res<Conf>),
    mut last_hover_tile: Local<Option<Coords>>,
) {
    occupied_screen_space.reset();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .min_width(conf.ui.min_sidebar_width)
        .show(egui_ctxs.ctx_mut(), |ui| {
            sidebar(
                ui,
                &mut wos,
                &cursor_mode,
                &planet,
                &params,
                hover_tile.single(),
                &textures,
                &conf,
                &mut last_hover_tile,
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.ui.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(
                    ui,
                    &mut cursor_mode,
                    &mut wos,
                    &mut speed,
                    (
                        &mut app_exit_events,
                        &mut ew_manage_planet,
                        &mut next_game_state,
                    ),
                    &textures,
                    &params,
                );
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.ui.scale_factor;
}

fn toolbar(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    speed: &mut GameSpeed,
    (app_exit_events, ew_manage_planet, next_game_state): (
        &mut EventWriter<AppExit>,
        &mut EventWriter<ManagePlanet>,
        &mut NextState<GameState>,
    ),
    textures: &UiTextures,
    params: &Params,
) {
    // Menu buttons

    let button = |ui: &mut egui::Ui, path: &str, s: &str| {
        ui.add(egui::ImageButton::new(textures.get(path)))
            .on_hover_text(t!(s))
            .clicked()
    };
    let menu_button =
        |path: &str| egui::Button::image(textures.get(path)).min_size(egui::Vec2::new(30.0, 24.0));

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-game-menu"), |ui| {
        game_menu(ui, wos, app_exit_events, ew_manage_planet, next_game_state);
    });

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-build"), |ui| {
        build_menu(ui, cursor_mode, textures, params);
    });

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-action"), |ui| {
        action_menu(ui, cursor_mode, textures, params);
    });

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    // Window buttons

    if button(ui, "ui/icon-space-buildings", "space-buildings") {
        wos.space_building = !wos.space_building;
    }

    if button(ui, "ui/icon-animal", "animal") {
        wos.animals = !wos.animals;
    }

    if button(ui, "ui/icon-map", "map") {
        wos.map = !wos.map;
    }

    if button(ui, "ui/icon-layers", "layers") {
        wos.layers = !wos.layers;
    }

    if button(ui, "ui/icon-stat", "statistics") {
        wos.stat = !wos.stat;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    // Game speed selector

    let texture = if *speed == GameSpeed::Paused {
        "ui/icon-speed-paused-selected"
    } else {
        "ui/icon-speed-paused"
    };
    if button(ui, texture, "speed-paused") {
        *speed = GameSpeed::Paused;
    }

    let texture = if *speed == GameSpeed::Slow {
        "ui/icon-speed-slow-selected"
    } else {
        "ui/icon-speed-slow"
    };
    if button(ui, texture, "speed-slow") {
        *speed = GameSpeed::Slow;
    }

    let texture = if *speed == GameSpeed::Medium {
        "ui/icon-speed-medium-selected"
    } else {
        "ui/icon-speed-medium"
    };
    if button(ui, texture, "speed-medium") {
        *speed = GameSpeed::Medium;
    }

    let texture = if *speed == GameSpeed::Fast {
        "ui/icon-speed-fast-selected"
    } else {
        "ui/icon-speed-fast"
    };
    if button(ui, texture, "speed-fast") {
        *speed = GameSpeed::Fast;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    if button(ui, "ui/icon-help", "help") {
        wos.help = !wos.help;
    }
}

fn sidebar(
    ui: &mut egui::Ui,
    wos: &mut WindowsOpenState,
    cursor_mode: &CursorMode,
    planet: &Planet,
    params: &Params,
    hover_tile: &HoverTile,
    textures: &UiTextures,
    conf: &Conf,
    last_hover_tile: &mut Option<Coords>,
) {
    // Resource indicators
    super::misc::power_indicator(ui, textures, planet.res.power, planet.res.used_power);
    super::misc::material_indicator(ui, textures, planet.res.material, planet.res.diff_material);
    super::misc::gene_point_indicator(
        ui,
        textures,
        planet.res.gene_point,
        planet.res.diff_gene_point,
    );

    ui.separator();

    // Information about selected tool
    ui.vertical(|ui| {
        ui.set_height(50.0);
        ui.add_space(3.0);
        let cost_list = crate::action::cursor_mode_lack_and_cost(planet, params, cursor_mode);
        let bg_color = if cost_list.iter().any(|(lack, _)| *lack) {
            egui::Color32::DARK_RED
        } else {
            egui::Color32::DARK_GRAY
        };
        let bg_color = egui::lerp(
            egui::Rgba::from(bg_color)..=egui::Rgba::from(ui.visuals().window_fill()),
            0.9,
        );
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 6.0, bg_color);
        ui.add_space(2.0);
        let text = match cursor_mode {
            CursorMode::Normal => "".into(),
            CursorMode::Demolition => {
                t!("demolition")
            }
            CursorMode::Build(kind) => {
                t!(kind)
            }
            CursorMode::TileEvent(kind) => {
                t!(kind)
            }
            CursorMode::SpawnAnimal(ref animal_id) => {
                format!("{} {}", t!("animal"), t!("animal", animal_id))
            }
            CursorMode::EditBiome(biome) => {
                format!("biome editing: {}", biome.as_ref())
            }
            CursorMode::ChangeHeight(value) => {
                format!("change height: {}", value)
            }
            CursorMode::PlaceSettlement(id, age) => {
                format!("settlement: {} {:?}", id, age)
            }
        };
        ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
        ui.horizontal(|ui| {
            for (lack, cost) in &cost_list {
                let (texture, s) = match cost {
                    Cost::Power(value, _) => (
                        textures.get("ui/icon-power"),
                        WithUnitDisplay::Power(*value).to_string(),
                    ),
                    Cost::Material(value) => (
                        textures.get("ui/icon-material"),
                        WithUnitDisplay::Material(*value).to_string(),
                    ),
                    Cost::GenePoint(value) => (
                        textures.get("ui/icon-gene"),
                        WithUnitDisplay::GenePoint(*value).to_string(),
                    ),
                };
                if *lack {
                    ui.add(LabelWithIcon::new(
                        texture,
                        egui::RichText::new(s).color(egui::Color32::RED),
                    ));
                } else {
                    ui.add(LabelWithIcon::new(texture, s));
                }
            }
        });
    });

    ui.separator();

    // Information about the hovered tile
    last_hover_tile.get_or_insert(Coords(0, 0));
    if hover_tile.0.is_some() {
        *last_hover_tile = hover_tile.0;
    }

    let p = hover_tile.0.unwrap_or(last_hover_tile.unwrap());

    let tile = &planet.map[p];

    ui.horizontal(|ui| {
        ui.image(textures.get("ui/icon-coordinates"))
            .on_hover_text(t!("coordinates"));
        ui.label(format!("[{}, {}]", p.0, p.1))
            .on_hover_text(t!("coordinates"));

        let (longitude, latitude) = planet.calc_longitude_latitude(p);
        ui.label(format!(
            "{:.0}°, {:.0}°",
            longitude * 180.0 * std::f32::consts::FRAC_1_PI,
            latitude * 180.0 * std::f32::consts::FRAC_1_PI,
        ))
        .on_hover_text(format!("{}, {}", t!("longitude"), t!("latitude")));
    });

    let items: &[(&str, String, &str)] = &[
        (
            "ui/icon-height",
            format!("{:.0} m", planet.height_above_sea_level(p)),
            "height",
        ),
        (
            "ui/icon-air-temperature",
            format!("{:.1} °C", tile.temp - KELVIN_CELSIUS),
            "air-temperature",
        ),
        (
            "ui/icon-rainfall",
            format!("{:.0} mm", tile.rainfall),
            "rainfall",
        ),
        (
            "ui/icon-fertility",
            format!("{:.0} %", tile.fertility),
            "fertility",
        ),
        (
            "ui/icon-biomass",
            format!("{:.1} kg/m²", tile.biomass),
            "biomass",
        ),
    ];

    for (icon, label, s) in items {
        let s = t!(s);
        ui.horizontal(|ui| {
            ui.image(textures.get(icon)).on_hover_text(&s);
            ui.label(label).on_hover_text(s);
        });
    }

    ui.separator();

    ui.label(t!(tile.biome));

    let s = if let Some(structure) = &tile.structure {
        crate::info::structure_info(structure)
    } else if let Some(animal) = tile.largest_animal() {
        t!("animal", animal.id)
    } else {
        "".into()
    };
    ui.label(s);

    ui.separator();

    super::dialog::msg_list(ui, wos, planet, conf);
}

fn build_menu(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    textures: &UiTextures,
    params: &Params,
) {
    if ui.button(t!("demolition")).clicked() {
        *cursor_mode = CursorMode::Demolition;
        ui.close_menu();
    }
    ui.separator();
    let pos_tooltip = ui.response().rect.right_top() + egui::Vec2::new(16.0, 0.0);
    for kind in StructureKind::iter().filter(|kind| kind.buildable_by_player()) {
        let response = ui.button(t!(kind));
        if response.clicked() {
            *cursor_mode = CursorMode::Build(kind);
            ui.close_menu();
        }
        if response.hovered() {
            egui::containers::show_tooltip_at(
                &response.ctx,
                response.layer_id,
                response.id,
                pos_tooltip,
                |ui| {
                    ui.set_max_width(super::HELP_TOOLTIP_WIDTH);
                    HelpItem::Structures(kind).ui(ui, textures, params);
                },
            );
        }
        ui.end_row();
    }
}

fn action_menu(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    textures: &UiTextures,
    params: &Params,
) {
    let pos_tooltip = ui.response().rect.right_top() + egui::Vec2::new(16.0, 0.0);
    for &kind in params.event.tile_event_costs.keys() {
        let response = ui.button(t!(kind));
        if response.clicked() {
            *cursor_mode = CursorMode::TileEvent(kind);
            ui.close_menu();
        }
        if response.hovered() {
            egui::containers::show_tooltip_at(
                &response.ctx,
                response.layer_id,
                response.id,
                pos_tooltip,
                |ui| {
                    ui.set_max_width(super::HELP_TOOLTIP_WIDTH);
                    HelpItem::TileEvent(kind).ui(ui, textures, params);
                },
            );
        }
        ui.end_row();
    }
}

fn game_menu(
    ui: &mut egui::Ui,
    wos: &mut WindowsOpenState,
    app_exit_events: &mut EventWriter<AppExit>,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    next_game_state: &mut NextState<GameState>,
) {
    if ui.button(t!("save")).clicked() {
        ew_manage_planet.send(ManagePlanet::Save {
            auto: false,
            _new_name: None,
        });
        ui.close_menu();
    }
    // if ui.button(format!("{}...", t!("save-as"))).clicked() {
    //     wos.save = true;
    //     ui.close_menu();
    // }
    if ui.button(format!("{}...", t!("load"))).clicked() {
        wos.load = true;
        ui.close_menu();
    }
    ui.separator();
    if ui.button(format!("{}...", t!("preferences"))).clicked() {
        wos.preferences = true;
        ui.close_menu();
    }
    ui.separator();
    if ui.button(t!("main-menu")).clicked() {
        next_game_state.set(GameState::MainMenu);
        ui.close_menu();
    }
    ui.separator();
    if ui.button(t!("exit")).clicked() {
        app_exit_events.send(bevy::app::AppExit::Success);
        ui.close_menu();
        crate::platform::window_close();
    }
}
