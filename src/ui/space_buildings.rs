use bevy::prelude::*;
use bevy_egui::{
    egui::{self, epaint, load::SizedTexture},
    EguiContexts,
};
use compact_str::format_compact;
use strum::IntoEnumIterator;

use super::{help::HelpItem, OccupiedScreenSpace, UiTextures, WindowsOpenState};
use crate::planet::*;

const BUILDING_BACKGROUND_SIZE: (u32, u32) = (336, 48);

pub fn space_buildings_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    mut sim: ResMut<Sim>,
    textures: Res<UiTextures>,
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
    textures: &UiTextures,
    params: &Params,
    attrs: &BuildingAttrs,
) {
    let build_max = attrs.build_max.unwrap();
    let buildable = planet.buildable(attrs) && build_max > planet.space_building(kind).n;
    let building = planet.space_building_mut(kind);
    let n = building.n;

    ui.horizontal(|ui| {
        let building_text = format!("{} ({}/{})", t!(kind), building.n, build_max);
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
        if build_max >= 5 && ui.add_enabled(buildable, egui::Button::new("+5")).clicked() {
            for _ in 0..5 {
                if planet.buildable(attrs) && build_max > planet.space_building(kind).n {
                    planet.build_space_building(kind, sim, params);
                } else {
                    break;
                }
            }
        }

        if attrs.power < 0.0 {
            if n > 0 {
                if ui.button("-1").clicked() {
                    planet.demolish_space_building(kind, 1, sim, params);
                }
                if build_max >= 5 && ui.button("-5").clicked() {
                    planet.demolish_space_building(kind, 5, sim, params);
                }
            } else {
                ui.add_visible(false, egui::Button::new("-1"));
                if build_max >= 5 {
                    ui.add_visible(false, egui::Button::new("-5"));
                }
            }
        }
    });

    let building = planet.space_building_mut(kind);
    match &mut building.control {
        BuildingControlValue::AlwaysEnabled => {}
        BuildingControlValue::IncreaseRate(rate) => {
            ui.add(egui::Slider::new(rate, -100..=100));
        }
    }
}

struct BuildingImage {
    building: SizedTexture,
    background: SizedTexture,
    background_star: Option<SizedTexture>,
    left_padding: f32,
    n: u32,
}

impl BuildingImage {
    fn new(kind: SpaceBuildingKind, n: u32, textures: &UiTextures) -> Self {
        let (background_star, left_padding) = match kind {
            SpaceBuildingKind::DysonSwarmUnit => {
                let t = textures.get("ui/background-building-star");
                let left_padding = t.size.x;
                (Some(t), left_padding)
            }
            SpaceBuildingKind::OrbitalMirror => {
                (Some(textures.get("ui/background-building-planet")), 0.0)
            }
            SpaceBuildingKind::IonIrradiator => {
                let t = textures.get("ui/background-building-planet-left");
                let left_padding = t.size.x;
                (Some(t), left_padding)
            }
            _ => (None, 0.0),
        };
        Self {
            building: textures.get(format_compact!("ui/building-{}", kind.as_ref())),
            background: textures.get("ui/background-building-space"),
            background_star,
            left_padding,
            n,
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

            let n = std::cmp::min(
                self.n,
                (rect.width() - self.left_padding) as u32 / self.building.size.x as u32,
            );
            for i in 0..n {
                let v = egui::Vec2::new(
                    self.building.size.x * i as f32 + self.left_padding,
                    0.5 * (rect.height() - self.building.size.y),
                );
                painter.add(epaint::RectShape {
                    rect: egui::Rect::from_min_size(rect.min + v, self.building.size),
                    rounding: egui::Rounding::ZERO,
                    fill: egui::Color32::WHITE,
                    stroke: egui::Stroke::NONE,
                    blur_width: 0.0,
                    fill_texture_id: self.building.id,
                    uv,
                });
            }
        }
        response
    }
}
