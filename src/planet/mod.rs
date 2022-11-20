mod action;
mod defs;
mod sim;

pub use crate::planet::defs::*;
use fnv::FnvHashSet;
use geom::{Array2d, Coords};
use serde::{Deserialize, Serialize};

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
    pub energy: f32,
    pub material: f32,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Planet {
    pub tick: u64,
    pub player: Player,
    pub map: Array2d<Tile>,
}

impl Planet {
    pub fn new(w: u32, h: u32) -> Planet {
        let map = Array2d::new(w, h, Tile::default());

        let mut planet = Planet {
            tick: 0,
            player: Player::default(),
            map,
        };

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

    pub fn place(&mut self, p: Coords, size: StructureSize, structure: Structure) {
        assert!(self.placeable(p, size));

        self.map[p].structure = structure;

        for p_rel in size.occupied_tiles().into_iter() {
            self.map[p + p_rel].structure = Structure::Occupied { by: p };
        }
    }
}
