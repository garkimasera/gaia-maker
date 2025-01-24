use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::saveload::{load_save_file_list, SaveFileList, N_SAVE_FILES};
use crate::sim::{ManagePlanet, SaveFileMetadata};

static SAVE_FILE_LIST: Mutex<Option<SaveFileList>> = Mutex::new(None);

pub fn saveload_window(
    mut egui_ctxs: EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut save_file_metadata: ResMut<SaveFileMetadata>,
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
        Some(&mut save_file_metadata),
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
    save_file_metadata: Option<&mut SaveFileMetadata>,
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

    egui::Modal::new("saveload".into()).show(ctx, |ui| {
        let row_height = egui::TextStyle::Body.resolve(ui.style()).size * 1.2;
        let table = egui_extras::TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(egui_extras::Column::auto().at_least(35.0))
            .column(egui_extras::Column::auto().at_least(100.0))
            .column(egui_extras::Column::auto().at_least(145.0))
            .column(egui_extras::Column::auto().at_least(50.0))
            .min_scrolled_height(0.0);

        table
            .max_scroll_height(220.0)
            .header(row_height, |mut header| {
                header.col(|ui| {
                    ui.strong("#");
                });
                header.col(|ui| {
                    ui.strong(t!("planet-name"));
                });
                header.col(|ui| {
                    ui.strong(t!("date-saved"));
                });
                header.col(|_ui| {});
            })
            .body(|mut body| {
                for i in start..=N_SAVE_FILES {
                    let save_file_list = save_file_list.as_ref().unwrap();
                    let name = save_file_list.name(i);
                    let time = save_file_list.saved_time(i);
                    body.row(row_height, |mut row| {
                        row.col(|ui| {
                            if i == 0 {
                                ui.label("Auto");
                            } else {
                                ui.label(format!("{}", i));
                            }
                        });
                        row.col(|ui| {
                            if let Some(name) = name {
                                ui.label(name);
                            }
                        });
                        row.col(|ui| {
                            if let Some(time) = time {
                                let len = time.len();
                                ui.label(&time[0..(len - 3)]);
                            }
                        });
                        row.col(|ui| {
                            if (!load || time.is_some()) && ui.button(action_name.clone()).clicked()
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
    });

    if canceled {
        *open_state = false;
    }
    if let Some(selected) = selected {
        if load {
            ew_manage_planet.send(ManagePlanet::Load(selected));
        } else {
            if let Some(save_file_metadata) = save_file_metadata {
                save_file_metadata.manual_slot = Some(selected);
            }
            ew_manage_planet.send(ManagePlanet::Save(selected));
        }
        *open_state = false;
    }
    if !*open_state {
        *save_file_list = None;
    }
}
