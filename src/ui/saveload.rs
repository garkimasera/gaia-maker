use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::saveload::{load_save_file_list, SaveFileList, N_SAVE_FILES};
use crate::sim::{ManagePlanet, SaveSlot};

static SAVE_FILE_LIST: Mutex<Option<SaveFileList>> = Mutex::new(None);

pub fn saveload_window(
    mut egui_ctxs: EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut save_slot: ResMut<SaveSlot>,
    mut wos: ResMut<WindowsOpenState>,
) {
    debug_assert!(!(wos.save && wos.load));
    if !wos.save && !wos.load {
        return;
    }

    let mut open_state = true;
    show_saveload_window(
        egui_ctxs.ctx_mut(),
        &mut ew_manage_planet,
        &mut open_state,
        Some(&mut save_slot),
        wos.load,
    );

    if open_state {
        occupied_screen_space.opening_modal = true;
    } else {
        wos.save = false;
        wos.load = false;
    }
}

pub fn show_saveload_window(
    ctx: &mut egui::Context,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    open_state: &mut bool,
    save_slot: Option<&mut SaveSlot>,
    load: bool,
) {
    let (start, action_name) = if load {
        (0, t!("load"))
    } else {
        (1, t!("save"))
    };
    let mut selected = None;
    let mut canceled = false;
    let mut save_file_list = SAVE_FILE_LIST.lock().unwrap();
    if save_file_list.is_none() {
        *save_file_list = Some(load_save_file_list());
    }

    egui::Window::new(action_name.clone())
        .open(open_state)
        .vscroll(false)
        .collapsible(false)
        .resizable([false, false])
        .default_width(100.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            let row_height = egui::TextStyle::Body.resolve(ui.style()).size * 1.2;
            let table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(egui_extras::Column::auto().at_least(35.0))
                .column(egui_extras::Column::auto().at_least(180.0))
                .column(egui_extras::Column::auto().at_least(50.0))
                .min_scrolled_height(0.0);

            table
                .max_scroll_height(220.0)
                .header(row_height, |mut header| {
                    header.col(|ui| {
                        ui.strong("#");
                    });
                    header.col(|ui| {
                        ui.strong(t!("date"));
                    });
                    header.col(|_ui| {});
                })
                .body(|mut body| {
                    for i in start..=N_SAVE_FILES {
                        let time = save_file_list.as_ref().unwrap().saved_time(i);
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                if i == 0 {
                                    ui.label("Auto");
                                } else {
                                    ui.label(format!("{}", i));
                                }
                            });
                            row.col(|ui| {
                                if let Some(time) = time {
                                    ui.label(time);
                                }
                            });
                            row.col(|ui| {
                                if (!load || time.is_some())
                                    && ui.button(action_name.clone()).clicked()
                                {
                                    selected = Some(i);
                                }
                            });
                        });
                    }
                });
            ui.add(egui::Separator::default().spacing(2.0).horizontal());
            ui.vertical_centered(|ui| {
                if ui.button(t!("cancel")).clicked() {
                    canceled = true;
                }
            });
        })
        .unwrap();

    if canceled {
        *open_state = false;
    }
    if let Some(selected) = selected {
        if load {
            ew_manage_planet.send(ManagePlanet::Load(selected));
        } else {
            if let Some(save_slot) = save_slot {
                save_slot.0 = Some(selected);
            }
            ew_manage_planet.send(ManagePlanet::Save(selected));
        }
        *open_state = false;
    }
    if !*open_state {
        *save_file_list = None;
    }
}
