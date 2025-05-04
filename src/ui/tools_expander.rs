use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::screen::OccupiedScreenSpace;

use super::{UiTextures, WindowsOpenState};

pub fn tools_expander(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    textures: Res<UiTextures>,
) {
    let ctx = egui_ctxs.ctx_mut();
    let rect = egui::Window::new("tools-expander")
        .anchor(
            egui::Align2::LEFT_TOP,
            [0.0, occupied_screen_space.toolbar_height],
        )
        .frame(super::misc::small_window_frame(ctx))
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            if ui
                .add(
                    egui::ImageButton::new(textures.get("ui/icon-space-buildings"))
                        .selected(wos.space_building),
                )
                .on_hover_text(t!("space-buildings"))
                .clicked()
            {
                wos.space_building = !wos.space_building;
                wos.animals = false;
                wos.control = false;
            }
            if ui
                .add(egui::ImageButton::new(textures.get("ui/icon-animal")).selected(wos.animals))
                .on_hover_text(t!("animals"))
                .clicked()
            {
                wos.animals = !wos.animals;
                wos.space_building = false;
                wos.control = false;
            }
            if ui
                .add(egui::ImageButton::new(textures.get("ui/icon-control")).selected(wos.control))
                .on_hover_text(t!("control"))
                .clicked()
            {
                wos.control = !wos.control;
                wos.space_building = false;
                wos.animals = false;
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.tools_expander_width = rect.width();
    occupied_screen_space.push_egui_window_rect(rect);
}
