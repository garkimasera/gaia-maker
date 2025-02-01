use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use geom::Coords;

use crate::{
    conf::Conf,
    planet::{Cost, Params, Planet, KELVIN_CELSIUS},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::{ManagePlanet, SaveFileMetadata},
    text::WithUnitDisplay,
    GameSpeed, GameState,
};

use super::{help::HelpItem, EguiTextures, LabelWithIcon, WindowsOpenState};

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
    (planet, textures, params, save_file_metadata, conf): (
        Res<Planet>,
        Res<EguiTextures>,
        Res<Params>,
        Res<SaveFileMetadata>,
        Res<Conf>,
    ),
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
                    &save_file_metadata,
                    (
                        &mut app_exit_events,
                        &mut ew_manage_planet,
                        &mut next_game_state,
                    ),
                    &textures,
                    &planet,
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
    save_file_metadata: &SaveFileMetadata,
    (app_exit_events, ew_manage_planet, next_game_state): (
        &mut EventWriter<AppExit>,
        &mut EventWriter<ManagePlanet>,
        &mut NextState<GameState>,
    ),
    textures: &EguiTextures,
    planet: &Planet,
    params: &Params,
) {
    let button = |ui: &mut egui::Ui, path: &str, s: &str| {
        ui.add(egui::ImageButton::new(textures.get(path)))
            .on_hover_text(t!(s))
            .clicked()
    };
    let menu_button =
        |path: &str| egui::Button::image(textures.get(path)).min_size(egui::Vec2::new(30.0, 24.0));

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-game-menu"), |ui| {
        game_menu(
            ui,
            wos,
            save_file_metadata,
            app_exit_events,
            ew_manage_planet,
            next_game_state,
        );
    });

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-build"), |ui| {
        build_menu(ui, cursor_mode, textures, planet, params);
    });

    egui::menu::menu_custom_button(ui, menu_button("ui/icon-action"), |ui| {
        action_menu(ui, cursor_mode, textures, params);
    });

    ui.add(egui::Separator::default().spacing(2.0).vertical());

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

    let texture = if *speed == GameSpeed::Paused {
        "ui/icon-speed-paused-selected"
    } else {
        "ui/icon-speed-paused"
    };
    if button(ui, texture, "speed-paused") {
        *speed = GameSpeed::Paused;
    }

    let texture = if *speed == GameSpeed::Normal {
        "ui/icon-speed-normal-selected"
    } else {
        "ui/icon-speed-normal"
    };
    if button(ui, texture, "speed-normal") {
        *speed = GameSpeed::Normal;
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
    textures: &EguiTextures,
    conf: &Conf,
    last_hover_tile: &mut Option<Coords>,
) {
    // Energy
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-energy");
        ui.image(texture).on_hover_text(t!("energy"));
        let used_energy = crate::text::format_float_1000(planet.res.used_energy, 1);
        let energy = crate::text::format_float_1000(planet.res.energy, 1);
        ui.label(format!("{} / {} TW", used_energy, energy));
    });
    // Material
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-material");
        ui.image(texture).on_hover_text(t!("material"));
        ui.label(WithUnitDisplay::Material(planet.res.material).to_string());
        ui.label(
            egui::RichText::new(format!(
                "(+{})",
                WithUnitDisplay::Material(planet.res.diff_material)
            ))
            .small(),
        );
    });
    // Gene point
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-gene");
        ui.image(texture).on_hover_text(t!("gene-points"));
        ui.label(WithUnitDisplay::GenePoint(planet.res.gene_point).to_string());
        ui.label(egui::RichText::new(format!("({:+.2})", planet.res.diff_gene_point)).small());
    });
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
            CursorMode::PlaceSettlement(id, age) => {
                format!("settlement: {} {:?}", id, t!(age))
            }
        };
        ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
        ui.horizontal(|ui| {
            for (lack, cost) in &cost_list {
                let (texture, s) = match cost {
                    Cost::Energy(value, _) => (
                        textures.get("ui/icon-energy"),
                        WithUnitDisplay::Energy(*value).to_string(),
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

    if let Some(structure) = &tile.structure {
        ui.label(crate::info::structure_info(structure));
    } else {
        ui.label("");
    }

    ui.separator();

    super::dialog::msg_list(ui, wos, planet, conf);
}

fn build_menu(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    textures: &EguiTextures,
    planet: &Planet,
    params: &Params,
) {
    if ui.button(t!("demolition")).clicked() {
        *cursor_mode = CursorMode::Demolition;
        ui.close_menu();
    }
    ui.separator();
    let pos_tooltip = ui.response().rect.right_top() + egui::Vec2::new(16.0, 0.0);
    for kind in &planet.player.buildable_structures {
        let response = ui.button(t!(kind));
        if response.clicked() {
            *cursor_mode = CursorMode::Build(*kind);
            ui.close_menu();
        }
        if response.hovered() {
            egui::containers::show_tooltip_at(
                &response.ctx,
                response.layer_id,
                response.id,
                pos_tooltip,
                |ui| HelpItem::Structures(*kind).ui(ui, textures, params),
            );
        }
        ui.end_row();
    }
}

fn action_menu(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    textures: &EguiTextures,
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
                |ui| HelpItem::TileEvent(kind).ui(ui, textures, params),
            );
        }
        ui.end_row();
    }
}

fn game_menu(
    ui: &mut egui::Ui,
    wos: &mut WindowsOpenState,
    save_file_metadata: &SaveFileMetadata,
    app_exit_events: &mut EventWriter<AppExit>,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    next_game_state: &mut NextState<GameState>,
) {
    ui.scope(|ui| {
        if let Some(slot) = save_file_metadata.manual_slot {
            if ui.button(t!("save-to-slot"; slot=slot)).clicked() {
                ew_manage_planet.send(ManagePlanet::Save(slot));
                ui.close_menu();
            }
        } else {
            ui.disable();
            let _ = ui.button(t!("save-to-slot-disabled"));
        }
    });
    if ui.button(format!("{}...", t!("save"))).clicked() {
        wos.save = true;
        ui.close_menu();
    }
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
