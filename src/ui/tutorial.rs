use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::UiTextures;
use crate::tutorial::*;
use crate::{manage_planet::SaveState, screen::OccupiedScreenSpace};

const WINDOW_WIDTH: f32 = 350.0;

pub fn tutorial_popup(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut save_state: ResMut<SaveState>,
    textures: Res<UiTextures>,
) {
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };

    let has_next_tutorial = tutorial_state.current_step().has_next_tutorial();
    let can_back = tutorial_state.current_step().can_back();
    let can_complete = tutorial_state.current_step().can_complete();
    let tutorial_ui = tutorial_state.current_step().ui();
    let checked = tutorial_state.checked();

    let rect = egui::Window::new(t!("tutorial"))
        .default_width(WINDOW_WIDTH)
        .show(egui_ctxs.ctx_mut(), |ui| {
            tutorial_ui(ui, &textures);

            if has_next_tutorial && !tutorial_state.checklist().is_empty() {
                ui.add_space(8.0);
                egui::Grid::new("tutorial_checklist")
                    .num_columns(2)
                    .min_col_width(24.0)
                    .max_col_width(300.0)
                    .show(ui, |ui| {
                        for (item, checked) in tutorial_state.checklist() {
                            let texture = if *checked {
                                textures.get("ui/icon-check")
                            } else {
                                textures.get("ui/icon-cross")
                            };

                            ui.image(texture);
                            ui.label(t!("tutorial", item));
                            ui.end_row();
                        }
                    });
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                if can_complete {
                    if ui.button(t!("close")).clicked() {
                        tutorial_state.complete();
                    }
                } else {
                    if can_back && ui.button(t!("back")).clicked() {
                        tutorial_state.move_back();
                    }
                    if has_next_tutorial {
                        if ui
                            .add_enabled(checked, egui::Button::new(t!("next-tutorial")))
                            .clicked()
                        {
                            tutorial_state.move_next();
                        }
                    } else if ui.button(t!("next")).clicked() {
                        tutorial_state.move_next();
                    }
                }
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

impl TutorialStep {
    fn ui(&self) -> fn(&mut egui::Ui, &UiTextures) {
        match *self {
            Self::Start(0) => |ui, _| {
                ui.label(t!("tutorial", "start-0"));
            },
            Self::Start(1) => |ui, textures| {
                ui.label(t!("tutorial", "start-1-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/tutorial-move-keys"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "start-1-2"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-map"));
                });
            },
            Self::Power(0) => |ui, textures| {
                ui.label(t!("tutorial", "power-0-1"));
                super::misc::power_indicator(ui, textures, 30.0, 2.0);
                ui.add_space(8.0);
                ui.label(t!("tutorial", "power-0-2"));
                super::misc::material_indicator(ui, textures, 100.0, 20.0);
            },
            Self::Power(1) => |ui, textures| {
                ui.label(t!("tutorial", "power-1-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-space-buildings"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "power-1-2"));
            },
            Self::Fertilize(0) => |ui, textures| {
                ui.label(t!("tutorial", "fertilize-0-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-air-temperature"));
                    ui.image(textures.get("ui/icon-rainfall"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "fertilize-0-2"));
            },
            Self::Fertilize(1) => |ui, textures| {
                ui.label(t!("tutorial", "fertilize-1-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-build"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "fertilize-1-2"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-speed-medium"));
                });
            },
            Self::BuildOxygen(0) => |ui, textures| {
                ui.label(t!("tutorial", "build-oxygen-0-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-stat"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "build-oxygen-0-2"));
            },
            Self::BuildOxygen(1) => |ui, textures| {
                ui.label(t!("tutorial", "build-oxygen-1-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-space-buildings"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "build-oxygen-1-2"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-build"));
                });
            },
            Self::WaitOxygen(0) => |ui, textures| {
                ui.label(t!("tutorial", "wait-oxygen-0-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-speed-fast"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "wait-oxygen-0-2"));
            },
            Self::Carbon(0) => |ui, textures| {
                ui.label(t!("tutorial", "carbon-0-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-build"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "carbon-0-2"));
            },
            Self::Animal(0) => |ui, textures| {
                ui.label(t!("tutorial", "animal-0-1"));
                super::misc::gene_point_indicator(ui, textures, 100.0, 0.1);
                ui.add_space(8.0);
                ui.label(t!("tutorial", "animal-0-2"));
            },
            Self::Animal(1) => |ui, textures| {
                ui.label(t!("tutorial", "animal-1-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-animal"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "animal-1-2"));
            },
            Self::Civilize(0) => |ui, _textures| {
                ui.label(t!("tutorial", "civilize-0-1"));
            },
            Self::Complete(_) => |ui, _textures| {
                ui.label(t!("tutorial", "complete-0-1"));
            },
            _ => unreachable!(),
        }
    }
}
