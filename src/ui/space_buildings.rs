use bevy::prelude::*;
use bevy_egui::{
    EguiContexts,
    egui::{self, epaint, load::SizedTexture},
};
use compact_str::format_compact;
use strum::IntoEnumIterator;

use super::{
    HELP_TOOLTIP_WIDTH, OccupiedScreenSpace, UiTextures, WindowsOpenState, help::HelpItem,
};
use crate::planet::*;

const BUILDING_BACKGROUND_SIZE: (u32, u32) = (336, 48);

pub fn space_buildings_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    mut sim: ResMut<Sim>,
    window: bevy::prelude::Query<
        &mut bevy::window::Window,
        bevy::prelude::With<bevy::window::PrimaryWindow>,
    >,
    textures: Res<UiTextures>,
    params: Res<Params>,
) {
    if !wos.space_building {
        return;
    }
    let window_width = window.get_single().unwrap().width();

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
                        window_width,
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
    window_width: f32,
) {
    let build_max = attrs.build_max.unwrap();
    let cannot_build_reason = if build_max <= planet.space_building(kind).n {
        Some(CannotBuildReason::Limit)
    } else if let Err(cost) = planet.buildable(attrs) {
        Some(CannotBuildReason::Cost(cost))
    } else {
        None
    };
    let building = planet.space_building_mut(kind);
    let n = building.n;

    ui.horizontal(|ui| {
        let building_text = format!("{} ({}/{})", t!(kind), building.n, build_max);
        ui.add(egui::Label::new(building_text).extend());
    });

    let response = ui.add(BuildingImage::new(kind, building.n, textures));
    if response.hovered() {
        let help_item = HelpItem::SpaceBuildings(kind);
        let right_top = response.rect.right_top();
        let pos = if right_top.x + HELP_TOOLTIP_WIDTH < window_width {
            right_top
        } else {
            response.rect.left_top() + egui::vec2(-HELP_TOOLTIP_WIDTH - 20.0, 0.0)
        };

        egui::containers::show_tooltip_at(
            &response.ctx,
            response.layer_id,
            response.id,
            pos,
            |ui| {
                ui.set_max_width(HELP_TOOLTIP_WIDTH);
                help_item.ui(ui, textures, params);
            },
        );
    }

    ui.horizontal(|ui| {
        if let Some(cannot_build_reason) = cannot_build_reason {
            ui.add_enabled(false, egui::Button::new("+1"))
                .on_disabled_hover_ui(|ui| cannot_build_reason.ui(ui, textures));
        } else if ui.button("+1").clicked() {
            planet.build_space_building(kind, sim, params);
        }

        if build_max >= 5 {
            if let Some(cannot_build_reason) = cannot_build_reason {
                ui.add_enabled(false, egui::Button::new("+5"))
                    .on_disabled_hover_ui(|ui| cannot_build_reason.ui(ui, textures));
            } else if ui.button("+5").clicked() {
                for _ in 0..5 {
                    if planet.buildable(attrs).is_ok() && build_max > planet.space_building(kind).n
                    {
                        planet.build_space_building(kind, sim, params);
                    } else {
                        break;
                    }
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
            painter.add(
                epaint::RectShape::filled(rect, 0, egui::Color32::WHITE)
                    .with_texture(self.background.id, uv),
            );

            if let Some(background_star) = &self.background_star {
                painter.add(
                    epaint::RectShape::filled(
                        egui::Rect::from_min_size(rect.min, background_star.size),
                        0,
                        egui::Color32::WHITE,
                    )
                    .with_texture(background_star.id, uv),
                );
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
                painter.add(
                    epaint::RectShape::filled(
                        egui::Rect::from_min_size(rect.min + v, self.building.size),
                        0,
                        egui::Color32::WHITE,
                    )
                    .with_texture(self.building.id, uv),
                );
            }
        }
        response
    }
}

#[derive(Clone, Copy, Debug)]
enum CannotBuildReason {
    Limit,
    Cost(Cost),
}

impl CannotBuildReason {
    fn ui(&self, ui: &mut egui::Ui, textures: &UiTextures) {
        match self {
            Self::Limit => {
                ui.label(
                    egui::RichText::new(t!("building-limit-reached")).color(egui::Color32::RED),
                );
            }
            Self::Cost(cost) => {
                ui.horizontal_centered(|ui| {
                    ui.label(egui::RichText::new(t!("not-enough")).color(egui::Color32::RED));
                    let image = match cost {
                        Cost::Power(_, _) => "ui/icon-power",
                        Cost::Material(_) => "ui/icon-material",
                        Cost::GenePoint(_) => "ui/icon-gene",
                    };
                    ui.image(textures.get(image));
                });
            }
        }
    }
}
