use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use bevy_egui::{EguiContexts, egui};
use geom::Coords;

use crate::{
    conf::Conf,
    planet::{Cost, KELVIN_CELSIUS, Params, Planet, Structure, TileEvent},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    text::WithUnitDisplay,
};

use super::{UiTextures, misc::LabelWithIcon};

const TOOLBAR_HEIGHT: f32 = 30.0;

pub fn indicators(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    cursor_mode: Res<CursorMode>,
    (planet, textures, params, conf): (Res<Planet>, Res<UiTextures>, Res<Params>, Res<Conf>),
    diagnostics_store: Res<DiagnosticsStore>,
    mut last_hover_tile: Local<Option<Coords>>,
) {
    let ctx = egui_ctxs.ctx_mut();
    let visuals = &ctx.style().visuals;
    let frame = egui::Frame::default()
        .fill(visuals.window_fill.gamma_multiply(0.92))
        .stroke(visuals.window_stroke)
        .inner_margin(egui::Margin::same(4));

    // Information about selected tool
    if !matches!(*cursor_mode, CursorMode::Normal) {
        let rect = egui::Window::new("cursor-mode-indicator")
            .vscroll(false)
            .resizable(false)
            .title_bar(false)
            .frame(frame)
            .anchor(egui::Align2::LEFT_TOP, [0.0, TOOLBAR_HEIGHT])
            .show(ctx, |ui| {
                cursor_mode_indicator(ui, &cursor_mode, &textures, &planet, &params);
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
    }

    // Resource indicators
    let mut y = TOOLBAR_HEIGHT;
    let rect = egui::Window::new("resource-indicators")
        .vscroll(false)
        .resizable(false)
        .title_bar(false)
        .frame(frame)
        .anchor(egui::Align2::RIGHT_TOP, [0.0, y])
        .show(ctx, |ui| {
            power_indicator(ui, &textures, planet.res.power, planet.res.used_power);
            material_indicator(ui, &textures, planet.res.material, planet.res.diff_material);
            gene_point_indicator(
                ui,
                &textures,
                planet.res.gene_point,
                planet.res.diff_gene_point,
            );
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);

    let max_width = rect.width();
    y += rect.height();

    // Information about the hovered tile
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
        .max_width(max_width)
        .frame(frame)
        .anchor(egui::Align2::RIGHT_TOP, [0.0, y])
        .show(ctx, |ui| {
            ui_tile_info(ui, p, &planet, &textures);
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
    y += rect.height();

    // FPS indicator
    const FPS: bevy::diagnostic::DiagnosticPath =
        bevy::diagnostic::DiagnosticPath::const_new("fps");
    if conf.show_fps {
        let rect = egui::Window::new("fps-indicator")
            .vscroll(false)
            .resizable(false)
            .title_bar(false)
            .max_width(max_width)
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
pub fn ui_tile_info(ui: &mut egui::Ui, p: Coords, planet: &Planet, textures: &UiTextures) {
    let tile = &planet.map[p];
    ui.horizontal(|ui| {
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

    let items: &[(&str, String, &str)] = &[
        (
            "ui/icon-height",
            format!("{:.0} m", planet.height_above_sea_level(p)),
            "height",
        ),
        (
            "ui/icon-air-temperature",
            format!("{:.1} °C", tile.temp - KELVIN_CELSIUS),
            "air-temperature",
        ),
        (
            "ui/icon-rainfall",
            format!("{:.0} mm", tile.rainfall),
            "rainfall",
        ),
        (
            "ui/icon-fertility",
            format!("{:.0} %", tile.fertility),
            "fertility",
        ),
        (
            "ui/icon-biomass",
            format!("{:.1} kg/m²", tile.biomass),
            "biomass",
        ),
    ];

    for (icon, label, s) in items {
        let s = t!(s);
        ui.horizontal(|ui| {
            ui.image(textures.get(icon)).on_hover_text(&s);
            ui.label(label).on_hover_text(s);
        });
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
