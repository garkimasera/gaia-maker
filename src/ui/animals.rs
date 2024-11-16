use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::planet::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn animals_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut _planet: ResMut<Planet>,
    _params: Res<Params>,
) {
    if !wos.animals {
        return;
    }

    let rect = egui::Window::new(t!("animals"))
        .open(&mut wos.animals)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |_ui| {})
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}
