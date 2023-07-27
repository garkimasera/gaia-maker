use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use geom::RectIter;

use super::{convert_rect, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;

pub fn map_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    params: Res<Params>,
    mut map_tex_handle: Local<Option<egui::TextureHandle>>,
    mut image_update_counter: Local<usize>,
) {
    *image_update_counter += 1;

    if !wos.map {
        return;
    }

    let ctx = egui_ctxs.ctx_mut();

    let map_tex_handle = if let Some(map_tex_handle) = &mut *map_tex_handle {
        map_tex_handle
    } else {
        let color_image = map_img(&planet, &params);
        *map_tex_handle = Some(ctx.load_texture("map", color_image, egui::TextureOptions::NEAREST));
        map_tex_handle.as_mut().unwrap()
    };

    if *image_update_counter >= 60 {
        let color_image = map_img(&planet, &params);
        map_tex_handle.set(color_image, egui::TextureOptions::NEAREST);
    }

    let rect = egui::Window::new(t!("map"))
        .open(&mut wos.map)
        .vscroll(true)
        .show(ctx, |ui| {
            paint(ui, map_tex_handle.id());
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn paint(ui: &mut egui::Ui, map_tex_id: egui::TextureId) {
    let (response, painter) =
        ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::click());

    let rect = response.rect;
    let c = rect.center();

    painter.image(
        map_tex_id,
        egui::Rect::from_center_size(c, egui::vec2(256.0, 128.0)),
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}

fn map_img(planet: &Planet, params: &Params) -> egui::ColorImage {
    let (w, h) = planet.map.size();
    let m = 2;

    let pixels = RectIter::new((0, 0), (w * m - 1, h * m - 1))
        .map(|coords| {
            let x = coords.0 / m as i32;
            let y = h as i32 - 1 - coords.1 / m as i32;
            let biome = planet.map[(x, y)].biome;
            let color = params.biomes[&biome].color;
            egui::Color32::from_rgba_unmultiplied(color[0], color[1], color[2], 255)
        })
        .collect();

    egui::ColorImage {
        size: [(w * m) as usize, (h * m) as usize],
        pixels,
    }
}
