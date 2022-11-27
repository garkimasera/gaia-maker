mod action;
mod atm;
mod defs;
mod resources;
mod sim;

pub use self::defs::*;
pub use self::resources::*;
use fnv::{FnvHashMap, FnvHashSet};
use geom::{Array2d, Coords};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use self::atm::Atmosphere;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub biome: Biome,
    pub structure: Structure,
    pub height: f32,
    pub biomass: f32,
    pub temp: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Player {
    pub buildable_structures: FnvHashSet<StructureKind>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            biome: Biome::Rock,
            structure: Structure::None,
            height: 0.0,
            biomass: 0.0,
            temp: 300.0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Building {
    pub n: u32,
    pub enabled: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub tick: u64,
    pub player: Player,
    pub res: Resources,
    pub map: Array2d<Tile>,
    pub atmo: Atmosphere,
    pub orbit: FnvHashMap<OrbitalBuildingKind, Building>,
    pub star_system: FnvHashMap<StarSystemBuildingKind, Building>,
}

impl Planet {
    pub fn new(w: u32, h: u32, params: &Params) -> Planet {
        let map = Array2d::new(w, h, Tile::default());

        let mut planet = Planet {
            tick: 0,
            player: Player::default(),
            res: Resources::new(&params.start),
            map,
            atmo: Atmosphere::from_params(&params.start),
            orbit: OrbitalBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
            star_system: StarSystemBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
        };

        for (kind, &n) in &params.start.orbital_buildings {
            let building = planet.orbit.get_mut(kind).unwrap();
            building.n = n;
            building.enabled = n;
        }
        for (kind, &n) in &params.start.star_system_buildings {
            let building = planet.star_system.get_mut(kind).unwrap();
            building.n = n;
            building.enabled = n;
        }

        planet
            .player
            .buildable_structures
            .insert(StructureKind::OxygenGenerator);
        planet
            .player
            .buildable_structures
            .insert(StructureKind::FertilizationPlant);

        planet
    }

    pub fn placeable(&self, p: Coords, size: StructureSize) -> bool {
        if !self.map.in_range(p) {
            return false;
        }

        for p in size.occupied_tiles().into_iter() {
            if let Some(tile) = self.map.get(p) {
                if !matches!(tile.structure, Structure::None) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn place(&mut self, p: Coords, size: StructureSize, structure: Structure, params: &Params) {
        assert!(self.placeable(p, size));

        let kind = structure.kind();
        self.map[p].structure = structure;

        for p_rel in size.occupied_tiles().into_iter() {
            self.map[p + p_rel].structure = Structure::Occupied { by: p };
        }

        self.res
            .remove_by_map(&params.structures[&kind].building.cost);
    }
}
