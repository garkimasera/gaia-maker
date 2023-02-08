use bevy::prelude::EventWriter;
use bevy_egui::{egui, EguiContext};

use crate::{
    planet::{Params, PlanetBasics, StartParams},
    sim::ManagePlanet,
};

use super::main_menu::MainMenuState;

#[derive(Clone, Debug)]
pub struct NewPlanetState {
    solar_constant: f32,
}

impl NewPlanetState {
    pub fn new(params: &Params) -> Self {
        NewPlanetState {
            solar_constant: params.new_planet.solar_constant.default,
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
                    .text(t!("solar-constant")),
                );

                ui.separator();

                if ui.button(t!("start")).clicked() {
                    let start_params = StartParams {
                        basics: PlanetBasics {
                            solar_constant: state.new_planet.solar_constant,
                            ..params.default_start_params.clone().basics
                        },
                        ..params.default_start_params.clone()
                    };
                    ew_manage_planet.send(ManagePlanet::New(start_params));
                }
            });
        })
        .unwrap();
}
