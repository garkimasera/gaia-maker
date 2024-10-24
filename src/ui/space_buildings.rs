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
            for kind in OrbitalBuildingKind::iter() {
                ui.vertical(|ui| {
                    buildng_row(
                        ui,
                        kind,
                        &mut planet,
                        &params,
                        params.building_attrs(kind).unwrap(),
                    );
                });
                ui.separator();
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.ui.scale_factor));
}

pub fn star_system_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    conf: Res<Conf>,
    params: Res<Params>,
) {
    if !wos.star_system {
        return;
    }

    let rect = egui::Window::new(t!("star-system"))
        .open(&mut wos.star_system)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            for kind in StarSystemBuildingKind::iter() {
                ui.vertical(|ui| {
                    buildng_row(
                        ui,
                        kind,
                        &mut planet,
                        &params,
                        params.building_attrs(kind).unwrap(),
                    );
                });
                ui.separator();
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.ui.scale_factor));
}

pub fn buildng_row<T: Into<SpaceBuildingKind> + AsRef<str> + Copy>(
    ui: &mut egui::Ui,
    kind: T,
    planet: &mut Planet,
    params: &Params,
    attrs: &BuildingAttrs,
) {
    let buildable = planet.buildable(attrs, 1)
        && attrs
            .build_max
            .map(|build_max| build_max > planet.space_building(kind).n)
            .unwrap_or(true);
    let buildable10 = planet.buildable(attrs, 10)
        && attrs
            .build_max
            .map(|build_max| build_max >= planet.space_building(kind).n + 10)
            .unwrap_or(true);
    let building = planet.space_building_mut(kind);

    ui.horizontal(|ui| {
        let building_text = format!("{} x{}\t", t!(kind.as_ref()), building.n);
        ui.add(egui::Label::new(building_text).extend());
        let help_item: HelpItem = kind.into().into();
        ui.label("?").on_hover_ui(|ui| help_item.ui(ui, params));
    });

    ui.horizontal(|ui| {
        if ui.add_enabled(buildable, egui::Button::new("+1")).clicked() {
            planet.build_space_building(kind, params);
        }
        if ui
            .add_enabled(buildable10, egui::Button::new("+10"))
            .clicked()
        {
            for _ in 0..10 {
                planet.build_space_building(kind, params);
            }
        }
    });

    let building = planet.space_building_mut(kind);
    match &mut building.control {
        BuildingControlValue::AlwaysEnabled => {}
        BuildingControlValue::EnabledNumber(enabled) => {
            ui.add(egui::Slider::new(enabled, 0..=building.n));
        }
        BuildingControlValue::IncreaseRate(rate) => {
            ui.add(egui::Slider::new(rate, -100..=100));
        }
    }
}
