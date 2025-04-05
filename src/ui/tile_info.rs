use bevy_egui::egui;
use geom::Coords;

use crate::planet::{KELVIN_CELSIUS, Planet, Structure, TileEvent};

use super::UiTextures;

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
