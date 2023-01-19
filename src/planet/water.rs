use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Water {
    /// Volume of water [m^3]
    pub water_volume: f32,
    /// Sea level [m]
    pub sea_level: f32,
}

impl Water {
    pub fn new(start_params: &StartParams) -> Self {
        Water {
            water_volume: start_params.water_volume,
            sea_level: 0.0,
        }
    }
}

pub fn sim_water(planet: &mut Planet, sim: &mut Sim, _params: &Params) {
    planet.water.sea_level = bisection(|x| target_function(planet, sim, x), 0.0, 10000.0, 10, 10.0);

    for p in planet.map.iter_idx() {
        let tile = &mut planet.map[p];

        if tile.height < planet.water.sea_level {
            tile.biome = Biome::Ocean;
        } else if tile.biome == Biome::Ocean {
            tile.biome = Biome::Rock;
        }
    }
}

fn target_function(planet: &Planet, sim: &Sim, assumed_sea_level: f32) -> f32 {
    let mut v = 0.0;

    for p in planet.map.iter_idx() {
        let h = planet.map[p].height;

        if h < assumed_sea_level {
            v += (assumed_sea_level - h) * sim.tile_area;
        }
    }

    v - planet.water.water_volume
}

fn bisection<F: Fn(f32) -> f32>(
    f: F,
    mut a: f32,
    mut b: f32,
    n_max: usize,
    target_diff: f32,
) -> f32 {
    let mut c = (a + b) / 2.0;

    for _ in 0..n_max {
        if f(c) < 0.0 {
            a = c;
        } else {
            b = c;
        }
        c = (a + b) / 2.0;
        if (b - a) < target_diff * 2.0 {
            return c;
        }
    }
    c
}
