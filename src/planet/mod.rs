mod action;
mod animal;
mod atmo;
mod biome;
mod buildings;
mod civ;
mod civ_energy;
mod defs;
mod event;
mod heat_transfer;
mod initial_conditions;
mod map_generator;
mod misc;
mod monitoring;
mod msg;
mod new;
mod plague;
mod resources;
mod serde_with_types;
mod sim;
mod stat;
mod tile_event;
mod water;

pub mod debug;

pub use self::atmo::Atmosphere;
use self::civ::Civs;
pub use self::defs::*;
pub use self::event::*;
pub use self::msg::*;
pub use self::resources::*;
pub use self::sim::Sim;
pub use self::stat::{Record, Stat};
pub use self::tile_event::TileEvents;
pub use self::water::*;

use fnv::FnvHashMap;
use geom::{Array2d, Coords};
use misc::SymmetricalLinearDist;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub structure: Option<Structure>,
    pub animal: [Option<Animal>; AnimalSize::LEN],
    pub height: f32,
    /// Biomass density [kg/m2]
    pub biomass: f32,
    /// Tile fertility [%]
    pub fertility: f32,
    /// Air temperature [K]
    pub temp: f32,
    /// Sea temperature [K]
    pub sea_temp: f32,
    pub rainfall: f32,
    pub vapor: f32,
    /// Buried carbon mass [Mt]
    pub buried_carbon: f32,
    pub ice: f32,
    pub tile_events: TileEvents,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Rock,
            structure: None,
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
            tile_events: TileEvents::default(),
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
            BuildingControlValue::IncreaseRate(_) => self.n,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug, Serialize, Deserialize)]
pub enum BuildingControlValue {
    #[default]
    AlwaysEnabled,
    IncreaseRate(i32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub cycles: u64,
    pub basics: Basics,
    pub state: State,
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
    pub fn advance(&mut self, sim: &mut Sim, params: &Params) {
        self.update(sim, params);
        self.cycles += 1;
        self.res.apply_diff();

        self::civ_energy::sim_energy_source(self, sim, params);

        self::tile_event::advance(self, sim, params);
        self::buildings::advance(self, sim, params);
        self::heat_transfer::advance(self, sim, params);
        self::atmo::sim_atmosphere(self, params);
        self::water::sim_water(self, sim, params);
        self::biome::sim_biome(self, sim, params);
        self::animal::sim_animal(self, sim, params);
        self::civ::sim_civs(self, sim, params);
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

        // Add gene point based on planet biomass
        self.res.diff_gene_point =
            (self.stat.sum_biomass / params.sim.coef_gene_point_income).sqrt();
    }

    pub fn start_event(&mut self, event: PlanetEvent, _sim: &mut Sim, params: &Params) {
        self.events.start_event(event, params);
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
    let mut rng = misc::get_rng();

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
            name: "".into(),
            origin: id.into(),
            radius: floor(
                10.0,
                rng.sample(SymmetricalLinearDist::from(start_planet.radius)),
            ) * 1000.0,
            solar_constant: floor(
                10.0,
                rng.sample(SymmetricalLinearDist::from(start_planet.solar_constant)),
            ),
            geothermal_power: start_planet
                .geothermal_power
                .map(|geothermal_power| rng.sample(SymmetricalLinearDist::from(geothermal_power)))
                .unwrap_or(params.default_start_params.basics.geothermal_power),
        },
        difference_in_elevation: rng.sample(SymmetricalLinearDist::from(start_planet.elevation)),
        water_volume: rng.sample(SymmetricalLinearDist::from(start_planet.water_volume)),
        atmo,
        initial_conditions: start_planet.initial_conditions.clone(),
        height_table: start_planet.height_table.clone(),
        target_sea_level: start_planet.target_sea_level,
        height_map: start_planet.height_map.clone(),
        ..params.default_start_params.clone()
    }
}

fn floor(a: f32, f: f32) -> f32 {
    (f / a).floor() * a
}
