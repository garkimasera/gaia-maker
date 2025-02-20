use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::{
    conf::Conf,
    planet::{Cost, Params, Planet},
    screen::{CursorMode, OccupiedScreenSpace},
};

use super::UiTextures;

pub fn hover_tile_tooltip(
    mut egui_ctxs: EguiContexts,
    (planet, textures, params, _conf): (Res<Planet>, Res<UiTextures>, Res<Params>, Res<Conf>),
    window: Query<&Window, With<PrimaryWindow>>,
    cursor_mode: Res<CursorMode>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
) {
    let window = window.get_single().unwrap();

    let cursor_pos = if let Some(pos) = window.cursor_position() {
        Vec2::new(pos.x, window.height() - pos.y)
    } else {
        return;
    };
    if !occupied_screen_space.check(window.width(), window.height(), cursor_pos) {
        return;
    }

    let cost_list = crate::action::cursor_mode_lack_and_cost(&planet, &params, &cursor_mode);
    if cost_list.iter().all(|(lack, _)| !lack) {
        return;
    }

    egui::show_tooltip_at_pointer(
        egui_ctxs.ctx_mut(),
        egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("hover_tile_tooltip_layer"),
        ),
        egui::Id::new("hover_tile_tooltip"),
        |ui| {
            ui.horizontal_centered(|ui| {
                ui.label(egui::RichText::new(t!("not-enough")).color(egui::Color32::RED));
                for (lack, cost) in cost_list {
                    if lack {
                        let image = match cost {
                            Cost::Power(_, _) => "ui/icon-power",
                            Cost::Material(_) => "ui/icon-material",
                            Cost::GenePoint(_) => "ui/icon-gene",
                        };
                        ui.image(textures.get(image));
                    }
                }
            });
        },
    );
}
