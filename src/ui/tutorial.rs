use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::UiTextures;
use crate::tutorial::*;
use crate::{screen::OccupiedScreenSpace, sim::SaveState};

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
    let tutorial_ui = tutorial_state.current_step().ui();
    let checked = tutorial_state.checked();

    let rect = egui::Window::new(t!("tutorial"))
        .default_width(WINDOW_WIDTH)
        .show(egui_ctxs.ctx_mut(), |ui| {
            tutorial_ui(ui, &textures);
            ui.separator();

            ui.vertical_centered(|ui| {
                if can_back && ui.button(t!("back")).clicked() {
                    tutorial_state.move_back();
                }
                let s = if has_next_tutorial {
                    t!("next-tutorial")
                } else {
                    t!("next")
                };
                if ui.add_enabled(checked, egui::Button::new(s)).clicked() {
                    tutorial_state.move_next();
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
            _ => unreachable!(),
        }
    }
}
