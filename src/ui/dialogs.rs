use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use super::{Dialog, OccupiedScreenSpace, WindowsOpenState};
use crate::{
    audio::SoundEffectPlayer,
    draw::UpdateDraw,
    manage_planet::ManagePlanetError,
    planet::{Params, Planet},
};

pub fn dialogs(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut update_draw: ResMut<UpdateDraw>,
    mut planet: ResMut<Planet>,
    params: Res<Params>,
    se_player: SoundEffectPlayer,
) {
    let Some(dialog) = wos.dialogs.last() else {
        return;
    };

    let mut close = false;

    let Dialog::Civilize { p, id } = dialog;

    occupied_screen_space.opening_modal = true;

    egui::Modal::new("civilize-animal".into()).show(egui_ctxs.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.label(t!("msg/civilize-animal"));
            ui.label(t!("animal", id));
            ui.separator();
            if ui.button(t!("ok")).clicked() {
                planet.civilize_animal(*p, *id, &params);
                close = true;
                update_draw.update();
                se_player.play_with_priority("civilize", true);
            }
            if ui.button(t!("cancel")).clicked() {
                close = true;
                se_player.play("window-close");
            }
        });
    });

    if close {
        wos.dialogs.pop();
    }
}

pub fn error_popup(
    mut egui_ctxs: EguiContexts,
    mut er_manage_planet_error: EventReader<ManagePlanetError>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
) {
    if wos.error_popup.is_none() {
        if let Some(e) = er_manage_planet_error.read().next() {
            wos.error_popup = Some(e.clone());
        }
    }
    let Some(e) = &wos.error_popup else {
        return;
    };

    occupied_screen_space.opening_modal = true;

    let mut close = false;
    egui::Modal::new("error_popup".into()).show(egui_ctxs.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui_management_planet_error(ui, e);
            ui.separator();
            if ui.button(t!("close")).clicked() {
                close = true;
            }
        });
    });

    if close {
        wos.error_popup = None;
    }
}

pub fn ui_management_planet_error(ui: &mut egui::Ui, e: &ManagePlanetError) {
    if matches!(e, ManagePlanetError::Decode) {
        ui.label(t!("msg/loading-failed-desc-decode-error"));
    } else {
        ui.label(t!("msg/loading-failed-desc-not-found"));
    }
}
