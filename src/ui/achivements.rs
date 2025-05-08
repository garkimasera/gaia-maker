use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use compact_str::format_compact;

use super::{UiTextures, WindowsOpenState};
use crate::{
    achivement_save::UnlockedAchivements, planet::ACHIVEMENTS, screen::OccupiedScreenSpace,
};

pub fn achivements_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    unlocked_achivements: Res<UnlockedAchivements>,
    textures: Res<UiTextures>,
) {
    if !wos.achivements {
        return;
    }

    let ctx = egui_ctxs.ctx_mut();
    let rect = egui::Window::new(t!("achivements"))
        .constrain_to(super::misc::constrain_to_rect(ctx, &occupied_screen_space))
        .resizable(egui::Vec2b::new(false, false))
        .open(&mut wos.achivements)
        .show(ctx, |ui| {
            show_achivement_list(ui, &unlocked_achivements, &textures);
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn show_achivement_list(
    ui: &mut egui::Ui,
    unlocked_achivements: &UnlockedAchivements,
    textures: &UiTextures,
) {
    let description_width = 220.0;

    let layout = egui::Layout::left_to_right(egui::Align::Min);
    for achivement_chunk in ACHIVEMENTS.chunks(6) {
        ui.with_layout(layout, |ui| {
            for achivement in achivement_chunk {
                let unlocked = unlocked_achivements.0.contains(achivement);
                let texture_name = if unlocked {
                    format_compact!("ui/achivement-{}", achivement.as_ref())
                } else {
                    "ui/achivement-locked".into()
                };
                ui.image(textures.get(texture_name)).on_hover_ui(|ui| {
                    ui.set_min_width(description_width);
                    if unlocked {
                        ui.strong(t!("achivement", achivement.as_ref()));
                    } else {
                        ui.strong("???");
                    }
                    ui.label(t!("achivement/desc", achivement.as_ref()));
                });
            }
        });
    }
}
