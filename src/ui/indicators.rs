use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use bevy_egui::{EguiContexts, egui};
use geom::Coords;

use crate::{
    conf::Conf,
    draw::UpdateDraw,
    overlay::OverlayLayerKind,
    planet::{Cost, KELVIN_CELSIUS, Params, Planet, Structure, TileEvent},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    text::WithUnitDisplay,
};

use super::{UiTextures, misc::LabelWithIcon};

pub const TILE_INFO_INDICATOR_WIDTH: f32 = 208.0;

pub fn indicators(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    cursor_mode: Res<CursorMode>,
    (planet, textures, params, conf): (Res<Planet>, Res<UiTextures>, Res<Params>, Res<Conf>),
    diagnostics_store: Res<DiagnosticsStore>,
    mut last_hover_tile: Local<Option<Coords>>,
    mut current_layer: ResMut<OverlayLayerKind>,
    mut update_draw: ResMut<UpdateDraw>,
) {
    let ctx = egui_ctxs.ctx_mut();
    let visuals = &ctx.style().visuals;
    let frame = egui::Frame::default()
        .fill(visuals.window_fill.gamma_multiply(0.92))
        .stroke(visuals.window_stroke)
        .inner_margin(egui::Margin::same(4));

    // Information about the hovered tile
    let mut y = occupied_screen_space.toolbar_height;
    let hover_tile = hover_tile.single();
    last_hover_tile.get_or_insert(Coords(0, 0));
    if hover_tile.0.is_some() {
        *last_hover_tile = hover_tile.0;
    }

    let p = hover_tile.0.unwrap_or(last_hover_tile.unwrap());
    let rect = egui::Window::new("hover-tile-indicator")
        .vscroll(false)
        .resizable(false)
        .title_bar(false)
        .default_width(TILE_INFO_INDICATOR_WIDTH)
        .frame(frame)
        .anchor(egui::Align2::RIGHT_TOP, [0.0, y])
        .show(ctx, |ui| {
            tile_info_indicators(
                ui,
                p,
                &planet,
                &textures,
                &mut current_layer,
                &mut update_draw,
            );
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
    y += rect.height();

    // Information about selected tool
    if !matches!(*cursor_mode, CursorMode::Normal) {
        let rect = egui::Window::new("cursor-mode-indicator")
            .vscroll(false)
            .resizable(false)
            .title_bar(false)
            .frame(frame)
            .anchor(egui::Align2::RIGHT_TOP, [0.0, y])
            .show(ctx, |ui| {
                cursor_mode_indicator(ui, &cursor_mode, &textures, &planet, &params);
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
        y += rect.height();
    }

    // FPS indicator
    const FPS: bevy::diagnostic::DiagnosticPath =
        bevy::diagnostic::DiagnosticPath::const_new("fps");
    if conf.show_fps {
        let rect = egui::Window::new("fps-indicator")
            .vscroll(false)
            .resizable(false)
            .title_bar(false)
            .frame(frame)
            .anchor(egui::Align2::RIGHT_TOP, [0.0, y])
            .show(ctx, |ui| {
                if let Some(fps) = diagnostics_store.get(&FPS).and_then(|d| d.average()) {
                    ui.label(format!("FPS: {:.2}", fps));
                }
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
    }
}

fn cursor_mode_indicator(
    ui: &mut egui::Ui,
    cursor_mode: &CursorMode,
    textures: &UiTextures,
    planet: &Planet,
    params: &Params,
) {
    let cost_list = crate::action::cursor_mode_lack_and_cost(planet, params, cursor_mode);
    let text = match cursor_mode {
        CursorMode::Normal => "".into(),
        CursorMode::Demolition => {
            t!("demolition")
        }
        CursorMode::Build(kind) => {
            t!(kind)
        }
        CursorMode::TileEvent(kind) => {
            t!(kind)
        }
        CursorMode::SpawnAnimal(animal_id) => {
            format!("{} {}", t!("animal"), t!("animal", animal_id))
        }
        CursorMode::EditBiome(biome) => {
            format!("biome editing: {}", biome.as_ref())
        }
        CursorMode::ChangeHeight(value) => {
            format!("change height: {}", value)
        }
        CursorMode::PlaceSettlement(id, age) => {
            format!("settlement: {} {:?}", id, age)
        }
        CursorMode::CauseEvent(kind) => AsRef::<str>::as_ref(&kind).to_owned(),
    };
    ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
    ui.horizontal(|ui| {
        for (lack, cost) in &cost_list {
            let (texture, s) = match cost {
                Cost::Power(value, _) => (
                    textures.get("ui/icon-power"),
                    WithUnitDisplay::Power(*value).to_string(),
                ),
                Cost::Material(value) => (
                    textures.get("ui/icon-material"),
                    WithUnitDisplay::Material(*value).to_string(),
                ),
                Cost::GenePoint(value) => (
                    textures.get("ui/icon-gene"),
                    WithUnitDisplay::GenePoint(*value).to_string(),
                ),
            };
            if *lack {
                ui.add(LabelWithIcon::new(
                    texture,
                    egui::RichText::new(s).color(egui::Color32::RED),
                ));
            } else {
                ui.add(LabelWithIcon::new(texture, s));
            }
        }
    });
}

pub fn power_indicator(ui: &mut egui::Ui, textures: &UiTextures, power: f32, used_power: f32) {
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-power");
        ui.image(texture).on_hover_text(t!("power"));
        let used_power = crate::text::format_float_1000(used_power, 1);
        let power = crate::text::format_float_1000(power, 1);
        ui.label(format!("{} / {} TW", used_power, power));
    });
}

pub fn material_indicator(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    material: f32,
    diff_material: f32,
) {
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-material");
        ui.image(texture).on_hover_text(t!("material"));
        ui.label(WithUnitDisplay::Material(material).to_string());
        ui.label(
            egui::RichText::new(format!("(+{})", WithUnitDisplay::Material(diff_material))).small(),
        );
    });
}

pub fn gene_point_indicator(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    gene_point: f32,
    diff_gene_point: f32,
) {
    ui.horizontal(|ui| {
        let texture = textures.get("ui/icon-gene");
        ui.image(texture).on_hover_text(t!("gene-points"));
        ui.label(WithUnitDisplay::GenePoint(gene_point).to_string());
        ui.label(egui::RichText::new(format!("({:+.2})", diff_gene_point)).small());
    });
}

pub fn tile_info_indicators(
    ui: &mut egui::Ui,
    p: Coords,
    planet: &Planet,
    textures: &UiTextures,
    current_layer: &mut OverlayLayerKind,
    update_draw: &mut UpdateDraw,
) {
    let layer = *current_layer;
    let tile = &planet.map[p];
    ui.horizontal(|ui| {
        ui.radio_value(current_layer, OverlayLayerKind::None, "")
            .on_hover_text(t!("coordinates"));
        ui.image(textures.get("ui/icon-coordinates"))
            .on_hover_text(t!("coordinates"));
        ui.label(format!("[{}, {}]", p.0, p.1))
            .on_hover_text(t!("coordinates"));

        let (longitude, latitude) = planet.calc_longitude_latitude(p);
        ui.label(format!(
            "{:.0}°, {:.0}°",
            longitude * 180.0 * std::f32::consts::FRAC_1_PI,
            latitude * 180.0 * std::f32::consts::FRAC_1_PI,
        ))
        .on_hover_text(format!("{}, {}", t!("longitude"), t!("latitude")));
    });

    let buried_carbon = if tile.buried_carbon > 5000.0 {
        format!("{:.1} Gt", tile.buried_carbon / 1000.0)
    } else {
        format!("{:.1} Mt", tile.buried_carbon)
    };

    let items: &[(OverlayLayerKind, &str, String, &str)] = &[
        (
            OverlayLayerKind::Height,
            "ui/icon-height",
            format!("{:.0} m", planet.height_above_sea_level(p)),
            "height",
        ),
        (
            OverlayLayerKind::AirTemperature,
            "ui/icon-air-temperature",
            format!("{:.1} °C", tile.temp - KELVIN_CELSIUS),
            "air-temperature",
        ),
        (
            OverlayLayerKind::Rainfall,
            "ui/icon-rainfall",
            format!("{:.0} mm", tile.rainfall),
            "rainfall",
        ),
        (
            OverlayLayerKind::Fertility,
            "ui/icon-fertility",
            format!("{:.0} %", tile.fertility),
            "fertility",
        ),
        (
            OverlayLayerKind::Biomass,
            "ui/icon-biomass",
            format!("{:.1} kg/m²", tile.biomass),
            "biomass",
        ),
        (
            OverlayLayerKind::BuriedCarbon,
            "ui/icon-carbon",
            buried_carbon,
            "buried-carbon",
        ),
    ];

    for (layer, icon, label, s) in items {
        let s = t!(s);
        ui.horizontal(|ui| {
            ui.radio_value(current_layer, *layer, "").on_hover_text(&s);
            ui.image(textures.get(icon)).on_hover_text(&s);
            ui.label(label).on_hover_text(s);
        });
    }

    if layer != *current_layer {
        update_draw.update();
    }

    ui.separator();

    ui.label(t!(tile.biome));

    match &tile.structure {
        Some(Structure::Settlement(settlement)) => {
            let s = if settlement.pop >= 10.0 {
                t!("city")
            } else {
                t!("settlement")
            };
            ui.label(format!("{} ({})", s, t!("age", settlement.age),));
            ui.label(planet.civ_name(settlement.id));
            ui.label(format!("{}: {:.1}", t!("population"), settlement.pop));
        }
        Some(structure) => {
            ui.label(t!(structure.kind().as_ref()));
        }
        None => (),
    }

    if let Some(animal) = tile.largest_animal() {
        ui.label(format!("{}: {}", t!("animal"), t!("animal", animal.id)));
    }

    for tile_event in tile.tile_events.list().iter() {
        match tile_event {
            TileEvent::Fire => {
                ui.label(t!("fire"));
            }
            TileEvent::BlackDust { .. } => {
                ui.label(t!("black-dust"));
            }
            TileEvent::AerosolInjection { .. } => {
                ui.label(t!("aerosol-injection"));
            }
            TileEvent::Plague { cured, .. } => {
                if !*cured {
                    ui.label(t!("plague"));
                }
            }
            TileEvent::Vehicle { id, .. } => {
                ui.label(format!("{} ({})", t!("vehicle"), planet.civ_name(*id)));
            }
            TileEvent::Decadence { cured, .. } => {
                if !*cured {
                    ui.label(t!("decadence"));
                }
            }
            TileEvent::War { .. } => {
                ui.label(t!("war"));
            }
            TileEvent::NuclearExplosion { .. } => {
                ui.label(t!("nuclear-explosion"));
            }
            TileEvent::Troop { id, .. } => {
                ui.label(format!("{} ({})", t!("troop"), planet.civ_name(*id)));
            }
        }
    }
}
