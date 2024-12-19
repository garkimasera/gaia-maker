use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use geom::RectIter;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{OccupiedScreenSpace, WindowsOpenState};
use crate::overlay::{ColorMaterials, OverlayLayerKind};
use crate::planet::*;
use crate::screen::{Centering, InScreenTileRange};

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
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Resource)]
pub struct NeedUpdate(bool);

pub fn map_window(
    mut egui_ctxs: EguiContexts,
    mut wos: ResMut<WindowsOpenState>,
    mut ew_centering: EventWriter<Centering>,
    mut need_update: ResMut<NeedUpdate>,
    planet: Res<Planet>,
    params: Res<Params>,
    color_materials: Res<ColorMaterials>,
    mut screen: (
        Res<InScreenTileRange>,
        ResMut<OccupiedScreenSpace>,
        Query<&bevy_egui::EguiSettings, With<bevy::window::PrimaryWindow>>,
    ),
    (mut map_tex_handle, mut image_update_counter, mut map_layer, mut before_map_layer): (
        Local<Option<egui::TextureHandle>>,
        Local<usize>,
        Local<MapLayer>,
        Local<MapLayer>,
    ),
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

    if *image_update_counter >= 60 || *map_layer != *before_map_layer || need_update.0 {
        let color_image = map_img(&planet, &params, *map_layer, &color_materials, m);
        map_tex_handle.set(color_image, egui::TextureOptions::NEAREST);
        *before_map_layer = *map_layer;
        *image_update_counter = 0;
        need_update.0 = false;
    }

    let rect = egui::Window::new(t!("map"))
        .open(&mut wos.map)
        .vscroll(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                egui::ComboBox::from_id_salt("map-layer-items")
                    .selected_text(t!(*map_layer))
                    .show_ui(ui, |ui| {
                        for l in MapLayer::iter() {
                            ui.selectable_value(&mut *map_layer, l, t!(l));
                        }
                    });
                ui.separator();
                let response = map_ui(ui, map_tex_handle, &screen, m as f32);
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos {
                        let pos = pos - response.rect.min;
                        let pos = Vec2::new(
                            pos.x / m as f32 * TILE_SIZE,
                            (planet.map.size().1 as f32 - pos.y / m as f32 - 1.0) * TILE_SIZE,
                        );
                        ew_centering.send(Centering(pos));
                    }
                }
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
        Query<&bevy_egui::EguiSettings, With<bevy::window::PrimaryWindow>>,
    ),
    scale: f32,
) -> egui::Response {
    let egui_settings = egui_settings.single();
    let [w, h] = map_tex_handle.size();
    let size = egui::vec2(w as _, h as _);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());

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
    painter.rect_stroke(r1, egui::Rounding::ZERO, stroke1);
    painter.rect_stroke(r2, egui::Rounding::ZERO, stroke2);
    painter.rect_stroke(
        r1.translate(egui::vec2(w as f32, 0.0)),
        egui::Rounding::ZERO,
        stroke1,
    );
    painter.rect_stroke(
        r2.translate(egui::vec2(w as f32, 0.0)),
        egui::Rounding::ZERO,
        stroke2,
    );
    painter.rect_stroke(
        r1.translate(egui::vec2(-(w as f32), 0.0)),
        egui::Rounding::ZERO,
        stroke1,
    );
    painter.rect_stroke(
        r2.translate(egui::vec2(-(w as f32), 0.0)),
        egui::Rounding::ZERO,
        stroke2,
    );

    response
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
                MapLayer::AirTemperature => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::AirTemperature)
                }
                MapLayer::Rainfall => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Rainfall)
                }
                MapLayer::Fertility => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Fertility)
                }
                MapLayer::Biomass => {
                    color_materials.get_rgb(planet, (x, y).into(), OverlayLayerKind::Biomass)
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
