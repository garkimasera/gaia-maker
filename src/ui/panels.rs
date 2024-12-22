use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};
use geom::Coords;

use crate::{
    conf::Conf,
    planet::{Params, Planet, KELVIN_CELSIUS},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::{ManagePlanet, SaveSlot},
    text::WithUnitDisplay,
    GameSpeed, GameState,
};

use super::{help::HelpItem, EguiTextures, WindowsOpenState};

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
    (planet, textures, params, save_slot, conf): (
        Res<Planet>,
        Res<EguiTextures>,
        Res<Params>,
        Res<SaveSlot>,
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
                    *save_slot,
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
    save_slot: SaveSlot,
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

    ui.menu_image_button(textures.get("ui/icon-build"), |ui| {
        build_menu(ui, cursor_mode, textures, planet, params);
    });

    ui.menu_image_button(textures.get("ui/icon-action"), |ui| {
        action_menu(ui, cursor_mode, params);
    });

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

    let image = textures.get("ui/icon-game-menu");
    ui.menu_image_button(image, |ui| {
        game_menu(
            ui,
            wos,
            save_slot,
            app_exit_events,
            ew_manage_planet,
            next_game_state,
        );
    });

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
        ui.label(format!(
            "{:.1} / {:.1} TW",
            planet.res.used_energy, planet.res.energy
        ));
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
        let warn = crate::action::cursor_mode_warn(planet, params, cursor_mode);
        let bg_color = if warn.is_none() {
            egui::Color32::DARK_GRAY
        } else {
            egui::Color32::DARK_RED
        };
        let bg_color = egui::lerp(
            egui::Rgba::from(bg_color)..=egui::Rgba::from(ui.visuals().window_fill()),
            0.9,
        );
        ui.painter()
            .rect_filled(ui.available_rect_before_wrap(), 6.0, bg_color);
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
                format!("{} {}", t!("animal"), t!(animal_id))
            }
            CursorMode::EditBiome(biome) => {
                format!("biome editing: {}", biome.as_ref())
            }
            CursorMode::PlaceSettlement(settlement) => {
                format!("settlement: {}", settlement.age.as_ref())
            }
        };
        ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
        if let Some(warn) = warn {
            ui.label(egui::RichText::new(warn).color(egui::Color32::RED));
        }
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
    egui::Grid::new("build_menu").striped(true).show(ui, |ui| {
        for kind in &planet.player.buildable_structures {
            if ui.button(t!(kind)).clicked() {
                *cursor_mode = CursorMode::Build(*kind);
                ui.close_menu();
            }
            ui.label("?")
                .on_hover_ui(|ui| HelpItem::Structures(*kind).ui(ui, textures, params));
            ui.end_row();
        }
    });
}

fn action_menu(ui: &mut egui::Ui, cursor_mode: &mut CursorMode, params: &Params) {
    for &kind in params.event.tile_event_costs.keys() {
        if ui.button(t!(kind)).clicked() {
            *cursor_mode = CursorMode::TileEvent(kind);
            ui.close_menu();
        }
        ui.end_row();
    }
}

fn game_menu(
    ui: &mut egui::Ui,
    wos: &mut WindowsOpenState,
    save_slot: SaveSlot,
    app_exit_events: &mut EventWriter<AppExit>,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    next_game_state: &mut NextState<GameState>,
) {
    ui.scope(|ui| {
        if let Some(slot) = save_slot.0 {
            if ui.button(t!("save-to-slot")).clicked() {
                ew_manage_planet.send(ManagePlanet::Save(slot));
                ui.close_menu();
            }
        } else {
            ui.disable();
            let _ = ui.button(t!("save-to-slot-disabled"));
        }
    });
    if ui.button(format!("{}...", t!("save-as"))).clicked() {
        wos.save = true;
        ui.close_menu();
    }
    if ui.button(format!("{}...", t!("load"))).clicked() {
        wos.load = true;
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
        crate::screen::window_close();
    }
}
