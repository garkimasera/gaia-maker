use bevy::{app::AppExit, prelude::*};
use bevy_egui::{EguiContexts, egui};
use strum::IntoEnumIterator;

use crate::{
    GameSpeed, GameState,
    achivement_save::AchivementNotification,
    conf::Conf,
    draw::{DisplayOpts, UpdateDraw},
    manage_planet::ManagePlanet,
    planet::{Params, Planet, StructureKind},
    screen::{CursorMode, OccupiedScreenSpace},
};

use super::{UiTextures, WindowsOpenState, help::HelpItem};

pub fn toolbar(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    (mut app_exit_events, mut ew_manage_planet): (EventWriter<AppExit>, EventWriter<ManagePlanet>),
    mut next_game_state: ResMut<NextState<GameState>>,
    planet: Res<Planet>,
    (textures, params, conf): (Res<UiTextures>, Res<Params>, Res<Conf>),
    mut achivement_notification: ResMut<AchivementNotification>,
    mut display_opts: ResMut<DisplayOpts>,
    mut update_draw: ResMut<UpdateDraw>,
) {
    occupied_screen_space.reset();

    let height = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar_ui(
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
                    &planet,
                    &params,
                    &mut achivement_notification,
                    &mut display_opts,
                    &mut update_draw,
                );
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();

    occupied_screen_space.toolbar_height = height;
    occupied_screen_space.occupied_top = height * conf.ui.scale_factor;
}

fn toolbar_ui(
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
    planet: &Planet,
    params: &Params,
    achivement_notification: &mut AchivementNotification,
    display_opts: &mut DisplayOpts,
    update_draw: &mut UpdateDraw,
) {
    // Menu buttons

    let button = |ui: &mut egui::Ui, path: &str, s: &str| {
        ui.add(egui::ImageButton::new(textures.get(path)))
            .on_hover_text(t!(s))
            .clicked()
    };
    let menu_button =
        |path: &str| egui::Button::image(textures.get(path)).min_size(egui::Vec2::new(30.0, 24.0));

    let menu_clicked = egui::menu::menu_custom_button(ui, menu_button("ui/icon-game-menu"), |ui| {
        game_menu(ui, wos, app_exit_events, ew_manage_planet, next_game_state);
    })
    .response
    .clicked();

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let menu_clicked = egui::menu::menu_custom_button(ui, menu_button("ui/icon-build"), |ui| {
        build_menu(ui, cursor_mode, textures, params);
    })
    .response
    .clicked()
        | menu_clicked;

    let menu_clicked = egui::menu::menu_custom_button(ui, menu_button("ui/icon-action"), |ui| {
        action_menu(ui, cursor_mode, textures, params);
    })
    .response
    .clicked()
        | menu_clicked;

    let menu_clicked = egui::menu::menu_custom_button(ui, menu_button("ui/icon-layers"), |ui| {
        layers_menu(ui, display_opts, update_draw);
    })
    .response
    .clicked()
        | menu_clicked;

    if menu_clicked {
        wos.space_building = false;
        wos.animals = false;
        wos.control = false;
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

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    // Resource indicators
    super::indicators::power_indicator(ui, textures, planet.res.power, planet.res.used_power);
    ui.separator();
    super::indicators::material_indicator(
        ui,
        textures,
        planet.res.material,
        planet.res.diff_material,
    );
    ui.separator();
    super::indicators::gene_point_indicator(
        ui,
        textures,
        planet.res.gene_point,
        planet.res.diff_gene_point,
    );
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

fn layers_menu(ui: &mut egui::Ui, display_opts: &mut DisplayOpts, update_draw: &mut UpdateDraw) {
    let old = *display_opts;
    ui.checkbox(&mut display_opts.animals, t!("animal"));
    ui.checkbox(&mut display_opts.cities, t!("cities"));
    ui.checkbox(&mut display_opts.city_icons, t!("city-icons"));
    ui.checkbox(&mut display_opts.structures, t!("structures"));
    if *display_opts != old {
        update_draw.update();
    }
}
