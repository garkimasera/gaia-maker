use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::EguiTextures;
use crate::tutorial::*;
use crate::{screen::OccupiedScreenSpace, sim::SaveState};

const WINDOW_WIDTH: f32 = 350.0;

pub fn tutorial_popup(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut save_state: ResMut<SaveState>,
    textures: Res<EguiTextures>,
) {
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };

    if let Some(popup_ui) = tutorial_state.popup_ui() {
        occupied_screen_space.opening_modal = true;

        egui::Modal::new("tutorial_popup".into()).show(egui_ctxs.ctx_mut(), |ui| {
            ui.set_max_width(WINDOW_WIDTH);
            popup_ui(ui, &textures);
            ui.separator();

            ui.vertical_centered(|ui| {
                if tutorial_state.has_next_popup_page() {
                    if ui.button(t!("next")).clicked() {
                        *tutorial_state = tutorial_state.next().unwrap();
                    }
                } else if ui.button(t!("ok")).clicked() {
                    *tutorial_state = tutorial_state.next().unwrap();
                }
            });
        });
    } else if let Some(tutorial_ui) = tutorial_state.tutorial_ui() {
        let next_by_manual = tutorial_state.next_by_manual();

        let rect = egui::Window::new(t!("tutorial"))
            .default_width(WINDOW_WIDTH)
            .show(egui_ctxs.ctx_mut(), |ui| {
                tutorial_ui(ui, &textures);
                ui.separator();

                if next_by_manual {
                    ui.vertical_centered(|ui| {
                        if ui.button(t!("next-tutorial")).clicked() {
                            *tutorial_state = tutorial_state.next().unwrap();
                        }
                    });
                }
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
    }
}

impl TutorialState {
    fn popup_ui(&self) -> Option<fn(&mut egui::Ui, &EguiTextures)> {
        match *self {
            Self::Start(0) => Some(|ui, _| {
                ui.label(t!("tutorial", "start-0"));
            }),
            Self::Start(1) => Some(start_last),
            _ => None,
        }
    }

    fn tutorial_ui(&self) -> Option<fn(&mut egui::Ui, &EguiTextures)> {
        match *self {
            Self::TryingMapMove => Some(start_last),
            _ => None,
        }
    }
}

fn start_last(ui: &mut egui::Ui, textures: &EguiTextures) {
    ui.label(t!("tutorial", "start-1-1"));
    ui.vertical_centered(|ui| {
        ui.image(textures.get("ui/tutorial-move-keys"));
    });
    ui.add_space(8.0);
    ui.label(t!("tutorial", "start-1-2"));
    ui.vertical_centered(|ui| {
        ui.image(textures.get("ui/icon-map"));
    });
}
