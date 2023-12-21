use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, EguiContexts};

use crate::{
    assets::UiTexture,
    conf::Conf,
    draw::UpdateMap,
    overlay::OverlayLayerKind,
    planet::{Params, Planet, Structure, KELVIN_CELSIUS},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    sim::ManagePlanet,
    text::Unit,
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
    (mut current_layer, mut update_map): (ResMut<OverlayLayerKind>, ResMut<UpdateMap>),
    (mut app_exit_events, mut ew_manage_planet, mut next_game_state): (
        EventWriter<AppExit>,
        EventWriter<ManagePlanet>,
        ResMut<NextState<GameState>>,
    ),
    planet: Res<Planet>,
    textures: Res<EguiTextures>,
    params: Res<Params>,
    conf: Res<Conf>,
) {
    occupied_screen_space.window_rects.clear();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            sidebar(
                ui,
                &mut wos,
                &cursor_mode,
                &planet,
                hover_tile.single(),
                &textures,
                &conf,
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(
                    ui,
                    &mut cursor_mode,
                    &mut wos,
                    &mut speed,
                    (&mut current_layer, &mut update_map),
                    (
                        &mut app_exit_events,
                        &mut ew_manage_planet,
                        &mut next_game_state,
                    ),
                    &textures,
                    &planet,
                    &params,
                    &conf,
                );
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.scale_factor;
}

fn toolbar(
    ui: &mut egui::Ui,
    cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    speed: &mut GameSpeed,
    (_current_layer, _update_map): (&mut OverlayLayerKind, &mut UpdateMap),
    (app_exit_events, ew_manage_planet, next_game_state): (
        &mut EventWriter<AppExit>,
        &mut EventWriter<ManagePlanet>,
        &mut NextState<GameState>,
    ),
    textures: &EguiTextures,
    planet: &Planet,
    params: &Params,
    _conf: &Conf,
) {
    let button = |ui: &mut egui::Ui, icon: UiTexture, s: &str| {
        ui.add(egui::ImageButton::new(textures.0[&icon]))
            .on_hover_text(t!(s))
            .clicked()
    };

    ui.menu_image_button(textures.0[&UiTexture::IconBuild], |ui| {
        build_menu(ui, cursor_mode, planet, params);
    });

    if button(ui, UiTexture::IconOrbit, "orbit") {
        wos.orbit = !wos.orbit;
    }

    if button(ui, UiTexture::IconStarSystem, "star-system") {
        wos.star_system = !wos.star_system;
    }

    if button(ui, UiTexture::IconMap, "map") {
        wos.map = !wos.map;
    }

    if button(ui, UiTexture::IconLayers, "layers") {
        wos.layers = !wos.layers;
    }

    if button(ui, UiTexture::IconStat, "statistics") {
        wos.stat = !wos.stat;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let texture = if *speed == GameSpeed::Paused {
        UiTexture::IconSpeedPausedSelected
    } else {
        UiTexture::IconSpeedPaused
    };
    if button(ui, texture, "speed-paused") {
        *speed = GameSpeed::Paused;
    }

    let texture = if *speed == GameSpeed::Normal {
        UiTexture::IconSpeedNormalSelected
    } else {
        UiTexture::IconSpeedNormal
    };
    if button(ui, texture, "speed-normal") {
        *speed = GameSpeed::Normal;
    }

    let texture = if *speed == GameSpeed::Fast {
        UiTexture::IconSpeedFastSelected
    } else {
        UiTexture::IconSpeedFast
    };
    if button(ui, texture, "speed-fast") {
        *speed = GameSpeed::Fast;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let image = textures.0[&UiTexture::IconGameMenu];
    ui.menu_image_button(image, |ui| {
        game_menu(ui, app_exit_events, ew_manage_planet, next_game_state);
    });

    if button(ui, UiTexture::IconHelp, "help") {
        wos.help = !wos.help;
    }
}

fn sidebar(
    ui: &mut egui::Ui,
    wos: &mut WindowsOpenState,
    cursor_mode: &CursorMode,
    planet: &Planet,
    hover_tile: &HoverTile,
    textures: &EguiTextures,
    conf: &Conf,
) {
    let mut stock: Vec<_> = planet.res.stock.iter().collect();
    stock.sort_by_key(|&(res, _)| res);
    for (kind, v) in stock.into_iter() {
        let texture = textures.0[&UiTexture::from(*kind)];
        ui.horizontal(|ui| {
            ui.image(texture).on_hover_text(t!(kind.as_ref()));
            ui.label(kind.display_with_value(*v).to_string());
            let diff = planet.res.diff[kind];
            let sign = if diff > 0.0 { '+' } else { '-' };
            ui.label(
                egui::RichText::new(format!("({}{})", sign, kind.display_with_value(diff.abs())))
                    .small(),
            );
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
        CursorMode::Demolition => {
            ui.label(t!("demolition"));
        }
        CursorMode::EditBiome(biome) => {
            ui.label(format!("biome editing: {}", biome.as_ref()));
        }
        CursorMode::PlaceSettlement(settlement) => {
            ui.label(format!("settlement: {}", settlement.age.as_ref()));
        }
    }

    ui.separator();

    // Information about the hovered tile
    if let Some(p) = hover_tile.0 {
        let tile = &planet.map[p];

        ui.horizontal(|ui| {
            ui.image(textures.0[&UiTexture::IconCoordinates])
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

        let items: &[(UiTexture, String, &str)] = &[
            (
                UiTexture::IconHeight,
                format!("{:.0} m", planet.height_above_sea_level(p)),
                "height",
            ),
            (
                UiTexture::IconAirTemprature,
                format!("{:.1} °C", tile.temp - KELVIN_CELSIUS),
                "air-temprature",
            ),
            (
                UiTexture::IconRainfall,
                format!("{:.0} mm", tile.rainfall),
                "rainfall",
            ),
            (
                UiTexture::IconFertility,
                format!("{:.0} %", tile.fertility),
                "fertility",
            ),
            (
                UiTexture::IconBiomass,
                format!("{:.1} kg/m²", tile.biomass),
                "biomass",
            ),
        ];

        for (icon, label, s) in items {
            let s = t!(s);
            ui.horizontal(|ui| {
                ui.image(textures.0[icon]).on_hover_text(&s);
                ui.label(label).on_hover_text(s);
            });
        }

        ui.separator();

        ui.label(t!(tile.biome.as_ref()));

        let s = match &tile.structure {
            Structure::None => None,
            Structure::Occupied { by } => {
                Some(crate::info::structure_info(&planet.map[*by].structure))
            }
            other => Some(crate::info::structure_info(other)),
        };

        if let Some(s) = s {
            ui.label(s);
        } else {
            ui.label("");
        }

        ui.separator();

        super::message::msg_list(ui, wos, planet, conf);
    }
}

fn build_menu(ui: &mut egui::Ui, cursor_mode: &mut CursorMode, planet: &Planet, params: &Params) {
    if ui.button(t!("demolition")).clicked() {
        *cursor_mode = CursorMode::Demolition;
        ui.close_menu();
    }
    ui.separator();
    egui::Grid::new("build_menu").striped(true).show(ui, |ui| {
        for kind in &planet.player.buildable_structures {
            let s: &str = kind.as_ref();
            if ui.button(t!(s)).clicked() {
                *cursor_mode = CursorMode::Build(*kind);
                ui.close_menu();
            }
            ui.label("?")
                .on_hover_ui(|ui| HelpItem::Structures(*kind).ui(ui, params));
            ui.end_row();
        }
    });
}

fn game_menu(
    ui: &mut egui::Ui,
    app_exit_events: &mut EventWriter<AppExit>,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    next_game_state: &mut NextState<GameState>,
) {
    if ui.button(t!("save")).clicked() {
        ew_manage_planet.send(ManagePlanet::Save("main.planet".into()));
        ui.close_menu();
    }
    if ui.button(t!("load")).clicked() {
        ew_manage_planet.send(ManagePlanet::Load("main.planet".into()));
        ui.close_menu();
    }
    ui.separator();
    if ui.button(t!("main-menu")).clicked() {
        next_game_state.set(GameState::MainMenu);
        ui.close_menu();
    }
    ui.separator();
    if ui.button(t!("exit")).clicked() {
        app_exit_events.send(bevy::app::AppExit);
        ui.close_menu();
        crate::screen::window_close();
    }
}
