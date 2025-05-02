use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    planet::{Planet, TILE_SIZE},
    screen::{Centering, OccupiedScreenSpace},
};

use super::WindowsOpenState;

const DEFAULT_WINDOW_WIDTH: f32 = 180.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 150.0;

pub fn reports_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_centering: EventWriter<Centering>,
    planet: Res<Planet>,
) {
    if !wos.reports {
        return;
    }

    let rect = egui::Window::new(t!("reports"))
        .open(&mut wos.reports)
        .vscroll(true)
        .collapsible(false)
        .default_width(DEFAULT_WINDOW_WIDTH)
        .default_height(DEFAULT_WINDOW_HEIGHT)
        .show(egui_ctxs.ctx_mut(), |ui| {
            report_list(ui, &planet, &mut ew_centering);
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

pub fn report_list(ui: &mut egui::Ui, planet: &Planet, ew_centering: &mut EventWriter<Centering>) {
    for report in planet.reports.iter() {
        let (style, text) = report.text();
        ui.horizontal(|ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            ui.label(style.icon());
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
    }
}
