use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::IntoEnumIterator;

use super::{convert_rect, help::HelpItem, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;

pub fn space_buildings_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    mut sim: ResMut<Sim>,
    conf: Res<Conf>,
    params: Res<Params>,
) {
    if !wos.space_building {
        return;
    }

    let rect = egui::Window::new(t!("space-buildings"))
        .open(&mut wos.space_building)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            for kind in SpaceBuildingKind::iter() {
                ui.vertical(|ui| {
                    buildng_row(
                        ui,
                        kind,
                        &mut planet,
                        &mut sim,
                        &params,
                        params.building_attrs(kind),
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

pub fn buildng_row(
    ui: &mut egui::Ui,
    kind: SpaceBuildingKind,
    planet: &mut Planet,
    sim: &mut Sim,
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
        let help_item = HelpItem::SpaceBuildings(kind);
        ui.label("?").on_hover_ui(|ui| help_item.ui(ui, params));
    });

    ui.horizontal(|ui| {
        if ui.add_enabled(buildable, egui::Button::new("+1")).clicked() {
            planet.build_space_building(kind, sim, params);
        }
        if ui
            .add_enabled(buildable10, egui::Button::new("+10"))
            .clicked()
        {
            for _ in 0..10 {
                planet.build_space_building(kind, sim, params);
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
