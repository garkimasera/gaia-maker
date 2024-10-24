use bevy::prelude::EventWriter;
use bevy_egui::{egui, EguiContexts};

use crate::{
    planet::{Basics, GasKind, Params, StartParams},
    sim::ManagePlanet,
};

use super::main_menu::{MainMenuMode, MainMenuState};

#[derive(Clone, Debug)]
pub struct NewPlanetState {
    solar_constant: f32,
    difference_in_elevation: f32,
    water: f32,
    nitrogen: f32,
    carbon_dioxide: f32,
}

impl NewPlanetState {
    pub fn new(params: &Params) -> Self {
        NewPlanetState {
            solar_constant: params.custom_planet.solar_constant.default,
            difference_in_elevation: params.custom_planet.difference_in_elevation.default,
            water: params.custom_planet.water_volume.default_percentage,
            nitrogen: params.custom_planet.nitrogen.default_percentage,
            carbon_dioxide: params.custom_planet.carbon_dioxide.default_percentage,
        }
    }
}

pub fn new_planet(
    egui_ctxs: &mut EguiContexts,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: &Params,
    state: &mut MainMenuState,
) {
    let npp = &params.custom_planet;

    egui::Window::new(t!("search-new-planet"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.label(t!("search-new-planet"));
                ui.separator();

                ui.add(
                    egui::Slider::new(
                        &mut state.new_planet.solar_constant,
                        npp.solar_constant.min..=npp.solar_constant.max,
                    )
                    .text(format!("{} [W/m²]", t!("solar-constant"))),
                );

                ui.add(
                    egui::Slider::new(
                        &mut state.new_planet.difference_in_elevation,
                        npp.difference_in_elevation.min..=npp.difference_in_elevation.max,
                    )
                    .text(format!("{} [m]", t!("difference-in-elevation"))),
                );

                ui.add(
                    egui::Slider::new(&mut state.new_planet.water, 0.0..=100.0)
                        .show_value(false)
                        .text(t!("water")),
                );

                ui.add(
                    egui::Slider::new(&mut state.new_planet.nitrogen, 0.0..=100.0)
                        .show_value(false)
                        .text(t!("nitrogen")),
                );
                ui.add(
                    egui::Slider::new(&mut state.new_planet.carbon_dioxide, 0.0..=100.0)
                        .show_value(false)
                        .text(t!("carbon-dioxide")),
                );

                ui.separator();

                if ui.button(t!("cancel")).clicked() {
                    state.mode = MainMenuMode::Menu;
                }

                if ui.button(t!("start")).clicked() {
                    let mut atmo_mass = params.default_start_params.atmo_mass.clone();
                    *atmo_mass.get_mut(&GasKind::Nitrogen).unwrap() =
                        (params.custom_planet.nitrogen.max * state.new_planet.nitrogen) as f64
                            / 100.0;
                    *atmo_mass.get_mut(&GasKind::CarbonDioxide).unwrap() =
                        (params.custom_planet.carbon_dioxide.max * state.new_planet.carbon_dioxide)
                            as f64
                            / 100.0;

                    let start_params = StartParams {
                        basics: Basics {
                            solar_constant: state.new_planet.solar_constant,
                            ..params.default_start_params.clone().basics
                        },
                        difference_in_elevation: state.new_planet.difference_in_elevation,
                        water_volume: params.custom_planet.water_volume.max
                            * state.new_planet.water
                            / 100.0,
                        atmo_mass,
                        ..params.default_start_params.clone()
                    };
                    ew_manage_planet.send(ManagePlanet::New(start_params));
                }
            });
        })
        .unwrap();
}
