use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::IntoEnumIterator;

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::{
    audio::SoundEffectPlayer,
    conf::{Conf, ConfChange, HighLow3},
};

pub fn preferences_window(
    mut egui_ctxs: EguiContexts,
    mut wos: ResMut<WindowsOpenState>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut conf: ResMut<Conf>,
    mut ew_conf_change: EventWriter<ConfChange>,
    se_player: SoundEffectPlayer,
) {
    if !wos.preferences {
        return;
    }
    let conf_before_change = conf.clone();

    let rect = egui::Window::new(t!("preferences"))
        .open(&mut wos.preferences)
        .show(egui_ctxs.ctx_mut(), |ui| {
            if ui
                .checkbox(
                    &mut conf.autosave_enabled,
                    t!("preference", "autosave-enabled"),
                )
                .clicked()
            {
                se_player.play("select-item");
            }
            ui.horizontal(|ui| {
                ui.label(t!("preference", "screen-refresh-rate"));
                let response = egui::ComboBox::from_id_salt("screen-refresh-rate")
                    .selected_text(t!(conf.screen_refresh_rate))
                    .show_ui(ui, |ui| {
                        for item in HighLow3::iter() {
                            if ui
                                .selectable_value(&mut conf.screen_refresh_rate, item, t!(item))
                                .clicked()
                            {
                                se_player.play("select-item");
                            }
                        }
                    })
                    .response;
                if response.clicked() {
                    se_player.play("select-item");
                }
            });
            if ui
                .checkbox(&mut conf.show_fps, t!("preference", "show-fps"))
                .clicked()
            {
                se_player.play("select-item");
            }

            egui::Grid::new("volume_preferences").show(ui, |ui| {
                ui.label(t!("preference", "bgm-volume"));
                if ui
                    .add(egui::Slider::new(&mut conf.bgm_volume, 0..=100).suffix("%"))
                    .changed()
                {
                    se_player.play_if_stopped("slider");
                }
                ui.end_row();
                ui.label(t!("preference", "se-volume"));
                if ui
                    .add(egui::Slider::new(&mut conf.sound_effect_volume, 0..=100).suffix("%"))
                    .changed()
                {
                    se_player.play_if_stopped("slider");
                }
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
