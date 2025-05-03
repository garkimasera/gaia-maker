use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_plot as plot;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{OccupiedScreenSpace, UiTextures, WindowsOpenState, help::HelpItem};
use crate::{manage_planet::SaveState, planet::*};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    Planet,
    Atmosphere,
    Civilization,
    History,
}

pub fn stat_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    planet: Res<Planet>,
    params: Res<Params>,
    save_state: Res<SaveState>,
    textures: Res<UiTextures>,
    mut current_panel: Local<Panel>,
    mut current_civ_id: Local<Option<AnimalId>>,
    mut current_graph_item: Local<GraphItem>,
) {
    if !wos.stat {
        let rect = egui::Window::new("stat-expander")
            .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
            .frame(super::misc::small_window_frame(egui_ctxs.ctx_mut()))
            .resizable(false)
            .title_bar(false)
            .show(egui_ctxs.ctx_mut(), |ui| {
                if ui
                    .add(egui::ImageButton::new(textures.get("ui/icon-stat")))
                    .on_hover_text(t!("statistics"))
                    .clicked()
                {
                    wos.stat = true;
                }
            })
            .unwrap()
            .response
            .rect;
        occupied_screen_space.push_egui_window_rect(rect);
        occupied_screen_space.stat_width = rect.width();
        return;
    }

    let rect = egui::Window::new("stat-window")
        .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
        .title_bar(false)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("▼").clicked() {
                    wos.stat = false;
                }
                let title = if *current_panel == Panel::History {
                    format!("{} - {}", t!("statistics"), t!(current_graph_item.as_ref()))
                } else {
                    t!("statistics")
                };
                ui.heading(title);
            });
            ui.separator();

            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, t!(panel));
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Planet => planet_stat(
                    ui,
                    &textures,
                    &planet,
                    save_state.save_file_metadata.debug_mode_enabled,
                ),
                Panel::Atmosphere => atmo_stat(ui, &textures, &planet, &params),
                Panel::Civilization => civ_stat(ui, &textures, &planet, &mut current_civ_id),
                Panel::History => {
                    history_stat(ui, &textures, &mut current_graph_item, &planet, &params)
                }
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
    occupied_screen_space.stat_width = rect.width();
}

fn planet_stat(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    planet: &Planet,
    debug_mode_enabled: bool,
) {
    let grid = egui::Grid::new("table_planet")
        .striped(true)
        .min_col_width(16.0);
    grid.show(ui, |ui| {
        let hover_text = t!("stat_item", "planet-name");
        ui.image(textures.get("ui/icon-planet"))
            .on_hover_text(&hover_text);
        ui.label(t!("planet-name")).on_hover_text(&hover_text);
        ui.label(&planet.basics.name).on_hover_text(&hover_text);
        ui.end_row();

        let hover_text = t!("stat_item", "cycles");
        ui.image(textures.get("ui/icon-cycles"))
            .on_hover_text(&hover_text);
        ui.label(t!("cycles")).on_hover_text(&hover_text);
        ui.label(format!("{}", planet.cycles))
            .on_hover_text(&hover_text);
        ui.end_row();

        let hover_text = t!("stat_item", "radius");
        ui.image(textures.get("ui/icon-radius"))
            .on_hover_text(&hover_text);
        ui.label(t!("radius")).on_hover_text(&hover_text);
        ui.label(format!("{:.0} km", planet.basics.radius / 1000.0))
            .on_hover_text(&hover_text);
        ui.end_row();

        let hover_text = t!("help", "solar-constant");
        ui.image(textures.get("ui/icon-solar-constant"))
            .on_hover_text(&hover_text);
        ui.label(t!("solar-constant")).on_hover_text(&hover_text);
        ui.label(format!(
            "{:.0} W/m² ({:+.0}%)",
            planet.basics.solar_constant,
            (planet.state.solar_power_multiplier - 1.0) * 100.0
        ))
        .on_hover_text(&hover_text);
        ui.end_row();

        let hover_text = t!("help", "biomass");
        ui.image(textures.get("ui/icon-biomass"))
            .on_hover_text(&hover_text);
        ui.label(t!("biomass")).on_hover_text(&hover_text);
        ui.label(format!("{:.1} Gt", planet.stat.sum_biomass * 1e-3))
            .on_hover_text(&hover_text);
        ui.end_row();

        let sum_pop: f32 = planet.civs.iter().map(|civ| civ.1.total_pop).sum();
        let hover_text = t!("stat_item", "population");
        ui.image(textures.get("ui/icon-population"))
            .on_hover_text(&hover_text);
        ui.label(t!("population")).on_hover_text(&hover_text);
        ui.label(format!("{:.0}", sum_pop.abs()))
            .on_hover_text(&hover_text);
    });

    if debug_mode_enabled {
        ui.label(
            egui::RichText::new("Debug Mode Enabled")
                .color(egui::Color32::from_rgb(0xFF, 0x00, 0xFF)),
        );
    }
}

fn atmo_stat(ui: &mut egui::Ui, textures: &UiTextures, planet: &Planet, params: &Params) {
    ui.label(format!(
        "{}: {:.1} °C",
        t!("average-air-temperature"),
        planet.stat.average_air_temp - KELVIN_CELSIUS
    ));
    ui.label(format!(
        "{}: {:.0} mm",
        t!("average-rainfall"),
        planet.stat.average_rainfall
    ));
    ui.separator();
    ui.label(format!(
        "{}: {:.2} atm",
        t!("atmosphere-pressure"),
        planet.atmo.atm()
    ));
    ui.separator();

    egui::Grid::new("grid_atmo")
        .striped(true)
        .min_col_width(16.0)
        .show(ui, |ui| {
            for gas_kind in GasKind::iter() {
                let help_item = HelpItem::Atmosphere(gas_kind);
                let hover_text = t!("help", help_item);
                ui.image(textures.get(format!("ui/icon-{}", gas_kind.as_ref())))
                    .on_hover_text(&hover_text);
                ui.label(format!("{:.2}%", planet.atmo.mole_ratio[&gas_kind] * 100.0))
                    .on_hover_text(&hover_text);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(format!("{:.4} atm", planet.atmo.partial_pressure(gas_kind)))
                        .on_hover_text(&hover_text);
                });
                ui.end_row();
            }
        });

    ui.separator();
    let hover_text = t!("help", "cloud-albedo");
    ui.horizontal(|ui| {
        ui.image(textures.get("ui/icon-cloud-albedo"))
            .on_hover_text(&hover_text);
        ui.label(format!("{:.1} %", planet.cloud_albedo(params) * 100.0))
            .on_hover_text(&hover_text);
    });
}

fn civ_stat(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    planet: &Planet,
    current_civ_id: &mut Option<AnimalId>,
) {
    if planet.civs.is_empty() {
        ui.label(t!("no-civilization"));
        *current_civ_id = None;
        return;
    }

    let civ_ids: Vec<AnimalId> = planet.civs.keys().copied().collect();
    if current_civ_id.is_none() {
        *current_civ_id = Some(civ_ids[0]);
    }
    let mut selected_civ_id = current_civ_id.unwrap();

    egui::ComboBox::from_id_salt("select-civilization")
        .selected_text(planet.civ_name(selected_civ_id))
        .show_ui(ui, |ui| {
            for &id in &civ_ids {
                ui.selectable_value(&mut selected_civ_id, id, planet.civ_name(id));
            }
        });

    *current_civ_id = Some(selected_civ_id);
    let Some(c) = planet.civs.get(&selected_civ_id) else {
        *current_civ_id = None;
        return;
    };

    ui.label(format!("{}: {:.0}", t!("population"), c.total_pop));
    ui.separator();

    ui.label(t!("cities"));
    egui::Grid::new("table_cities")
        .min_col_width(16.0)
        .show(ui, |ui| {
            for age in CivilizationAge::iter() {
                let help_item = HelpItem::CivilizationAges(age);
                let hover_text = t!("help", help_item);
                ui.image(textures.get(format!("ui/icon-age-{}", age.as_ref())))
                    .on_hover_text(&hover_text);
                ui.label(t!("age", age)).on_hover_text(&hover_text);
                ui.label(format!("{}", c.total_settlement[age as usize]))
                    .on_hover_text(&hover_text);
                ui.end_row();
            }
        });
    ui.separator();

    ui.label(t!("energy-consumption"));
    egui::Grid::new("table_civ").min_col_width(16.0).show(ui, |ui| {
        let max = c
            .total_energy_consumption
            .iter()
            .map(|e| ordered_float::OrderedFloat::from(*e))
            .max()
            .unwrap()
            .into_inner();
        for src in EnergySource::iter() {
            let help_item = HelpItem::EnergySources(src);
            let hover_text = t!("help", help_item);
            ui.image(textures.get(format!("ui/icon-energy-source-{}", src.as_ref())))
                .on_hover_text(&hover_text);
            ui.label(t!("energy_source", src)).on_hover_text(&hover_text);
            let e = c.total_energy_consumption[src as usize];
            let s = if max < 1000.0 {
                format!("{} GJ", crate::text::format_float_1000(e, 0))
            } else {
                format!("{} TJ", crate::text::format_float_1000(e * 1e-3, 3))
            };
            ui.label(s).on_hover_text(&hover_text);
            ui.end_row();
        }
    });
}

fn history_stat(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    item: &mut GraphItem,
    planet: &Planet,
    params: &Params,
) {
    let layout = egui::Layout::left_to_right(egui::Align::Min);
    ui.with_layout(layout, |ui| {
        for g in GraphItem::iter() {
            let button = egui::Button::image(textures.get(g.icon())).selected(g == *item);
            if ui.add(button).on_hover_text(t!(g)).clicked() {
                *item = g;
            }
        }
    });
    ui.separator();

    let mut min = f64::MAX;
    let mut max = f64::MIN;
    let history = planet.stat.history();
    let line: plot::PlotPoints = (0..params.history.max_record)
        .map(|i| {
            let value = item.record_to_value(history.get(i));
            if value < min {
                min = value;
            }
            if value > max {
                max = value;
            }
            [
                (params.history.max_record - i - 1) as f64 * params.history.interval_cycles as f64,
                value,
            ]
        })
        .collect();
    let line = plot::Line::new(line);

    let item_copy = *item;
    let label_formatter = move |_s: &str, value: &plot::PlotPoint| item_copy.format_value(value.y);
    let x_axis_formatter = move |_, _range: &RangeInclusive<f64>| "".to_string();
    let min_bound_margin = match item {
        GraphItem::AverageAirTemperature | GraphItem::AverageSeaTemperature => 1.0e-1,
        GraphItem::AverageRainfall => 1.0e+0,
        GraphItem::Biomass | GraphItem::BuriedCarbon => 1.0e+0,
        GraphItem::Oxygen | GraphItem::Nitrogen | GraphItem::CarbonDioxide => 1.0e-5,
        GraphItem::Population => 1.0e+1,
    };
    let bound_margin = (max - min) * 0.08 + min_bound_margin;

    plot::Plot::new("history")
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .label_formatter(label_formatter)
        .x_axis_formatter(x_axis_formatter)
        .show_x(false)
        .show_y(true)
        .auto_bounds(egui::Vec2b::new(false, true))
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds(plot::PlotBounds::from_min_max(
                [0.0, min - bound_margin],
                [
                    (params.history.max_record - 1) as f64 * params.history.interval_cycles as f64,
                    max + bound_margin,
                ],
            ));
            plot_ui.line(line)
        });
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum GraphItem {
    #[default]
    AverageAirTemperature,
    AverageSeaTemperature,
    AverageRainfall,
    Biomass,
    Oxygen,
    Nitrogen,
    CarbonDioxide,
    BuriedCarbon,
    Population,
}

impl GraphItem {
    fn record_to_value(&self, record: Option<&Record>) -> f64 {
        match self {
            Self::AverageAirTemperature => record
                .map(|record| record.average_air_temp - KELVIN_CELSIUS)
                .unwrap_or(0.0) as f64,
            Self::AverageSeaTemperature => record
                .map(|record| record.average_sea_temp - KELVIN_CELSIUS)
                .unwrap_or(0.0) as f64,
            Self::AverageRainfall => record
                .map(|record| record.average_rainfall as f64)
                .unwrap_or(0.0),
            Self::Biomass => record.map(|record| record.biomass as f64 * 1e-3).unwrap_or(0.0),
            Self::Oxygen => record.map(|record| record.p_o2 as f64).unwrap_or(0.0),
            Self::Nitrogen => record.map(|record| record.p_n2 as f64).unwrap_or(0.0),
            Self::CarbonDioxide => record.map(|record| record.p_co2 as f64).unwrap_or(0.0),
            Self::BuriedCarbon => record
                .map(|record| record.buried_carbon as f64 / 1000.0)
                .unwrap_or(0.0),
            Self::Population => record.map(|record| record.pop(None) as f64).unwrap_or(0.0),
        }
    }

    fn format_value(&self, value: f64) -> String {
        match self {
            Self::AverageAirTemperature => format!("{:.1} °C", value),
            Self::AverageSeaTemperature => format!("{:.1} °C", value),
            Self::AverageRainfall => format!("{:.0} mm", value),
            Self::Biomass => format!("{:.1} Gt", value),
            Self::Oxygen => format!("{:.2e} atm", value),
            Self::Nitrogen => format!("{:.2e} atm", value),
            Self::CarbonDioxide => format!("{:.2e} atm", value),
            Self::BuriedCarbon => format!("{:.1} Gt", value),
            Self::Population => format!("{:.0}", value),
        }
    }

    fn icon(&self) -> &str {
        match self {
            Self::AverageAirTemperature => "ui/icon-air-temperature",
            Self::AverageSeaTemperature => "ui/icon-sea-temperature",
            Self::AverageRainfall => "ui/icon-rainfall",
            Self::Biomass => "ui/icon-biomass",
            Self::Oxygen => "ui/icon-oxygen",
            Self::Nitrogen => "ui/icon-nitrogen",
            Self::CarbonDioxide => "ui/icon-carbon-dioxide",
            Self::BuriedCarbon => "ui/icon-carbon",
            Self::Population => "ui/icon-population",
        }
    }
}
