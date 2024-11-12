use super::misc::linear_interpolation;
use super::*;
use geom::{CyclicMode, Direction};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Water {
    /// Volume of water including ice [m^3]
    pub water_volume: f32,
    /// Sea level [m]
    pub sea_level: f32,
    /// Volume of ice [m^3]
    pub ice_volume: f32,
}

impl Water {
    pub fn new(start_params: &StartParams) -> Self {
        Water {
            water_volume: start_params.water_volume,
            sea_level: 0.0,
            ice_volume: 0.0,
        }
    }

    pub fn sea_water_volume(&self) -> f32 {
        let v = self.water_volume - self.ice_volume;
        if v > 0.0 {
            v
        } else {
            0.0
        }
    }
}

pub fn sim_water(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    planet.water.sea_level = bisection(|x| target_function(planet, sim, x), 0.0, 10000.0, 10, 10.0);

    for p in planet.map.iter_idx() {
        let tile = &mut planet.map[p];

        if tile.height < planet.water.sea_level && tile.biome.is_land() {
            tile.biome = Biome::Ocean;
            tile.sea_temp = tile.temp;
        } else if tile.height >= planet.water.sea_level && tile.biome.is_sea() {
            tile.fertility *= params.sim.change_from_ocean_fertility_factor;
            tile.biome = Biome::Rock;
        }
    }

    advance_rainfall_calc(planet, sim, params);
    snow_calc(planet, sim, params);
}

fn target_function(planet: &Planet, sim: &Sim, assumed_sea_level: f32) -> f32 {
    let mut v = 0.0;

    for p in planet.map.iter_idx() {
        let h = planet.map[p].height;

        if h < assumed_sea_level {
            v += (assumed_sea_level - h) * sim.tile_area;
        }
    }

    v - planet.water.sea_water_volume()
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
            let building_effect = planet.working_building_effect(p, params);

            if biome == Biome::Ocean {
                sim.vapor_new[p] =
                    linear_interpolation(&params.sim.ocean_vaporization_table, planet.map[p].temp)
                        / RAINFALL_DURATION;
            } else if let Some(BuildingEffect::Vapor {
                value,
                additional_water,
            }) = building_effect
            {
                sim.vapor_new[p] = value / RAINFALL_DURATION;
                planet.water.water_volume += additional_water;
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

    let mut sum_rainfall = 0.0;

    // Set calculated new rainfall
    for p in map_iter_idx {
        planet.map[p].vapor = sim.vapor[p];
        let rainfall = sim.vapor[p] * RAINFALL_DURATION;
        planet.map[p].rainfall = rainfall;
        sum_rainfall += rainfall as f64;
        sim.humidity[p] = (rainfall
            - params.sim.humidity_factors.0
                * (planet.map[p].temp - KELVIN_CELSIUS + params.sim.humidity_factors.1))
            .max(0.0);
    }

    planet.stat.average_rainfall = sum_rainfall as f32 / planet.n_tile() as f32;
}

pub fn snow_calc(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut ice_height_sum = 0.0;

    for p in planet.map.iter_idx() {
        let t = planet.map[p].temp;
        let tile = &mut planet.map[p];

        if t > params.sim.ice_melting_temp {
            if tile.ice > 0.0 {
                let d = t - params.sim.ice_melting_temp;
                tile.ice -= params.sim.ice_melting_height_per_temp * d;
                if tile.ice < 0.0 {
                    tile.ice = 0.0;
                }
            }
        } else {
            tile.ice += tile.rainfall * params.sim.fallen_snow_factor;
        }

        ice_height_sum += tile.ice as f64;
    }

    planet.water.ice_volume = ice_height_sum as f32 * sim.tile_area;
}
