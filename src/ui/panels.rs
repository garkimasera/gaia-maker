use bevy::{app::AppExit, diagnostic::DiagnosticsStore, prelude::*};
use bevy_egui::{EguiContexts, egui};
use geom::Coords;
use strum::IntoEnumIterator;

use crate::{
    GameSpeed, GameState,
    achivement_save::AchivementNotification,
    conf::Conf,
    manage_planet::ManagePlanet,
    planet::{Cost, Params, Planet, StructureKind},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    text::WithUnitDisplay,
    ui::tile_info::ui_tile_info,
};

use super::{UiTextures, WindowsOpenState, help::HelpItem, misc::LabelWithIcon};

pub fn panels(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    (mut app_exit_events, mut ew_manage_planet): (EventWriter<AppExit>, EventWriter<ManagePlanet>),
    mut next_game_state: ResMut<NextState<GameState>>,
    (planet, textures, params, conf): (Res<Planet>, Res<UiTextures>, Res<Params>, Res<Conf>),
    diagnostics_store: Res<DiagnosticsStore>,
    mut achivement_notification: ResMut<AchivementNotification>,
    mut last_hover_tile: Local<Option<Coords>>,
) {
    occupied_screen_space.reset();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .min_width(conf.ui.min_sidebar_width)
        .show(egui_ctxs.ctx_mut(), |ui| {
            sidebar(
                ui,
                &cursor_mode,
                &planet,
                &params,
                hover_tile.single(),
                &textures,
                &conf,
                &mut last_hover_tile,
                &diagnostics_store,
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
                    &mut achivement_notification,
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
    achivement_notification: &mut AchivementNotification,
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

    let resp = ui.add(egui::ImageButton::new(textures.get("ui/icon-achivements")));
    let resp = if let Some(achivement) = achivement_notification.achivement {
        resp.show_tooltip_ui(|ui| {
            ui.set_width(200.0);
            ui.strong(t!("new-achivement"));
            ui.horizontal(|ui| {
                ui.image(textures.get(format!("ui/achivement-{}", achivement.as_ref())));
                ui.label(t!("achivement", achivement.as_ref()));
            });
        });
        resp
    } else {
        resp.on_hover_text(t!("achivements"))
    };
    if resp.clicked() {
        *achivement_notification = AchivementNotification::default();
        wos.achivements = !wos.achivements;
    }

    if button(ui, "ui/icon-help", "help") {
        wos.help = !wos.help;
    }
}

fn sidebar(
    ui: &mut egui::Ui,
    cursor_mode: &CursorMode,
    planet: &Planet,
    params: &Params,
    hover_tile: &HoverTile,
    textures: &UiTextures,
    conf: &Conf,
    last_hover_tile: &mut Option<Coords>,
    diagnostics_store: &DiagnosticsStore,
) {
    // FPS indicator
    const FPS: bevy::diagnostic::DiagnosticPath =
        bevy::diagnostic::DiagnosticPath::const_new("fps");
    if conf.show_fps {
        if let Some(fps) = diagnostics_store.get(&FPS).and_then(|d| d.average()) {
            ui.label(format!("FPS: {:.2}", fps));
        }
    }

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
            CursorMode::SpawnAnimal(animal_id) => {
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
            CursorMode::CauseEvent(kind) => AsRef::<str>::as_ref(&kind).to_owned(),
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
    ui_tile_info(ui, p, planet, textures);
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
                    HelpItem::Facilities(kind).ui(ui, textures, params);
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
                    HelpItem::TileEvents(kind).ui(ui, textures, params);
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
