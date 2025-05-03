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
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
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
        .default_pos([
            window.single().width()
                - WINDOW_WIDTH
                - super::indicators::TILE_INFO_INDICATOR_WIDTH
                - 30.0,
            35.0,
        ])
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
                super::indicators::power_indicator(ui, textures, 30.0, 2.0);
                ui.add_space(8.0);
                ui.label(t!("tutorial", "power-0-2"));
                super::indicators::material_indicator(ui, textures, 100.0, 20.0);
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
                ui.add_space(8.0);
                ui.label(t!("tutorial", "fertilize-1-3"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/tutorial-soil-fertilize-example"));
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
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/tutorial-oxygen-generator-example"));
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
                    ui.add_space(8.0);
                    ui.image(textures.get("ui/tutorial-carbon-capturer-example"));
                });
                ui.add_space(8.0);
                ui.label(t!("tutorial", "carbon-0-2"));
            },
            Self::Animal(0) => |ui, textures| {
                ui.label(t!("tutorial", "animal-0-1"));
                super::indicators::gene_point_indicator(ui, textures, 100.0, 0.1);
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
                ui.add_space(8.0);
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/tutorial-animal-habitat"));
                });
            },
            Self::Civilize(0) => |ui, _textures| {
                ui.label(t!("tutorial", "civilize-0-1"));
            },
            Self::OrbitalMirror(0) => |ui, textures| {
                ui.label(t!("tutorial", "orbital-mirror-0-1"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-space-buildings"));
                });
                ui.label(t!("tutorial", "orbital-mirror-0-2"));
                ui.vertical_centered(|ui| {
                    ui.image(textures.get("ui/icon-stat"));
                });
            },
            Self::Complete(_) => |ui, _textures| {
                ui.label(t!("tutorial", "complete-0-1"));
            },
            _ => unreachable!(),
        }
    }
}
