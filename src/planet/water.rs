use super::misc::linear_interpolation;
use super::*;
use geom::{CyclicMode, Direction};
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

pub fn sim_water(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    planet.water.sea_level = bisection(|x| target_function(planet, sim, x), 0.0, 10000.0, 10, 10.0);

    for p in planet.map.iter_idx() {
        let tile = &mut planet.map[p];

        if tile.height < planet.water.sea_level {
            tile.biome = Biome::Ocean;
        } else if tile.biome == Biome::Ocean {
            tile.fertility *= params.sim.change_from_ocean_fertility_factor;
            tile.biome = Biome::Rock;
        }
    }

    advance_rainfall_calc(planet, sim, params);
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

pub fn advance_rainfall_calc(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let map_iter_idx = planet.map.iter_idx();

    // Calculate new vapor amount of tiles
    for _ in 0..params.sim.n_loop_vapor_calc {
        for p in map_iter_idx {
            let biome = planet.map[p].biome;

            if biome == Biome::Ocean {
                sim.vapor_new[p] = linear_interpolation(
                    &params.sim.ocean_vaporization_table,
                    planet.map[p].temp - KELVIN_CELSIUS,
                ) / RAINFALL_DURATION;
            } else {
                let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
                    .into_iter()
                    .map(|dir| {
                        if let Some(adjacent_tile) =
                            CyclicMode::X.convert_coords(planet.map.size(), p + dir.as_coords())
                        {
                            let delta_vapor = sim.vapor[adjacent_tile] - sim.vapor[p];
                            0.5 * params.sim.vapor_diffusion_factor * delta_vapor
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let loss = sim.vapor[p]
                    * params.sim.vapor_loss_ratio
                    * (1.0 - params.biomes[&biome].revaporization_ratio);
                sim.vapor_new[p] = sim.vapor[p] + adjacent_tile_flow - loss;
            }
        }
        std::mem::swap(&mut sim.vapor, &mut sim.vapor_new);
    }

    // Set calculated new rainfall
    for p in map_iter_idx {
        planet.map[p].vapor = sim.vapor[p];
        planet.map[p].rainfall = sim.vapor[p] * RAINFALL_DURATION;
    }
}
