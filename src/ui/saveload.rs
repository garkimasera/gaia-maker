use std::sync::Mutex;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::{
    saveload::{SaveSubDirItem, SavedTime},
    sim::{ManagePlanet, SaveState},
};

#[derive(Default, Debug)]
struct WindowState {
    planet_name: String,
    current_sub_dir: String,
    dirs: Vec<(SavedTime, String)>,
    file_list: Vec<SaveSubDirItem>,
    delete: bool,
    delete_modal: Option<DeleteModal>,
}

#[derive(Clone, Default, Debug)]
struct DeleteModal {
    sub_dir_name: String,
    all: bool,
    auto: bool,
    n: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NeedInit {
    None,
    Files,
    All,
}

static WINDOW_STATE: std::sync::LazyLock<Mutex<WindowState>> =
    std::sync::LazyLock::new(Mutex::default);
static NEED_INIT: crossbeam::atomic::AtomicCell<NeedInit> =
    crossbeam::atomic::AtomicCell::new(NeedInit::All);

pub fn load_window(
    mut egui_ctxs: EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    save_state: Res<SaveState>,
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
        &save_state.current,
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
    playing_name: &str,
) {
    let mut ws_guard = WINDOW_STATE.lock().unwrap();
    let ws: &mut WindowState = &mut ws_guard;

    if let Some(delete_modal) = &ws.delete_modal {
        if delete_modal.show(ctx, ew_manage_planet) {
            ws.delete_modal = None;
            return;
        }
    }

    if NEED_INIT.load() != NeedInit::None {
        if NEED_INIT.load() == NeedInit::All {
            ws.current_sub_dir = playing_name.to_owned();
            ws.dirs = crate::platform::save_sub_dirs()
                .map_err(|e| {
                    log::warn!("{:?}", e);
                })
                .unwrap_or_default();
        }
        if ws.current_sub_dir.is_empty() {
            ws.planet_name.clear();
            ws.file_list.clear();
        } else {
            let (list, planet_name) = crate::saveload::read_save_sub_dir(&ws.current_sub_dir);
            ws.planet_name = planet_name;
            ws.file_list = list;
        }
        NEED_INIT.store(NeedInit::None);
    }

    let mut selected = None;
    let mut latest_selected = false;
    let mut canceled = false;
    let load_or_delete = if ws.delete { t!("delete") } else { t!("load") };

    egui::Modal::new("load".into()).show(ctx, |ui| {
        ui.horizontal(|ui| {
            egui::ScrollArea::vertical()
                .min_scrolled_height(300.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(150.0);
                        ui.set_min_height(300.0);
                        for (_, sub_dir) in &ws.dirs {
                            if ui
                                .selectable_value(&mut ws.current_sub_dir, sub_dir.clone(), sub_dir)
                                .clicked()
                            {
                                ws.current_sub_dir = sub_dir.clone();
                                let (list, planet_name) =
                                    crate::saveload::read_save_sub_dir(sub_dir);
                                ws.planet_name = planet_name;
                                ws.file_list = list;
                            }
                        }
                    });
                });
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(290.0);
                if ws.current_sub_dir.is_empty() {
                    return;
                }

                ui.heading(&ws.planet_name);
                ui.add_space(2.0);
                let enabled = if ws.delete {
                    ws.current_sub_dir != playing_name
                } else {
                    true
                };
                if ui
                    .add_enabled(enabled, egui::Button::new(load_or_delete.clone()))
                    .clicked()
                {
                    latest_selected = true;
                }
                ui.add_space(2.0);

                let row_height = egui::TextStyle::Body.resolve(ui.style()).size * 1.2;
                let table = egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(false)
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(egui_extras::Column::auto().at_least(39.0))
                    .column(egui_extras::Column::auto().at_least(145.0))
                    .column(egui_extras::Column::auto().at_least(64.0))
                    .min_scrolled_height(0.0);

                table
                    .max_scroll_height(220.0)
                    .header(row_height, |mut header| {
                        header.col(|_ui| {});
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
                                    let enabled = if ws.delete {
                                        ws.current_sub_dir != playing_name || ws.file_list.len() > 1
                                    } else {
                                        true
                                    };
                                    if ui
                                        .add_enabled(
                                            enabled,
                                            egui::Button::new(load_or_delete.clone()),
                                        )
                                        .clicked()
                                    {
                                        selected = Some(i);
                                    }
                                });
                            });
                        }
                    });
            });
        });
        ui.add(egui::Separator::default().spacing(2.0).horizontal());
        ui.vertical_centered(|ui| {
            let response = ui.button(t!("cancel"));
            if response.clicked() {
                canceled = true;
            }
            let rect =
                egui::Rect::from_min_size(response.rect.right_top(), egui::Vec2::new(100.0, 10.0));
            ui.put(rect, egui::Checkbox::new(&mut ws.delete, t!("delete")));
        });
    });

    if canceled {
        *open_state = false;
    }
    if let Some(selected) = selected {
        if ws.delete {
            ws.delete_modal = Some(DeleteModal {
                sub_dir_name: ws.current_sub_dir.clone(),
                auto: ws.file_list[selected].auto,
                n: ws.file_list[selected].n,
                all: ws.file_list.len() == 1,
            });
        } else {
            ew_manage_planet.send(ManagePlanet::Load {
                sub_dir_name: ws.current_sub_dir.clone(),
                auto: ws.file_list[selected].auto,
                n: ws.file_list[selected].n,
            });
            *open_state = false;
        }
    }
    if latest_selected {
        if ws.delete {
            ws.delete_modal = Some(DeleteModal {
                sub_dir_name: ws.current_sub_dir.clone(),
                all: true,
                ..Default::default()
            });
        } else {
            ew_manage_planet.send(ManagePlanet::Load {
                sub_dir_name: ws.current_sub_dir.clone(),
                auto: ws.file_list[0].auto,
                n: ws.file_list[0].n,
            });
            *open_state = false;
        }
    }
    if !*open_state {
        ws.delete = false;
        NEED_INIT.store(NeedInit::All);
    }
}

impl DeleteModal {
    /// Return true if closed
    fn show(
        &self,
        ctx: &mut egui::Context,
        ew_manage_planet: &mut EventWriter<ManagePlanet>,
    ) -> bool {
        let mut close = false;

        egui::Modal::new("delete-save".into()).show(ctx, |ui| {
            ui.label("really delete?");
            ui.horizontal(|ui| {
                if ui.button(t!("delete")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Delete {
                        sub_dir_name: self.sub_dir_name.clone(),
                        all: self.all,
                        auto: self.auto,
                        n: self.n,
                    });
                    close = true;

                    NEED_INIT.store(if self.all {
                        NeedInit::All
                    } else {
                        NeedInit::Files
                    });
                }
                if ui.button(t!("cancel")).clicked() {
                    close = true;
                }
            });
        });

        close
    }
}
