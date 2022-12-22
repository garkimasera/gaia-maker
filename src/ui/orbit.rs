use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use strum::IntoEnumIterator;

use super::{building_desc_tooltip, convert_rect, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;

pub fn orbit_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    conf: Res<Conf>,
    params: Res<Params>,
) {
    if !wos.orbit {
        return;
    }

    let rect = egui::Window::new(t!("orbit"))
        .open(&mut wos.orbit)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            egui::Grid::new("orbital_buildings")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("");
                    ui.label("");
                    ui.label(t!("enabled"));
                    ui.label("");
                    ui.end_row();
                    for kind in OrbitalBuildingKind::iter() {
                        let buildable = planet.buildable(&params.orbital_buildings[&kind]);
                        let building = planet.orbit.get_mut(&kind).unwrap();
                        ui.label(t!(kind.as_ref()));
                        ui.label(format!("{}", building.n));
                        ui.add(egui::Slider::new(&mut building.enabled, 0..=building.n));
                        if ui
                            .add_enabled(buildable, egui::Button::new(t!("add")))
                            .on_hover_ui(building_desc_tooltip(&params.orbital_buildings[&kind]))
                            .on_disabled_hover_ui(building_desc_tooltip(
                                &params.orbital_buildings[&kind],
                            ))
                            .clicked()
                        {
                            planet.build_orbital_building(kind, &params);
                        }
                        ui.end_row();
                    }
                });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}
