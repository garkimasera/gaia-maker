use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::conf::{Conf, ConfChange};
use crate::planet::Params;
use crate::sim::{ManagePlanet, ManagePlanetError};
use crate::text::Lang;
use strum::IntoEnumIterator;

use super::new_planet::NewPlanetState;
use super::EguiTextures;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MainMenuMode {
    #[default]
    Menu,
    NewPlanet,
    Error,
}

#[derive(Clone, Debug, Resource)]
pub struct MainMenuState {
    pub mode: MainMenuMode,
    pub new_planet: NewPlanetState,
    pub error: Option<ManagePlanetError>,
}

impl MainMenuState {
    fn new(params: &Params) -> Self {
        MainMenuState {
            mode: MainMenuMode::Menu,
            new_planet: NewPlanetState::new(params),
            error: None,
        }
    }
}

pub fn set_main_menu_state(mut command: Commands, params: Res<Params>) {
    command.insert_resource(MainMenuState::new(&params));
}

pub fn main_menu(
    mut egui_ctxs: EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: Res<Params>,
    mut conf: ResMut<Conf>,
    mut ew_conf_change: EventWriter<ConfChange>,
    mut ew_manage_planet_error: EventReader<ManagePlanetError>,
    mut app_exit_events: EventWriter<AppExit>,
    mut state: ResMut<MainMenuState>,
    textures: Res<EguiTextures>,
) {
    if let Some(e) = ew_manage_planet_error.read().next() {
        state.mode = MainMenuMode::Error;
        state.error = Some(e.clone());
    }

    match state.mode {
        MainMenuMode::Menu => {
            egui::Window::new(t!("menu"))
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .default_width(0.0)
                .resizable(false)
                .show(egui_ctxs.ctx_mut(), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        ui.image(textures.get("logo"));

                        if ui.button(t!("new")).clicked() {
                            state.mode = MainMenuMode::NewPlanet;
                        }
                        if ui.button(t!("load")).clicked() {
                            ew_manage_planet.send(ManagePlanet::Load("main.planet".into()));
                        }
                        if ui.button(t!("exit")).clicked() {
                            app_exit_events.send(bevy::app::AppExit::Success);
                            crate::screen::window_close();
                        }

                        ui.separator();

                        if let Some(lang) = language_selector(ui, crate::text::get_lang()) {
                            conf.lang = lang;
                            crate::text::set_lang(lang);
                            ew_conf_change.send_default();
                        }
                    });
                })
                .unwrap();
        }
        MainMenuMode::NewPlanet => {
            super::new_planet::new_planet(
                &mut egui_ctxs,
                ew_manage_planet,
                &params,
                &mut state,
                &textures,
            );
        }
        MainMenuMode::Error => {
            egui::Window::new(t!("msg/loading-failed"))
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(egui_ctxs.ctx_mut(), |ui| {
                    if matches!(state.error, Some(ManagePlanetError::Decode)) {
                        ui.label(t!("msg/loading-failed-desc-decode-error"));
                    } else {
                        ui.label(t!("msg/loading-failed-desc-not-found"));
                    }
                    ui.vertical_centered(|ui| {
                        if ui.button(t!("close")).clicked() {
                            state.mode = MainMenuMode::Menu;
                        }
                    });
                })
                .unwrap();
        }
    }
}

fn language_selector(ui: &mut egui::Ui, before: Lang) -> Option<Lang> {
    let mut selected = before;
    egui::ComboBox::from_label("")
        .selected_text(selected.name())
        .show_ui(ui, |ui| {
            for lang in Lang::iter() {
                ui.selectable_value(&mut selected, lang, lang.name());
            }
        });

    if selected != before {
        Some(selected)
    } else {
        None
    }
}
