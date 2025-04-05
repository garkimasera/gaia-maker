use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    conf::Conf,
    planet::{Planet, TILE_SIZE},
    screen::{Centering, OccupiedScreenSpace},
};

pub fn report_ui(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut ew_centering: EventWriter<Centering>,
    planet: Res<Planet>,
    conf: Res<Conf>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut open: Local<bool>,
) {
    const BUTTON_WIDTH: f32 = 32.0;
    let w = window.single().width() - BUTTON_WIDTH - occupied_screen_space.occupied_left - 10.0;
    occupied_screen_space.occupied_buttom = egui::TopBottomPanel::bottom("report_panel")
        .resizable(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                let s = if *open { "▼" } else { "▲" };
                if ui
                    .add_sized([BUTTON_WIDTH, 20.0], egui::Button::new(s))
                    .clicked()
                {
                    *open = !*open;
                }
                let n_items = if *open { 10 } else { 1 };
                ui.vertical(|ui| {
                    super::report::report_list(ui, &planet, &mut ew_centering, n_items, w);
                });
            });
        })
        .response
        .rect
        .height()
        * conf.ui.scale_factor;
}

pub fn report_list(
    ui: &mut egui::Ui,
    planet: &Planet,
    ew_centering: &mut EventWriter<Centering>,
    n_items: usize,
    width: f32,
) {
    let w = (width - 48.0).max(0.0);
    let wrap_mode = if n_items <= 1 {
        egui::TextWrapMode::Truncate
    } else {
        egui::TextWrapMode::Wrap
    };
    egui::Grid::new("report_ui_grid")
        .striped(true)
        .min_col_width(16.0)
        .show(ui, |ui| {
            for report in planet.reports.iter().take(n_items) {
                let (style, text) = report.text(planet);
                ui.label(style.icon());
                ui.vertical(|ui| {
                    ui.style_mut().wrap_mode = Some(wrap_mode);
                    ui.set_width(w);
                    if let Some(pos) = report.content.pos() {
                        if ui.link(text).clicked() {
                            ew_centering.send(Centering(Vec2::new(
                                pos.0 as f32 * TILE_SIZE,
                                pos.1 as f32 * TILE_SIZE,
                            )));
                        }
                    } else {
                        ui.label(text);
                    }
                });
                ui.end_row();
            }
        });
}
