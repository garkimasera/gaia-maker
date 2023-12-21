use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{conf::Conf, planet::Planet, screen::OccupiedScreenSpace};

use super::{convert_rect, WindowsOpenState};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MsgDialog {
    head: String,
    body: String,
}

pub fn msg_list(ui: &mut egui::Ui, wos: &mut WindowsOpenState, planet: &Planet, _conf: &Conf) {
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
            for msg in planet.msgs.iter_temp().chain(planet.msgs.iter()) {
                body.row(text_height, |mut row| {
                    row.col(|ui| {
                        ui.label(msg.icon());

                        let text = msg.text();
                        let (head, body) = crate::text::split_to_head_body(&text);
                        if let Some(body) = body {
                            if ui.link(head).clicked() {
                                let msg_dialog = MsgDialog {
                                    head: head.to_owned(),
                                    body: body.to_owned(),
                                };
                                if let Some((i, _)) = wos
                                    .msg_dialogs
                                    .iter()
                                    .enumerate()
                                    .find(|(_, m)| msg_dialog == **m)
                                {
                                    wos.msg_dialogs.remove(i);
                                } else {
                                    wos.msg_dialogs.push(msg_dialog);
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

pub fn msg_dialogs(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
) {
    let mut close_dialogs = Vec::new();
    for (i, msg_dialog) in wos.msg_dialogs.iter().enumerate() {
        let mut open = true;
        let rect = egui::Window::new(&msg_dialog.head)
            .open(&mut open)
            .vscroll(true)
            .show(egui_ctxs.ctx_mut(), |ui| {
                ui.label(&msg_dialog.body);
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space
            .window_rects
            .push(convert_rect(rect, conf.scale_factor));
        if !open {
            close_dialogs.push(i);
        }
    }

    for &i in close_dialogs.iter().rev() {
        wos.msg_dialogs.remove(i);
    }
}
