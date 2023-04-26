use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::IntoEnumIterator;

use super::{convert_rect, help::HelpItem, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;

pub fn orbit_window(
    mut egui_ctxs: EguiContexts,
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
        .show(egui_ctxs.ctx_mut(), |ui| {
            egui::Grid::new("orbital_buildings")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("");
                    ui.label("");
                    ui.label(t!("enabled"));
                    ui.label("");
                    ui.end_row();
                    for kind in OrbitalBuildingKind::iter() {
                        let buildable = planet.buildable(&params.orbital_buildings[&kind], 1)
                            && params.orbital_buildings[&kind]
                                .build_max
                                .map(|build_max| build_max > planet.space_building(kind).n)
                                .unwrap_or(true);
                        let buildable10 = planet.buildable(&params.orbital_buildings[&kind], 10)
                            && params.orbital_buildings[&kind]
                                .build_max
                                .map(|build_max| build_max >= planet.space_building(kind).n + 10)
                                .unwrap_or(true);
                        let building = planet.space_building_mut(kind);
                        ui.label(t!(kind.as_ref()));
                        ui.label(format!("{}", building.n));
                        ui.add(egui::Slider::new(&mut building.enabled, 0..=building.n));
                        if ui.add_enabled(buildable, egui::Button::new("+1")).clicked() {
                            planet.build_space_building(kind, &params);
                        }
                        if ui
                            .add_enabled(buildable10, egui::Button::new("+10"))
                            .clicked()
                        {
                            for _ in 0..10 {
                                planet.build_space_building(kind, &params);
                            }
                        }
                        ui.label("?")
                            .on_hover_ui(|ui| HelpItem::OrbitalBuildings(kind).ui(ui, &params));
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
