use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use geom::RectIter;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{convert_rect, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::overlay::{ColorMaterials, OverlayLayerKind};
use crate::planet::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, AsRefStr, EnumIter, Resource)]
#[strum(serialize_all = "kebab-case")]
pub enum MapLayer {
    #[default]
    Biome,
    Height,
    AirTemprature,
    Rainfall,
    Fertility,
}

pub fn map_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    params: Res<Params>,
    color_materials: Res<ColorMaterials>,
    mut map_tex_handle: Local<Option<egui::TextureHandle>>,
    mut image_update_counter: Local<usize>,
    mut map_layer: Local<MapLayer>,
    mut before_map_layer: Local<MapLayer>,
) {
    *image_update_counter += 1;

    if !wos.map {
        return;
    }

    let ctx = egui_ctxs.ctx_mut();
    let m = 3;

    let map_tex_handle = if let Some(map_tex_handle) = &mut *map_tex_handle {
        map_tex_handle
    } else {
        let color_image = map_img(&planet, &params, *map_layer, &color_materials, m);
        *map_tex_handle = Some(ctx.load_texture("map", color_image, egui::TextureOptions::NEAREST));
        map_tex_handle.as_mut().unwrap()
    };

    if *image_update_counter >= 60 || *map_layer != *before_map_layer {
        let color_image = map_img(&planet, &params, *map_layer, &color_materials, m);
        map_tex_handle.set(color_image, egui::TextureOptions::NEAREST);
        *before_map_layer = *map_layer;
    }

    let rect = egui::Window::new(t!("map"))
        .open(&mut wos.map)
        .vscroll(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                egui::ComboBox::from_id_source("map-layer-items")
                    .selected_text(t!(map_layer.as_ref()))
                    .show_ui(ui, |ui| {
                        for l in MapLayer::iter() {
                            ui.selectable_value(&mut *map_layer, l, t!(l.as_ref()));
                        }
                    });
                ui.separator();
                map_ui(ui, map_tex_handle);
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn map_ui(ui: &mut egui::Ui, map_tex_handle: &egui::TextureHandle) {
    let [w, h] = map_tex_handle.size();
    let size = egui::vec2(w as _, h as _);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        ui.painter().image(
            map_tex_handle.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    }
}

fn map_img(
    planet: &Planet,
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

            let color = match map_layer {
                MapLayer::Biome => {
                    let biome = planet.map[(x, y)].biome;
                    params.biomes[&biome].color
                }
                MapLayer::Height => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Height)
                }
                MapLayer::AirTemprature => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::AirTemprature)
                }
                MapLayer::Rainfall => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Rainfall)
                }
                MapLayer::Fertility => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Fertility)
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
