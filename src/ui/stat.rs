use std::ops::RangeInclusive;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use egui_plot as plot;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use super::{convert_rect, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    Planet,
    Atmosphere,
    History,
}

pub fn stat_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    params: Res<Params>,
    mut current_panel: Local<Panel>,
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
                    ui.selectable_value(&mut *current_panel, panel, t!(panel.as_ref()));
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Planet => planet_stat(ui, &planet),
                Panel::Atmosphere => atmo_stat(ui, &planet),
                Panel::History => history_stat(ui, &mut current_graph_item, &planet, &params),
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

fn planet_stat(ui: &mut egui::Ui, planet: &Planet) {
    egui::Grid::new("table_planet")
        .striped(true)
        .show(ui, |ui| {
            ui.label(t!("radius"));
            ui.label(format!("{:.0} km", planet.basics.radius / 1000.0));
            ui.end_row();
            ui.label(t!("density"));
            ui.label(format!("{:.1} g/cm³", planet.basics.density / 1000.0));
            ui.end_row();
            ui.label(t!("solar-constant"));
            ui.label(format!(
                "{:.0} W/m² ({:+.0}%)",
                planet.basics.solar_constant,
                (planet.state.solar_power_multiplier - 1.0) * 100.0
            ));
        });
}

fn atmo_stat(ui: &mut egui::Ui, planet: &Planet) {
    ui.label(format!(
        "{}: {:.1} °C",
        t!("average-air-temprature"),
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

    let total_mass = planet.atmo.total_mass();

    egui::Grid::new("table_atmo").striped(true).show(ui, |ui| {
        for gas_kind in GasKind::iter() {
            ui.label(t!(gas_kind.as_ref()));
            ui.label(&format!(
                "{:.2}%",
                planet.atmo.mass(gas_kind) / total_mass * 100.0
            ));
            ui.end_row();
        }
    });
}

fn history_stat(ui: &mut egui::Ui, item: &mut GraphItem, planet: &Planet, params: &Params) {
    egui::ComboBox::from_id_source("graph-items")
        .selected_text(t!(item.as_ref()))
        .show_ui(ui, |ui| {
            for graph_item in GraphItem::iter() {
                ui.selectable_value(item, graph_item, t!(graph_item.as_ref()));
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
    let x_axis_formatter = move |_, _, _range: &RangeInclusive<f64>| "".to_string();
    let bound_margin = (max - min) * 0.08 + 1.0e-5;

    plot::Plot::new("history")
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .label_formatter(label_formatter)
        .x_axis_formatter(x_axis_formatter)
        .show_x(false)
        .show_y(true)
        .auto_bounds_y()
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
    AverageAirTemprature,
    AverageSeaTemprature,
    AverageRainfall,
    Biomass,
    Oxygen,
    Nitrogen,
    CarbonDioxide,
}

impl GraphItem {
    fn record_to_value(&self, record: Option<&Record>) -> f64 {
        match self {
            Self::AverageAirTemprature => record
                .map(|record| record.average_air_temp - KELVIN_CELSIUS)
                .unwrap_or(0.0) as f64,
            Self::AverageSeaTemprature => record
                .map(|record| record.average_sea_temp - KELVIN_CELSIUS)
                .unwrap_or(0.0) as f64,
            Self::AverageRainfall => record
                .map(|record| record.average_rainfall as f64)
                .unwrap_or(0.0),
            Self::Biomass => record
                .map(|record| record.biomass as f64 * 1e-3)
                .unwrap_or(0.0),
            Self::Oxygen => record.map(|record| record.p_o2 as f64).unwrap_or(0.0),
            Self::Nitrogen => record.map(|record| record.p_n2 as f64).unwrap_or(0.0),
            Self::CarbonDioxide => record.map(|record| record.p_co2 as f64).unwrap_or(0.0),
        }
    }

    fn format_value(&self, value: f64) -> String {
        match self {
            Self::AverageAirTemprature => format!("{:.1} °C", value),
            Self::AverageSeaTemprature => format!("{:.1} °C", value),
            Self::AverageRainfall => format!("{:.0} mm", value),
            Self::Biomass => format!("{:.1} Gt", value),
            Self::Oxygen => format!("{:.2e} atm", value),
            Self::Nitrogen => format!("{:.2e} atm", value),
            Self::CarbonDioxide => format!("{:.2e} atm", value),
        }
    }
}
