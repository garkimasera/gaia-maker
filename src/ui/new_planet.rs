use bevy::prelude::EventWriter;
use bevy_egui::{egui, EguiContext};

use crate::{
    planet::{GasKind, Params, PlanetBasics, StartParams},
    sim::ManagePlanet,
};

use super::main_menu::MainMenuState;

#[derive(Clone, Debug)]
pub struct NewPlanetState {
    solar_constant: f32,
    water: f32,
    nitrogen: f32,
    carbon_dioxide: f32,
}

impl NewPlanetState {
    pub fn new(params: &Params) -> Self {
        NewPlanetState {
            solar_constant: params.new_planet.solar_constant.default,
            water: 0.0,
            nitrogen: 50.0,
            carbon_dioxide: 30.0,
        }
    }
}

pub fn new_planet(
    egui_ctx: &mut EguiContext,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: &Params,
    state: &mut MainMenuState,
) {
    let npp = &params.new_planet;

    egui::Window::new(t!("search-new-planet"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
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

                if ui.button(t!("start")).clicked() {
                    let mut atmo_mass = params.default_start_params.atmo_mass.clone();
                    *atmo_mass.get_mut(&GasKind::Nitrogen).unwrap() =
                        params.new_planet.nitrogen_max * state.new_planet.nitrogen / 100.0;
                    *atmo_mass.get_mut(&GasKind::CarbonDioxide).unwrap() =
                        params.new_planet.carbon_dioxide_max * state.new_planet.carbon_dioxide
                            / 100.0;

                    let start_params = StartParams {
                        basics: PlanetBasics {
                            solar_constant: state.new_planet.solar_constant,
                            ..params.default_start_params.clone().basics
                        },
                        water_volume: params.new_planet.water_volume_max * state.new_planet.water
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
