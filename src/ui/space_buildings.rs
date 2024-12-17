use bevy::prelude::*;
use bevy_egui::{
    egui::{self, epaint, load::SizedTexture},
    EguiContexts,
};
use strum::IntoEnumIterator;

use super::{help::HelpItem, EguiTextures, OccupiedScreenSpace, WindowsOpenState};
use crate::planet::*;

const BUILDING_BACKGROUND_SIZE: (u32, u32) = (336, 48);

pub fn space_buildings_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    mut sim: ResMut<Sim>,
    textures: Res<EguiTextures>,
    params: Res<Params>,
) {
    if !wos.space_building {
        return;
    }

    let rect = egui::Window::new(t!("space-buildings"))
        .open(&mut wos.space_building)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            for (i, kind) in SpaceBuildingKind::iter().enumerate() {
                if i != 0 {
                    ui.separator();
                }
                ui.vertical(|ui| {
                    buildng_row(
                        ui,
                        kind,
                        &mut planet,
                        &mut sim,
                        &textures,
                        &params,
                        params.building_attrs(kind),
                    );
                });
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

pub fn buildng_row(
    ui: &mut egui::Ui,
    kind: SpaceBuildingKind,
    planet: &mut Planet,
    sim: &mut Sim,
    textures: &EguiTextures,
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
    });

    let response = ui.add(BuildingImage::new(kind, building.n, textures));
    if response.hovered() {
        let help_item = HelpItem::SpaceBuildings(kind);
        egui::containers::show_tooltip_at(
            &response.ctx,
            response.layer_id,
            response.id,
            response.rect.right_top(),
            |ui| help_item.ui(ui, textures, params),
        );
    }

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

struct BuildingImage {
    background: SizedTexture,
    background_star: Option<SizedTexture>,
    _n: u32,
}

impl BuildingImage {
    fn new(kind: SpaceBuildingKind, n: u32, textures: &EguiTextures) -> Self {
        let background_star = match kind {
            SpaceBuildingKind::DysonSwarmUnit => Some(textures.get("ui/background-building-star")),
            _ => None,
        };
        Self {
            background: textures.get("ui/background-building-space"),
            background_star,
            _n: n,
        }
    }
}

impl egui::Widget for BuildingImage {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let size = egui::Vec2::new(
            BUILDING_BACKGROUND_SIZE.0 as _,
            BUILDING_BACKGROUND_SIZE.1 as _,
        );
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();
            painter.add(epaint::RectShape {
                rect,
                rounding: egui::Rounding::ZERO,
                fill: egui::Color32::WHITE,
                stroke: egui::Stroke::NONE,
                blur_width: 0.0,
                fill_texture_id: self.background.id,
                uv,
            });

            if let Some(background_star) = &self.background_star {
                painter.add(epaint::RectShape {
                    rect: egui::Rect::from_min_size(rect.min, background_star.size),
                    rounding: egui::Rounding::ZERO,
                    fill: egui::Color32::WHITE,
                    stroke: egui::Stroke::NONE,
                    blur_width: 0.0,
                    fill_texture_id: background_star.id,
                    uv,
                });
            }
        }
        response
    }
}
