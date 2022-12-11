use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::planet::Params;
use crate::sim::ManagePlanet;

pub fn main_menu(
    mut egui_ctx: ResMut<EguiContext>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: Res<Params>,
) {
    egui::Window::new(t!("menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button(t!("new")).clicked() {
                    let size = params.start.default_size;
                    ew_manage_planet.send(ManagePlanet::New(size.0, size.1));
                }
                if ui.button(t!("load")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Load("test.planet".into()));
                }
            });
        })
        .unwrap();
}
