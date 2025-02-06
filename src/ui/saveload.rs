use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::{
    saveload::SaveSubDirItem,
    sim::{ManagePlanet, SaveState},
};

#[derive(Default, Debug)]
struct WindowState {
    current_sub_dir: String,
    file_list: Vec<SaveSubDirItem>,
}

static WINDOW_STATE: std::sync::LazyLock<Mutex<WindowState>> =
    std::sync::LazyLock::new(Mutex::default);
static NEED_INIT: crossbeam::atomic::AtomicCell<bool> = crossbeam::atomic::AtomicCell::new(true);

pub fn load_window(
    mut egui_ctxs: EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    save_state: Res<SaveState>,
    mut wos: ResMut<WindowsOpenState>,
) {
    debug_assert!(!(wos.save && wos.load));

    if !wos.load {
        return;
    }

    let mut open_state = true;
    show_load_window(
        egui_ctxs.ctx_mut(),
        &mut ew_manage_planet,
        &mut open_state,
        &save_state,
    );

    if open_state {
        occupied_screen_space.opening_modal = true;
    } else {
        wos.load = false;
    }
}

pub fn show_load_window(
    ctx: &mut egui::Context,
    ew_manage_planet: &mut EventWriter<ManagePlanet>,
    open_state: &mut bool,
    save_state: &SaveState,
) {
    let mut ws = WINDOW_STATE.lock().unwrap();
    if NEED_INIT.load() {
        ws.current_sub_dir = save_state.current.clone();
        if !ws.current_sub_dir.is_empty() {
            ws.file_list = crate::saveload::read_save_sub_dir(&save_state.current);
        }
        NEED_INIT.store(false);
    }

    let mut load_selected = None;
    let mut canceled = false;

    egui::Modal::new("load".into()).show(ctx, |ui| {
        ui.horizontal(|ui| {
            egui::ScrollArea::vertical()
                .min_scrolled_height(300.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(150.0);
                        ui.set_min_height(300.0);
                        for (_, sub_dir) in &save_state.list {
                            if ui
                                .selectable_value(&mut ws.current_sub_dir, sub_dir.clone(), sub_dir)
                                .clicked()
                            {
                                ws.current_sub_dir = sub_dir.clone();
                                ws.file_list = crate::saveload::read_save_sub_dir(sub_dir);
                            }
                        }
                    });
                });
            ui.separator();
            ui.vertical(|ui| {
                let row_height = egui::TextStyle::Body.resolve(ui.style()).size * 1.2;
                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::auto().at_least(35.0))
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
                            ui.strong(t!("date-saved"));
                        });
                        header.col(|_ui| {});
                    })
                    .body(|mut body| {
                        for (i, item) in ws.file_list.iter().enumerate() {
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    if item.auto {
                                        ui.label("Auto");
                                    } else {
                                        ui.label("");
                                    }
                                });
                                row.col(|ui| {
                                    ui.label(item.time.to_string());
                                });
                                row.col(|ui| {
                                    if ui.button(t!("load")).clicked() {
                                        load_selected = Some(i);
                                    }
                                });
                            });
                        }
                    });
            });
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
    if let Some(selected) = load_selected {
        ew_manage_planet.send(ManagePlanet::Load {
            sub_dir_name: ws.current_sub_dir.clone(),
            auto: ws.file_list[selected].auto,
            n: ws.file_list[selected].n,
        });
        *open_state = false;
    }
    if !*open_state {
        NEED_INIT.store(true);
    }
}
