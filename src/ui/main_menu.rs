use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::conf::{Conf, ConfChange};
use crate::manage_planet::{GlobalData, ManagePlanet, ManagePlanetError, SaveState};
use crate::planet::Params;
use crate::text_assets::Lang;
use crate::tutorial::TUTORIAL_PLANET;
use strum::IntoEnumIterator;

use super::new_planet::NewPlanetState;
use super::UiTextures;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MainMenuMode {
    #[default]
    Menu,
    Tutorial,
    NewPlanet,
    Load,
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
    save_state: Res<SaveState>,
    (mut conf, mut ew_conf_change): (ResMut<Conf>, EventWriter<ConfChange>),
    mut er_manage_planet_error: EventReader<ManagePlanetError>,
    mut app_exit_events: EventWriter<AppExit>,
    mut state: ResMut<MainMenuState>,
    mut logo_visibility: Query<&mut Visibility, With<crate::title_screen::TitleScreenLogo>>,
    mut window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    (textures, global_data): (Res<UiTextures>, Res<GlobalData>),
    random_name_list_map: Res<crate::text_assets::RandomNameListMap>,
) {
    if let Some(e) = er_manage_planet_error.read().next() {
        state.mode = MainMenuMode::Error;
        state.error = Some(e.clone());
    }
    let mut logo_visibility = logo_visibility.get_single_mut().unwrap();
    *logo_visibility = Visibility::Hidden;

    match state.mode {
        MainMenuMode::Menu => {
            *logo_visibility = Visibility::Visible;
            egui::Window::new(t!("menu"))
                .title_bar(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 127.0))
                .default_width(150.0)
                .resizable(false)
                .show(egui_ctxs.ctx_mut(), |ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        resume_ui(ui, &global_data, &mut ew_manage_planet);
                        if ui.button(t!("new")).clicked() {
                            state.mode = MainMenuMode::NewPlanet;
                        }
                        if ui.button(t!(TUTORIAL_PLANET)).clicked() {
                            state.mode = MainMenuMode::Tutorial;
                        }
                        if ui.button(t!("load")).clicked() {
                            state.mode = MainMenuMode::Load;
                        }
                        if ui.button(t!("exit")).clicked() {
                            app_exit_events.send(bevy::app::AppExit::Success);
                            crate::platform::window_close();
                        }

                        ui.separator();

                        if let Some(lang) = language_selector(ui, crate::text_assets::get_lang()) {
                            conf.lang = lang;
                            crate::text_assets::set_lang(lang);
                            ew_conf_change.send_default();
                            state.new_planet.name = t!("new-planet");
                        }
                    });
                })
                .unwrap();
        }
        MainMenuMode::Tutorial => {
            if let Some(cancelled) = super::saveload::check_save_limit(
                egui_ctxs.ctx_mut(),
                &mut ew_manage_planet,
                &save_state,
            ) {
                if cancelled {
                    state.mode = MainMenuMode::Menu;
                }
            } else {
                let mut start_params =
                    crate::planet::start_planet_to_start_params(TUTORIAL_PLANET, &params);
                start_params.basics.name = t!(TUTORIAL_PLANET);
                ew_manage_planet.send(ManagePlanet::New(start_params));
            }
        }
        MainMenuMode::NewPlanet => {
            super::new_planet::new_planet(
                &mut egui_ctxs,
                ew_manage_planet,
                &params,
                &mut state,
                &textures,
                &mut window.single_mut(),
                &random_name_list_map,
                &save_state,
            );
        }
        MainMenuMode::Load => {
            let mut open_state = true;
            super::saveload::show_load_window(
                egui_ctxs.ctx_mut(),
                &mut ew_manage_planet,
                &mut open_state,
                &save_state.current_save_sub_dir,
            );
            if !open_state {
                state.mode = MainMenuMode::Menu;
            }
        }
        MainMenuMode::Error => {
            egui::Window::new(t!("msg/loading-failed"))
                .collapsible(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(egui_ctxs.ctx_mut(), |ui| {
                    super::error_popup::ui_management_planet_error(
                        ui,
                        state.error.as_ref().unwrap(),
                    );
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

fn resume_ui(
    ui: &mut egui::Ui,
    global_data: &GlobalData,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
) {
    if let Some((save_sub_dir, auto, n)) = &global_data.latest_save_dir_file {
        if ui.button(t!("resume")).clicked() {
            ew_manage_planet.send(ManagePlanet::Load {
                sub_dir_name: save_sub_dir.clone(),
                auto: *auto,
                n: *n,
            });
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
