use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    conf::Conf,
    planet::{Params, Planet, PlanetEvent},
    screen::OccupiedScreenSpace,
    sim::StartEvent,
};

use super::{convert_rect, Dialog, WindowsOpenState};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MsgDialog {
    head: String,
    body: String,
}

pub fn msg_list(ui: &mut egui::Ui, wos: &mut WindowsOpenState, planet: &Planet, conf: &Conf) {
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
            for msg in planet.msgs.iter().take(conf.ui.messages_in_list) {
                body.row(text_height, |mut row| {
                    row.col(|ui| {
                        let (style, text) = msg.text();
                        ui.label(style.icon());
                        let (head, body) = crate::text::split_to_head_body(&text);
                        if let Some(body) = body {
                            if ui.link(head).clicked() {
                                let msg_dialog = MsgDialog {
                                    head: head.to_owned(),
                                    body: body.to_owned(),
                                };
                                if let Some((i, _)) = wos
                                    .dialogs
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, m)| {
                                        if let Dialog::Msg(msg_dialog) = m {
                                            Some((i, msg_dialog))
                                        } else {
                                            None
                                        }
                                    })
                                    .find(|(_, m)| msg_dialog == **m)
                                {
                                    wos.dialogs.remove(i);
                                } else {
                                    wos.dialogs.push(Dialog::Msg(msg_dialog));
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

pub fn dialogs(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_start_event: EventWriter<StartEvent>,
    conf: Res<Conf>,
) {
    let mut close_dialogs = Vec::new();
    for (i, dialog) in wos.dialogs.iter_mut().enumerate() {
        let mut open = true;
        let mut close = false;

        let dialog = match dialog {
            Dialog::Msg(msg_dialog) => egui::Window::new(&msg_dialog.head)
                .open(&mut open)
                .vscroll(true)
                .show(egui_ctxs.ctx_mut(), |ui| {
                    ui.label(&msg_dialog.body);
                }),
            Dialog::Civilize(dialog) => egui::Window::new(t!("civilize-new"))
                .open(&mut open)
                .vscroll(true)
                .show(egui_ctxs.ctx_mut(), |ui| {
                    close = dialog.ui(ui, &mut ew_start_event);
                }),
        };
        let rect = dialog.unwrap().response.rect;
        occupied_screen_space
            .window_rects
            .push(convert_rect(rect, conf.ui.scale_factor));
        if !open || close {
            close_dialogs.push(i);
        }
    }

    for &i in close_dialogs.iter().rev() {
        wos.dialogs.remove(i);
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CivilizeDialog {}

impl CivilizeDialog {
    pub fn new(_params: &Params) -> Self {
        Self {}
    }

    fn ui(&mut self, ui: &mut egui::Ui, ew_start_event: &mut EventWriter<StartEvent>) -> bool {
        if ui.button(t!("start")).clicked() {
            ew_start_event.send(StartEvent(PlanetEvent::Civilize { target: 0 }));
            return true;
        }
        false
    }
}
