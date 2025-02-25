use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use geom::Coords;

use crate::{
    conf::Conf,
    planet::{Planet, TILE_SIZE},
    screen::{Centering, OccupiedScreenSpace},
};

use super::{Dialog, WindowsOpenState};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReportUi {
    head: String,
    body: String,
    pos: Option<Coords>,
}

pub fn report_list(ui: &mut egui::Ui, wos: &mut WindowsOpenState, planet: &Planet, conf: &Conf) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size * 1.2;

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto())
            .column(egui_extras::Column::auto())
            .min_scrolled_height(0.0);

        table.body(|mut body| {
            for report in planet.reports.iter().take(conf.ui.reports_in_list) {
                body.row(text_height, |mut row| {
                    row.col(|ui| {
                        let (style, text) = report.text();
                        ui.label(style.icon());
                        let (head, body) = crate::text::split_to_head_body(&text);
                        if let Some(body) = body {
                            if ui.link(head).clicked() {
                                let report_ui = ReportUi {
                                    head: head.to_owned(),
                                    body: body.to_owned(),
                                    pos: report.content.pos(),
                                };
                                if let Some((i, _)) = wos
                                    .dialogs
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, m)| {
                                        if let Dialog::Report(report_ui) = m {
                                            Some((i, report_ui))
                                        } else {
                                            None
                                        }
                                    })
                                    .find(|(_, m)| report_ui == **m)
                                {
                                    wos.dialogs.remove(i);
                                } else {
                                    wos.dialogs.push(Dialog::Report(report_ui));
                                }
                            }
                        } else {
                            ui.label(head);
                        }
                    });
                });
            }
        });
    });
}

pub fn report_windows(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_centering: EventWriter<Centering>,
) {
    let mut close_dialogs = Vec::new();
    for (i, dialog) in wos.dialogs.iter_mut().enumerate() {
        let mut open = true;
        let close = false;

        let dialog = match dialog {
            Dialog::Report(report_ui) => egui::Window::new(&report_ui.head)
                .open(&mut open)
                .default_height(240.0)
                .vscroll(true)
                .show(egui_ctxs.ctx_mut(), |ui| {
                    report_ui.ui(ui, &mut ew_centering);
                }),
            _ => unreachable!(),
        };
        let rect = dialog.unwrap().response.rect;
        occupied_screen_space.push_egui_window_rect(rect);
        if !open || close {
            close_dialogs.push(i);
        }
    }

    for &i in close_dialogs.iter().rev() {
        wos.dialogs.remove(i);
    }
}

impl ReportUi {
    fn ui(&self, ui: &mut egui::Ui, ew_centering: &mut EventWriter<Centering>) {
        ui.label(&self.body);
        if let Some(pos) = self.pos {
            if ui.button(t!("focus")).clicked() {
                ew_centering.send(Centering(Vec2::new(
                    pos.0 as f32 * TILE_SIZE,
                    pos.1 as f32 * TILE_SIZE,
                )));
            }
        }
    }
}
