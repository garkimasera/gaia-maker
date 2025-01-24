use bevy::prelude::EventWriter;
use bevy_egui::{egui, EguiContexts};

use crate::{
    planet::{Basics, GasKind, Params, StartParams},
    sim::ManagePlanet,
};

use super::{
    main_menu::{MainMenuMode, MainMenuState},
    EguiTextures,
};

#[derive(Clone, Debug)]
pub struct NewPlanetState {
    planet: NewPlanetKind,
    name: String,
    solar_constant: f32,
    difference_in_elevation: f32,
    water: f32,
    nitrogen: f32,
    carbon_dioxide: f32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NewPlanetKind {
    Id(String),
    Custom,
}

impl NewPlanetState {
    pub fn new(params: &Params) -> Self {
        NewPlanetState {
            planet: NewPlanetKind::Id(params.start_planets[0].id.clone()),
            name: String::new(),
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
    textures: &EguiTextures,
) {
    egui::Window::new(t!("search-new-planet"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .collapsible(false)
        .min_size((500.0, 400.0))
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                // Planet select panel
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_height(200.0);
                        for planet in &params.start_planets {
                            ui.selectable_value(
                                &mut state.new_planet.planet,
                                NewPlanetKind::Id(planet.id.clone()),
                                t!("planet", planet.id),
                            );
                        }
                        ui.selectable_value(
                            &mut state.new_planet.planet,
                            NewPlanetKind::Custom,
                            t!("planet/custom"),
                        );
                    });

                    ui.separator();

                    ui.vertical(|ui| match &state.new_planet.planet {
                        NewPlanetKind::Id(id) => {
                            planet_desc(ui, id, params, textures);
                        }
                        NewPlanetKind::Custom => {
                            custom(ui, params, state);
                        }
                    });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.label(t!("planet-name"));
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut state.new_planet.name).char_limit(92),
                        );
                    });
                });

                ui.separator();

                if ui.button(t!("cancel")).clicked() {
                    state.mode = MainMenuMode::Menu;
                }

                if ui.button(t!("start")).clicked() {
                    start(&mut ew_manage_planet, params, state);
                }
            });
        })
        .unwrap();
}

fn start(ew_manage_planet: &mut EventWriter<ManagePlanet>, params: &Params, state: &MainMenuState) {
    let mut start_params = match &state.new_planet.planet {
        NewPlanetKind::Id(id) => crate::planet::start_planet_to_start_params(id, params),
        NewPlanetKind::Custom => {
            let mut atmo = params.default_start_params.atmo.clone();
            *atmo.get_mut(&GasKind::Nitrogen).unwrap() =
                (params.custom_planet.nitrogen.max * state.new_planet.nitrogen) as f64 / 100.0;
            *atmo.get_mut(&GasKind::CarbonDioxide).unwrap() =
                (params.custom_planet.carbon_dioxide.max * state.new_planet.carbon_dioxide) as f64
                    / 100.0;

            StartParams {
                basics: Basics {
                    solar_constant: state.new_planet.solar_constant,
                    origin: "custom".into(),
                    ..params.default_start_params.clone().basics
                },
                difference_in_elevation: state.new_planet.difference_in_elevation,
                water_volume: params.custom_planet.water_volume.max * state.new_planet.water
                    / 100.0,
                atmo,
                ..params.default_start_params.clone()
            }
        }
    };

    start_params.basics.name = state.new_planet.name.clone();

    ew_manage_planet.send(ManagePlanet::New(start_params));
}

fn planet_desc(ui: &mut egui::Ui, id: &str, params: &Params, textures: &EguiTextures) {
    use crate::planet::PlanetHabitability;

    let start_planet = params
        .start_planets
        .iter()
        .find(|start_planet| start_planet.id == id)
        .unwrap();
    let color = match start_planet.habitability {
        PlanetHabitability::Ideal => egui::Color32::from_rgb(0x46, 0xCC, 0xFF),
        PlanetHabitability::Adequate => egui::Color32::GREEN,
        PlanetHabitability::Poor => egui::Color32::YELLOW,
        PlanetHabitability::Hostile => egui::Color32::RED,
    };

    ui.horizontal(|ui| {
        ui.image(textures.get(format!("start_planets/{}", id)));
        ui.heading(t!("planet", id));
    });

    ui.horizontal(|ui| {
        ui.label(format!("{}: ", t!("habitability")));
        ui.label(
            egui::RichText::new(t!("habitability", start_planet.habitability))
                .strong()
                .color(color),
        );
    });

    ui.label(t!("planet/desc", id));
}

fn custom(ui: &mut egui::Ui, params: &Params, state: &mut MainMenuState) {
    let npp = &params.custom_planet;
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
}
