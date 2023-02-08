use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::conf::{Conf, ConfChange};
use crate::planet::Params;
use crate::sim::ManagePlanet;
use crate::text::Lang;
use strum::IntoEnumIterator;

use super::new_planet::NewPlanetState;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MainMenuWindow {
    #[default]
    Menu,
    NewPlanet,
}

#[derive(Clone, Debug, Resource)]
pub struct MainMenuState {
    pub current: MainMenuWindow,
    pub new_planet: NewPlanetState,
}

impl MainMenuState {
    fn new(params: &Params) -> Self {
        MainMenuState {
            current: MainMenuWindow::Menu,
            new_planet: NewPlanetState::new(params),
        }
    }
}

pub fn set_main_menu_state(mut command: Commands, params: Res<Params>) {
    command.insert_resource(MainMenuState::new(&params));
}

pub fn main_menu(
    mut egui_ctx: ResMut<EguiContext>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: Res<Params>,
    mut conf: ResMut<Conf>,
    mut ew_conf_change: EventWriter<ConfChange>,
    mut state: ResMut<MainMenuState>,
) {
    match state.current {
        MainMenuWindow::Menu => {
            egui::Window::new(t!("menu"))
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .default_width(0.0)
                .resizable(false)
                .show(egui_ctx.ctx_mut(), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        if ui.button(t!("new")).clicked() {
                            state.current = MainMenuWindow::NewPlanet;
                        }
                        if ui.button(t!("load")).clicked() {
                            ew_manage_planet.send(ManagePlanet::Load("test.planet".into()));
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
        MainMenuWindow::NewPlanet => {
            super::new_planet::new_planet(&mut egui_ctx, ew_manage_planet, &params, &mut state);
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
