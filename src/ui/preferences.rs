use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::IntoEnumIterator;

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::conf::{Conf, ConfChange, HighLow3};

pub fn preferences_window(
    mut egui_ctxs: EguiContexts,
    mut wos: ResMut<WindowsOpenState>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut conf: ResMut<Conf>,
    mut ew_conf_change: EventWriter<ConfChange>,
) {
    if !wos.preferences {
        return;
    }
    let conf_before_change = conf.clone();

    let rect = egui::Window::new(t!("preferences"))
        .open(&mut wos.preferences)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.checkbox(
                &mut conf.autosave_enabled,
                t!("preference", "autosave-enabled"),
            );
            ui.horizontal(|ui| {
                ui.label(t!("preference", "screen-refresh-rate"));
                egui::ComboBox::from_id_salt("screen-refresh-rate")
                    .selected_text(t!(conf.screen_refresh_rate))
                    .show_ui(ui, |ui| {
                        for item in HighLow3::iter() {
                            ui.selectable_value(&mut conf.screen_refresh_rate, item, t!(item));
                        }
                    });
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);

    if *conf != conf_before_change {
        ew_conf_change.send_default();
    }
}
