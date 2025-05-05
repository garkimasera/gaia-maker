use std::collections::BTreeMap;
use std::sync::LazyLock;

use bevy::prelude::*;
use bevy_egui::egui::epaint;
use bevy_egui::{EguiContexts, egui};
use geom::{Coords, RectIter};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{OccupiedScreenSpace, UiTextures, WindowsOpenState};
use crate::manage_planet::SwitchPlanet;
use crate::overlay::{ColorMaterials, OverlayLayerKind};
use crate::planet::*;
use crate::screen::{Centering, InScreenTileRange};

pub const H_LEGEND_IMG: u32 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Default, AsRefStr, EnumIter, Resource)]
#[strum(serialize_all = "kebab-case")]
pub enum MapLayer {
    #[default]
    Biome,
    Height,
    AirTemperature,
    Rainfall,
    Fertility,
    Biomass,
    BuriedCarbon,
    Cities,
    Civilizations,
    Structures,
}

impl MapLayer {
    fn icon(&self) -> &'static str {
        match self {
            Self::Biome => "ui/icon-map",
            Self::Height => "ui/icon-height",
            Self::AirTemperature => "ui/icon-air-temperature",
            Self::Rainfall => "ui/icon-rainfall",
            Self::Fertility => "ui/icon-fertility",
            Self::Biomass => "ui/icon-biomass",
            Self::BuriedCarbon => "ui/icon-carbon",
            Self::Cities => "ui/icon-city",
            Self::Civilizations => "ui/icon-civilization",
            Self::Structures => "ui/icon-build",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Resource)]
pub struct NeedUpdate(bool);

pub fn map_window(
    mut egui_ctxs: EguiContexts,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_centering: EventWriter<Centering>,
    mut need_update: ResMut<NeedUpdate>,
    (planet, sim, params): (Res<Planet>, Res<Sim>, Res<Params>),
    color_materials: Res<ColorMaterials>,
    mut er_switch_planet: EventReader<SwitchPlanet>,
    mut screen: (
        Res<InScreenTileRange>,
        ResMut<OccupiedScreenSpace>,
        Query<&bevy_egui::EguiContextSettings, With<bevy::window::PrimaryWindow>>,
    ),
    (mut map_tex_handle, mut image_update_counter, mut map_layer, mut before_map_layer): (
        Local<Option<egui::TextureHandle>>,
        Local<usize>,
        Local<MapLayer>,
        Local<MapLayer>,
    ),
    textures: Res<UiTextures>,
    se_player: crate::audio::SoundEffectPlayer,
    mut legend: Local<Option<Legend>>,
) {
    *image_update_counter += 1;

    if !wos.map {
        let rect = egui::Window::new("reports-map")
            .anchor(egui::Align2::RIGHT_BOTTOM, [0.0, 0.0])
            .frame(super::misc::small_window_frame(egui_ctxs.ctx_mut()))
            .resizable(false)
            .title_bar(false)
            .show(egui_ctxs.ctx_mut(), |ui| {
                if ui
                    .add(egui::ImageButton::new(textures.get("ui/icon-map")))
                    .on_hover_text(t!("map"))
                    .clicked()
                {
                    wos.map = true;
                }
            })
            .unwrap()
            .response
            .rect;
        screen.1.push_egui_window_rect(rect);
        return;
    }

    let ctx = egui_ctxs.ctx_mut();
    let m = 3;

    let legend = if let Some(legend) = &*legend {
        legend
    } else {
        *legend = Some(Legend::new(ctx, &color_materials, &params));
        legend.as_ref().unwrap()
    };

    let map_tex_handle = if let Some(map_tex_handle) = &mut *map_tex_handle {
        map_tex_handle
    } else {
        let color_image = map_img(&planet, &sim, &params, *map_layer, &color_materials, m);
        *map_tex_handle = Some(ctx.load_texture("map", color_image, egui::TextureOptions::NEAREST));
        map_tex_handle.as_mut().unwrap()
    };

    let switched = er_switch_planet.read().fold(false, |_, _| true);
    if *image_update_counter >= 60 || *map_layer != *before_map_layer || need_update.0 || switched {
        let color_image = map_img(&planet, &sim, &params, *map_layer, &color_materials, m);
        map_tex_handle.set(color_image, egui::TextureOptions::NEAREST);
        *before_map_layer = *map_layer;
        *image_update_counter = 0;
        need_update.0 = false;
    }

    let rect = egui::Window::new(t!("map"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [0.0, 0.0])
        .title_bar(false)
        .vscroll(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_height(330.0);
            ui.horizontal(|ui| {
                if ui.button("▼").clicked() {
                    wos.map = false;
                }
                ui.heading(format!("{} - {}", t!("map"), t!(map_layer.as_ref())));
            });
            ui.separator();

            ui.vertical(|ui| {
                let layout = egui::Layout::left_to_right(egui::Align::Min);
                ui.with_layout(layout, |ui| {
                    for l in MapLayer::iter() {
                        let button =
                            egui::Button::image(textures.get(l.icon())).selected(l == *map_layer);
                        if ui.add(button).on_hover_text(t!(l)).clicked() {
                            *map_layer = l;
                            se_player.play("select-item");
                        }
                    }
                });
                ui.separator();
                let response = map_ui(ui, map_tex_handle, &screen, m as f32);
                if response.clicked() | response.dragged() {
                    if let Some(pos) = response.interact_pointer_pos {
                        let pos = pos - response.rect.min;
                        let pos = Vec2::new(
                            pos.x / m as f32 * TILE_SIZE,
                            (planet.map.size().1 as f32 - pos.y / m as f32 - 1.0) * TILE_SIZE,
                        );
                        ew_centering.send(Centering(pos));
                    }
                }
                legend.ui(ui, *map_layer, &planet, &params);
            });
        })
        .unwrap()
        .response
        .rect;
    screen.1.push_egui_window_rect(rect);
}

pub fn update(mut need_update: ResMut<NeedUpdate>) {
    need_update.0 = true;
}

fn map_ui(
    ui: &mut egui::Ui,
    map_tex_handle: &egui::TextureHandle,
    (in_screen_tile_range, occupied_screen_space, egui_settings): &(
        Res<InScreenTileRange>,
        ResMut<OccupiedScreenSpace>,
        Query<&bevy_egui::EguiContextSettings, With<bevy::window::PrimaryWindow>>,
    ),
    scale: f32,
) -> egui::Response {
    let egui_settings = egui_settings.single();
    let [w, h] = map_tex_handle.size();
    let size = egui::vec2(w as _, h as _);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click() | egui::Sense::drag());

    if !ui.is_rect_visible(rect) {
        return response;
    }

    let painter = ui.painter_at(rect);
    painter.image(
        map_tex_handle.id(),
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    let stroke1 = egui::Stroke {
        width: 1.0,
        color: egui::Color32::BLACK,
    };
    let stroke2 = egui::Stroke {
        width: 1.0,
        color: egui::Color32::WHITE,
    };
    let hide_by_sidebar =
        (occupied_screen_space.occupied_left * egui_settings.scale_factor / TILE_SIZE) as i32;
    let r1 = egui::Rect::from_two_pos(
        egui::pos2(
            in_screen_tile_range.to.0 as f32 * scale,
            h as f32 - in_screen_tile_range.y_to_from_not_clamped.0 as f32 * scale,
        ),
        egui::pos2(
            (in_screen_tile_range.from.0 + hide_by_sidebar) as f32 * scale,
            h as f32 - in_screen_tile_range.y_to_from_not_clamped.1 as f32 * scale,
        ),
    )
    .translate(rect.left_top().to_vec2());
    let r2 = egui::Rect::from_two_pos(r1.min + egui::vec2(1.0, 1.0), r1.max - egui::vec2(1.0, 1.0));
    let cr = egui::CornerRadius::ZERO;
    let sk = egui::StrokeKind::Middle;
    painter.rect_stroke(r1, cr, stroke1, sk);
    painter.rect_stroke(r2, cr, stroke2, sk);
    painter.rect_stroke(r1.translate(egui::vec2(w as f32, 0.0)), cr, stroke1, sk);
    painter.rect_stroke(r2.translate(egui::vec2(w as f32, 0.0)), cr, stroke2, sk);
    painter.rect_stroke(r1.translate(egui::vec2(-(w as f32), 0.0)), cr, stroke1, sk);
    painter.rect_stroke(r2.translate(egui::vec2(-(w as f32), 0.0)), cr, stroke2, sk);

    response
}

fn map_img(
    planet: &Planet,
    sim: &Sim,
    params: &Params,
    map_layer: MapLayer,
    color_materials: &ColorMaterials,
    m: u32,
) -> egui::ColorImage {
    let (w, h) = planet.map.size();

    let pixels = RectIter::new((0, 0), (w * m - 1, h * m - 1))
        .map(|coords| {
            let x = coords.0 / m as i32;
            let y = h as i32 - 1 - coords.1 / m as i32;
            let p = Coords::new(x, y);

            let color = match map_layer {
                MapLayer::Biome => {
                    let biome = planet.map[(x, y)].biome;
                    params.biomes[&biome].color
                }
                MapLayer::Height => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::Height, params)
                }
                MapLayer::AirTemperature => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::AirTemperature, params)
                }
                MapLayer::Rainfall => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::Rainfall, params)
                }
                MapLayer::Fertility => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::Fertility, params)
                }
                MapLayer::Biomass => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::Biomass, params)
                }
                MapLayer::BuriedCarbon => {
                    color_materials.get_rgb(planet, p, OverlayLayerKind::BuriedCarbon, params)
                }
                MapLayer::Cities => {
                    if let Some(Structure::Settlement(settlement)) = &planet.map[(x, y)].structure {
                        CITY_COLORS[settlement.age as usize]
                    } else if planet.map[(x, y)].biome.is_land() {
                        params.biomes[&Biome::Rock].color
                    } else {
                        params.biomes[&Biome::Ocean].color
                    }
                }
                MapLayer::Civilizations => {
                    if let Some((id, _)) = &sim.domain[(x, y)] {
                        params.animals[id]
                            .civ
                            .as_ref()
                            .map(|civ| civ.color)
                            .unwrap_or_default()
                    } else if planet.map[(x, y)].biome.is_land() {
                        params.biomes[&Biome::Rock].color
                    } else {
                        params.biomes[&Biome::Ocean].color
                    }
                }
                MapLayer::Structures => {
                    if let Some(kind) = planet.map[(x, y)]
                        .structure
                        .as_ref()
                        .map(|s| s.kind())
                        .and_then(|kind| {
                            if matches!(kind, StructureKind::Settlement) {
                                None
                            } else {
                                Some(kind)
                            }
                        })
                    {
                        STRUCTURE_COLORS[&kind]
                    } else if planet.map[(x, y)].biome.is_land() {
                        params.biomes[&Biome::Rock].color
                    } else {
                        params.biomes[&Biome::Ocean].color
                    }
                }
            };
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], 255)
        })
        .collect();

    egui::ColorImage {
        size: [(w * m) as usize, (h * m) as usize],
        pixels,
    }
}

pub struct Legend {
    gradation_images: Vec<(egui::TextureHandle, egui::load::SizedTexture)>,
    layout: egui::Layout,
    biome_colors: Vec<(Biome, [u8; 3])>,
}

impl Legend {
    fn new(ctx: &mut egui::Context, color_materials: &ColorMaterials, params: &Params) -> Self {
        let h = H_LEGEND_IMG;

        let gradation_images = color_materials
            .color_list()
            .iter()
            .enumerate()
            .map(|(i, colors)| {
                let w = colors.len() as u32;
                let pixels = RectIter::new((0, 0), (w - 1, h - 1))
                    .map(|coords| {
                        let color = colors[coords.0 as usize].to_srgba();
                        egui::Color32::from_rgba_unmultiplied(
                            (color.red * 255.0) as u8,
                            (color.green * 255.0) as u8,
                            (color.blue * 255.0) as u8,
                            255,
                        )
                    })
                    .collect();
                let image = egui::ColorImage {
                    size: [w as usize, h as usize],
                    pixels,
                };
                let texture_handle = ctx.load_texture(
                    format!("legend_gradation_image_{}", i),
                    image,
                    egui::TextureOptions::NEAREST,
                );
                let sized_texture = egui::load::SizedTexture::from_handle(&texture_handle);
                (texture_handle, sized_texture)
            })
            .collect();

        let layout = egui::Layout::from_main_dir_and_cross_align(
            egui::Direction::LeftToRight,
            egui::Align::Center,
        )
        .with_main_wrap(true);

        // Biome order in map legend
        let biomes = [
            Biome::Rock,
            Biome::Ocean,
            Biome::IceSheet,
            Biome::SeaIce,
            Biome::Desert,
            Biome::Tundra,
            Biome::Grassland,
            Biome::BorealForest,
            Biome::TemperateForest,
            Biome::TropicalRainforest,
        ];
        let biome_colors: Vec<_> = biomes
            .iter()
            .map(|biome| (*biome, params.biomes[biome].color))
            .collect();

        Self {
            gradation_images,
            layout,
            biome_colors,
        }
    }

    fn ui(&self, ui: &mut egui::Ui, map_layer: MapLayer, planet: &Planet, params: &Params) {
        ui.add_space(3.0);
        match map_layer {
            MapLayer::AirTemperature
            | MapLayer::Rainfall
            | MapLayer::Fertility
            | MapLayer::Biomass
            | MapLayer::BuriedCarbon => {
                ui_high_low(ui, self.gradation_images[0].1);
            }
            MapLayer::Height => {
                ui_high_low(ui, self.gradation_images[1].1);
            }
            MapLayer::Biome => {
                let legend_items = self
                    .biome_colors
                    .iter()
                    .map(|&(biome, color)| (color, t!(biome)));
                self.ui_color_legend(ui, legend_items);
            }
            MapLayer::Cities => {
                let legend_items =
                    CivilizationAge::iter().map(|age| (CITY_COLORS[age as usize], t!("age", age)));
                self.ui_color_legend(ui, legend_items);
            }
            MapLayer::Civilizations => {
                let legend_items = civilization_color_legends(planet, params);
                self.ui_color_legend(ui, legend_items.into_iter());
            }
            MapLayer::Structures => {
                let legend_items = STRUCTURE_COLORS.iter().map(|(kind, color)| (*color, t!(kind)));
                self.ui_color_legend(ui, legend_items);
            }
        }
    }

    fn ui_color_legend(&self, ui: &mut egui::Ui, items: impl Iterator<Item = ([u8; 3], String)>) {
        ui.allocate_ui(egui::vec2(ui.available_size_before_wrap().x, 0.0), |ui| {
            ui.with_layout(self.layout, |ui| {
                for (color, s) in items {
                    ui.add(ColorLegend::new(color, s));
                }
            })
        });
    }
}

fn ui_high_low(ui: &mut egui::Ui, texture: egui::load::SizedTexture) {
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.label(t!("low"));
            ui.image(texture);
            ui.label(t!("high"));
        });
    });
}

const CITY_COLORS: [[u8; 3]; CivilizationAge::LEN] = [
    [255, 0, 0],
    [255, 92, 0],
    [255, 240, 0],
    [0, 145, 0],
    [0, 204, 255],
    [190, 0, 255],
];

static STRUCTURE_COLORS: LazyLock<BTreeMap<StructureKind, [u8; 3]>> = LazyLock::new(|| {
    let mut map = BTreeMap::new();
    map.insert(StructureKind::OxygenGenerator, [0, 128, 255]);
    map.insert(StructureKind::Rainmaker, [0, 255, 255]);
    map.insert(StructureKind::FertilizationPlant, [255, 160, 0]);
    map.insert(StructureKind::Heater, [255, 0, 0]);
    map.insert(StructureKind::CarbonCapturer, [192, 192, 192]);
    map.insert(StructureKind::GiftTower, [190, 0, 255]);
    map
});

fn civilization_color_legends(planet: &Planet, params: &Params) -> Vec<([u8; 3], String)> {
    let mut civs = BTreeMap::new();
    for civ in &planet.civs {
        let color = params.animals[civ.0]
            .civ
            .as_ref()
            .map(|c| c.color)
            .unwrap_or_default();
        civs.insert(*civ.0, (color, planet.civ_name(*civ.0)));
    }
    civs.into_values().collect()
}

struct ColorLegend {
    color: egui::Color32,
    text: egui::WidgetText,
}

impl ColorLegend {
    fn new(color: [u8; 3], text: impl Into<egui::WidgetText>) -> Self {
        Self {
            color: egui::Color32::from_rgb(color[0], color[1], color[2]),
            text: text.into(),
        }
    }
}

impl egui::Widget for ColorLegend {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let space = 3.0;
        let left_space = 2.0;
        let layout_job =
            self.text
                .into_layout_job(ui.style(), egui::FontSelection::Default, egui::Align::Min);
        let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

        let icon_size = egui::Vec2::new(H_LEGEND_IMG as f32, H_LEGEND_IMG as f32);
        let galley_size = galley.rect.size();
        let desired_size = egui::Vec2::new(
            icon_size.x + galley_size.x + space,
            icon_size.y.max(galley_size.y) + left_space,
        );

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), galley.text())
        });

        let (icon_pos_y, galley_pos_y) = if icon_size.y > galley_size.y {
            (0.0, (icon_size.y - galley_size.y) / 2.0)
        } else {
            ((galley_size.y - icon_size.y) / 2.0, 0.0)
        };

        let icon_rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, icon_pos_y), icon_size)
            .translate(rect.left_top().to_vec2());
        let galley_pos = rect.left_top() + egui::Vec2::new(icon_size.x + space, galley_pos_y);

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();
            painter.add(epaint::RectShape::filled(
                icon_rect,
                egui::CornerRadius::ZERO,
                self.color,
            ));
            painter.add(epaint::TextShape::new(
                galley_pos,
                galley,
                ui.style().visuals.text_color(),
            ));
        }
        response
    }
}
