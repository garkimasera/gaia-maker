use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use super::{convert_rect, OccupiedScreenSpace, UiConf, WindowsOpenState};
use crate::planet::Planet;

pub fn orbit_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<UiConf>,
    _planet: Res<Planet>,
) {
    if !wos.orbit {
        return;
    }

    let rect = egui::Window::new(t!("orbit"))
        .open(&mut wos.orbit)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .always_show_scroll(true)
                .show(ui, |ui| ui.label("orbit"));
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}
