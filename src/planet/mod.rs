mod action;
mod animal;
mod atmo;
mod biome;
mod buildings;
mod civ;
mod defs;
mod event;
mod heat_transfer;
mod initial_conditions;
mod map_generator;
mod misc;
mod monitoring;
mod msg;
mod resources;
mod serde_with_types;
mod sim;
mod stat;
mod water;

pub mod debug_log;

pub use self::atmo::Atmosphere;
use self::civ::Civs;
pub use self::defs::*;
pub use self::event::*;
pub use self::msg::*;
pub use self::resources::*;
pub use self::sim::Sim;
pub use self::stat::{Record, Stat};
pub use self::water::*;

use fnv::FnvHashMap;
use geom::{Array2d, Coords};
use misc::{get_rng, SymmetricalLinearDist};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::f32::consts::PI;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub structure: Structure,
    pub animal: [Option<Animal>; AnimalSize::LEN],
    pub height: f32,
    pub biomass: f32,
    pub fertility: f32,
    pub temp: f32,
    pub sea_temp: f32,
    pub rainfall: f32,
    pub vapor: f32,
    pub buried_carbon: f32,
    pub ice: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub buildable_structures: BTreeSet<StructureKind>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Rock,
            structure: Structure::None,
            animal: [None, None, None],
            height: 0.0,
            biomass: 0.0,
            fertility: 0.0,
            temp: 300.0,
            sea_temp: 300.0,
            rainfall: 0.0,
            vapor: 0.0,
            buried_carbon: 0.0,
            ice: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub n: u32,
    pub control: BuildingControlValue,
}

impl Building {
    fn enabled(&self) -> u32 {
        match self.control {
            BuildingControlValue::AlwaysEnabled => self.n,
            BuildingControlValue::EnabledNumber(n) => n,
            BuildingControlValue::IncreaseRate(_) => self.n,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug, Serialize, Deserialize)]
pub enum BuildingControlValue {
    #[default]
    AlwaysEnabled,
    EnabledNumber(u32),
    IncreaseRate(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub cycles: u64,
    pub basics: Basics,
    pub state: State,
    pub player: Player,
    pub res: Resources,
    pub map: Array2d<Tile>,
    pub atmo: Atmosphere,
    pub water: Water,
    pub space_buildings: FnvHashMap<SpaceBuildingKind, Building>,
    pub events: Events,
    pub civs: Civs,
    pub stat: Stat,
    pub msgs: MsgHolder,
}

impl Planet {
    pub fn new(start_params: &StartParams, params: &Params) -> Planet {
        let mut map = Array2d::new(start_params.size.0, start_params.size.1, Tile::default());

        let gen_conf = map_generator::GenConf {
            w: start_params.size.0,
            h: start_params.size.1,
            max_height: start_params.difference_in_elevation,
        };
        let height_map = map_generator::generate(gen_conf);
        for (p, height) in height_map.iter_with_idx() {
            map[p].height = *height;
        }

        let mut msgs = MsgHolder::default();
        msgs.append(0, MsgKind::EventStart);

        let mut planet = Planet {
            cycles: 0,
            basics: start_params.basics.clone(),
            state: State::default(),
            player: Player::default(),
            res: Resources::new(start_params),
            map,
            atmo: Atmosphere::new(start_params, params),
            water: Water::new(start_params),
            space_buildings: SpaceBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
            events: Events::default(),
            civs: Civs::default(),
            stat: Stat::new(params),
            msgs,
        };

        for (&kind, &n) in &start_params.space_buildings {
            let building = planet.space_building_mut(kind);
            building.n = n;
            if params.building_attrs(kind).control == BuildingControl::EnabledNumber {
                building.control = BuildingControlValue::EnabledNumber(n);
            }
        }

        for structure_kind in StructureKind::iter() {
            if structure_kind.buildable_by_player() {
                planet.player.buildable_structures.insert(structure_kind);
            }
        }

        // Simulate before start
        let mut sim = Sim::new(&planet);
        sim.before_start = true;
        planet.advance(&mut sim, params);
        heat_transfer::init_temp(&mut planet, &mut sim, params);

        let water_volume = planet.water.water_volume;
        planet.water.water_volume = 0.0;
        for _ in 0..(start_params.cycles_before_start / 2) {
            // Advance without water to accelerate heat transfer calclation
            planet.advance(&mut sim, params);
        }
        planet.water.water_volume = water_volume;
        planet.advance(&mut sim, params);

        for initial_condition in &start_params.initial_conditions {
            initial_conditions::apply_initial_condition(
                &mut planet,
                &mut sim,
                initial_condition.clone(),
                params,
            );
        }

        for _ in 0..(start_params.cycles_before_start / 2) {
            planet.advance(&mut sim, params);
        }

        // Reset
        planet.cycles = 0;
        planet.stat.clear_history();
        planet.res.material = 0.0;
        self::stat::record_stats(&mut planet, params);

        planet
    }

    pub fn advance(&mut self, sim: &mut Sim, params: &Params) {
        self.update(sim, params);
        self.cycles += 1;
        self.res.apply_diff();

        self::buildings::advance(self, sim, params);
        self::heat_transfer::advance(self, sim, params);
        self::atmo::sim_atmosphere(self, params);
        self::water::sim_water(self, sim, params);
        self::biome::sim_biome(self, sim, params);
        self::animal::sim_animal(self, sim, params);
        self::event::advance(self, sim, params);
        self::stat::record_stats(self, params);
        self::monitoring::monitor(self, params);
    }

    /// Update after user action without advance the cycle
    pub fn update(&mut self, sim: &mut Sim, params: &Params) {
        // Variables need to initialize before simulation
        self.res.reset_before_update();
        self.state.solar_power_multiplier = 1.0;

        self::buildings::update(self, sim, params);

        self.state.solar_power = self.basics.solar_constant * self.state.solar_power_multiplier;
    }

    pub fn start_event(&mut self, event: PlanetEvent, sim: &mut Sim, params: &Params) {
        self::event::start_event(self, event, sim, params);
    }

    pub fn n_tile(&self) -> u32 {
        let size = self.map.size();
        size.0 * size.1
    }

    pub fn calc_longitude_latitude<T: Into<Coords>>(&self, coords: T) -> (f32, f32) {
        let coords = coords.into();
        let (nx, ny) = self.map.size();

        let x = (2.0 * coords.0 as f32 + 1.0) / (2.0 * nx as f32); // 0.0 ~ 1.0
        let y = ((2.0 * coords.1 as f32 + 1.0) / (2.0 * ny as f32)) * 2.0 - 1.0; // -1.0 ~ 1.0

        let longtitude = x * 2.0 * PI; // 0.0 ~ 2pi
        let latitude = y.asin(); // -pi/2 ~ pi/2
        (longtitude, latitude)
    }

    pub fn height_above_sea_level(&self, p: Coords) -> f32 {
        self.map[p].height - self.water.sea_level
    }
}

pub fn start_planet_to_start_params(id: &str, params: &Params) -> StartParams {
    let mut rng = get_rng();

    let start_planet = params
        .start_planets
        .iter()
        .find(|start_planet| start_planet.id == id)
        .unwrap();

    let mut atmo = params.default_start_params.atmo.clone();

    *atmo.get_mut(&GasKind::Nitrogen).unwrap() = rng
        .sample(SymmetricalLinearDist::from(start_planet.nitrogen))
        .into();
    *atmo.get_mut(&GasKind::CarbonDioxide).unwrap() = rng
        .sample(SymmetricalLinearDist::from(start_planet.carbon_dioxide))
        .into();

    StartParams {
        basics: Basics {
            solar_constant: floor(
                10.0,
                rng.sample(SymmetricalLinearDist::from(start_planet.solar_constant)),
            ),
            ..params.default_start_params.clone().basics
        },
        difference_in_elevation: rng.sample(SymmetricalLinearDist::from(start_planet.elevation)),
        water_volume: rng.sample(SymmetricalLinearDist::from(start_planet.water_volume)),
        atmo,
        initial_conditions: start_planet.initial_conditions.clone(),
        ..params.default_start_params.clone()
    }
}

fn floor(a: f32, f: f32) -> f32 {
    (f / a).floor() * a
}
