use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use egui_plot as plot;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{OccupiedScreenSpace, WindowsOpenState};
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
    mut current_panel: Local<Panel>,
    mut current_civ_id: Local<Option<AnimalId>>,
    mut current_graph_item: Local<GraphItem>,
) {
    if !wos.stat {
        return;
    }

    let rect = egui::Window::new(t!("statistics"))
        .open(&mut wos.stat)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, t!(panel));
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Planet => planet_stat(
                    ui,
                    &planet,
                    save_state.save_file_metadata.debug_mode_enabled,
                ),
                Panel::Atmosphere => atmo_stat(ui, &planet),
                Panel::Civilization => civ_stat(ui, &planet, &mut current_civ_id),
                Panel::History => history_stat(ui, &mut current_graph_item, &planet, &params),
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn planet_stat(ui: &mut egui::Ui, planet: &Planet, debug_mode_enabled: bool) {
    egui::Grid::new("table_planet").striped(true).show(ui, |ui| {
        ui.label(t!("planet-name"));
        ui.label(&planet.basics.name);
        ui.end_row();
        ui.label(t!("cycles"));
        ui.label(format!("{}", planet.cycles));
        ui.end_row();
        ui.label(t!("radius"));
        ui.label(format!("{:.0} km", planet.basics.radius / 1000.0));
        ui.end_row();
        ui.label(t!("solar-constant"));
        ui.label(format!(
            "{:.0} W/m² ({:+.0}%)",
            planet.basics.solar_constant,
            (planet.state.solar_power_multiplier - 1.0) * 100.0
        ));
        ui.end_row();
        ui.label(t!("biomass"));
        ui.label(format!("{:.1} Gt", planet.stat.sum_biomass * 1e-3));
        ui.end_row();
        let sum_pop: f32 = planet.civs.iter().map(|civ| civ.1.total_pop).sum();
        ui.label(t!("population"));
        ui.label(format!("{:.0}", sum_pop.abs()));
    });

    if debug_mode_enabled {
        ui.label(
            egui::RichText::new("Debug Mode Enabled")
                .color(egui::Color32::from_rgb(0xFF, 0x00, 0xFF)),
        );
    }
}

fn atmo_stat(ui: &mut egui::Ui, planet: &Planet) {
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

    egui::Grid::new("grid_atmo").striped(true).show(ui, |ui| {
        for gas_kind in GasKind::iter() {
            ui.label(t!(gas_kind));
            ui.label(format!("{:.2}%", planet.atmo.mole_ratio[&gas_kind] * 100.0));
            ui.horizontal(|ui| {
                ui.add_space(8.0);
                ui.label(format!("{:.4} atm", planet.atmo.partial_pressure(gas_kind)));
            });
            ui.end_row();
        }
    });
}

fn civ_stat(ui: &mut egui::Ui, planet: &Planet, current_civ_id: &mut Option<AnimalId>) {
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
        .selected_text(t!("animal", selected_civ_id))
        .show_ui(ui, |ui| {
            for id in &civ_ids {
                ui.selectable_value(&mut selected_civ_id, *id, t!("animal", id));
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
    egui::Grid::new("table_cities").show(ui, |ui| {
        for age in CivilizationAge::iter() {
            ui.label(t!("age", age));
            ui.label(format!("{}", c.total_settlement[age as usize]));
            ui.end_row();
        }
    });
    ui.separator();

    ui.label(t!("energy-consumption"));
    egui::Grid::new("table_civ").show(ui, |ui| {
        let max = c
            .total_energy_consumption
            .iter()
            .map(|e| ordered_float::OrderedFloat::from(*e))
            .max()
            .unwrap()
            .into_inner();
        for src in EnergySource::iter() {
            ui.label(t!("energy_source", src));
            let e = c.total_energy_consumption[src as usize];
            let s = if max < 1000.0 {
                format!("{} GJ", crate::text::format_float_1000(e, 0))
            } else {
                format!("{} PJ", crate::text::format_float_1000(e / 1000.0, 3))
            };
            ui.label(s);
            ui.end_row();
        }
    });
}

fn history_stat(ui: &mut egui::Ui, item: &mut GraphItem, planet: &Planet, params: &Params) {
    egui::ComboBox::from_id_salt("graph-items")
        .selected_text(t!(item))
        .show_ui(ui, |ui| {
            for graph_item in GraphItem::iter() {
                ui.selectable_value(item, graph_item, t!(graph_item));
            }
        });

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
}
