use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{
    audio::SoundEffectPlayer,
    manage_planet::SwitchPlanet,
    planet::{Planet, TILE_SIZE},
    screen::{Centering, OccupiedScreenSpace},
};

use super::{UiTextures, WindowsOpenState};

const DEFAULT_WINDOW_WIDTH: f32 = 220.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 105.0;

pub fn reports_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_centering: EventWriter<Centering>,
    textures: Res<UiTextures>,
    planet: Res<Planet>,
    se_player: SoundEffectPlayer,
    mut er_switch_planet: EventReader<SwitchPlanet>,
    mut n_items_prev: Local<Option<usize>>,
) {
    if er_switch_planet.read().last().is_some() {
        *n_items_prev = None;
    }

    let n_items = planet.reports.n_reports();
    if let Some(n_items_prev) = &mut *n_items_prev {
        if *n_items_prev < n_items {
            se_player.play("report");
            *n_items_prev = n_items;
        }
    } else {
        *n_items_prev = Some(n_items);
    }

    if !wos.reports {
        let rect = egui::Window::new("reports-expander")
            .anchor(
                egui::Align2::LEFT_BOTTOM,
                [occupied_screen_space.stat_width, 0.0],
            )
            .frame(super::misc::small_window_frame(egui_ctxs.ctx_mut()))
            .resizable(false)
            .title_bar(false)
            .show(egui_ctxs.ctx_mut(), |ui| {
                if ui
                    .add(egui::ImageButton::new(textures.get("ui/icon-reports")))
                    .on_hover_text(t!("reports"))
                    .clicked()
                {
                    wos.reports = true;
                }
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
        return;
    }

    let rect = egui::Window::new("reports-window")
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            [occupied_screen_space.stat_width, 0.0],
        )
        .title_bar(false)
        .default_width(DEFAULT_WINDOW_WIDTH)
        .default_height(DEFAULT_WINDOW_HEIGHT)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("▼").clicked() {
                    wos.reports = false;
                }
                ui.heading(t!("reports"));
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(egui::Vec2b::new(false, false))
                .show(ui, |ui| {
                    report_list(ui, &planet, &mut ew_centering);
                });
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
                    ew_centering.send(Centering::new(Vec2::new(
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
