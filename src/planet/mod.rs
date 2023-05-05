mod action;
mod atmo;
mod biome;
mod buildings;
mod defs;
mod heat_transfer;
mod map_generator;
mod misc;
mod resources;
mod sim;
mod stat;
mod water;

pub mod debug_log;

pub use self::atmo::Atmosphere;
pub use self::defs::*;
pub use self::resources::*;
pub use self::sim::Sim;
pub use self::stat::{Record, Stat};
pub use self::water::*;
use fnv::FnvHashMap;
use geom::{Array2d, Coords};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::f32::consts::PI;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub structure: Structure,
    pub height: f32,
    pub biomass: f32,
    pub fertility: f32,
    pub temp: f32,
    pub sea_temp: f32,
    pub rainfall: f32,
    pub vapor: f32,
    pub buried_carbon: f32,
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
            height: 0.0,
            biomass: 0.0,
            fertility: 0.0,
            temp: 300.0,
            sea_temp: 300.0,
            rainfall: 0.0,
            vapor: 0.0,
            buried_carbon: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub n: u32,
    pub enabled: u32,
    pub working: u32,
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
    pub stat: Stat,
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

        let mut planet = Planet {
            cycles: 0,
            basics: start_params.basics.clone(),
            state: State::default(),
            player: Player::default(),
            res: Resources::new(start_params),
            map,
            atmo: Atmosphere::new(start_params),
            water: Water::new(start_params),
            space_buildings: SpaceBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
            stat: Stat::new(params),
        };

        for (kind, &n) in &start_params.orbital_buildings {
            let building = planet.space_building_mut(*kind);
            building.n = n;
            building.enabled = n;
        }
        for (kind, &n) in &start_params.star_system_buildings {
            let building = planet.space_building_mut(*kind);
            building.n = n;
            building.enabled = n;
        }

        let buildable_structures = &[
            StructureKind::OxygenGenerator,
            StructureKind::NitrogenSprayer,
            StructureKind::CarbonDioxideSprayer,
            StructureKind::Rainmaker,
            StructureKind::FertilizationPlant,
            StructureKind::Heater,
        ];
        for structure_kind in buildable_structures {
            planet.player.buildable_structures.insert(*structure_kind);
        }

        // Simulate before start
        let mut sim = Sim::new(&planet);
        sim.before_start = true;
        for _ in 0..start_params.cycles_before_start {
            planet.advance(&mut sim, params);
        }

        // Reset
        planet.cycles = 0;
        planet.res = Resources::new(start_params);
        planet.stat.clear_history();
        self::stat::update_stats(&mut planet, params);

        planet
    }

    pub fn advance(&mut self, sim: &mut Sim, params: &Params) {
        self.cycles += 1;

        self::buildings::advance(self, sim, params);
        self::heat_transfer::advance(self, sim, params);
        self::atmo::sim_atmosphere(self, params);
        self::water::sim_water(self, sim, params);
        self::biome::sim_biome(self, sim, params);
        self::stat::update_stats(self, params);
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
