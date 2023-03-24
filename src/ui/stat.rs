use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
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
}

pub fn stat_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    planet: Res<Planet>,
    mut current_panel: Local<Panel>,
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
            ui.label(format!("{:.0} W/m²", planet.basics.solar_constant));
        });
}

fn atmo_stat(ui: &mut egui::Ui, planet: &Planet) {
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
