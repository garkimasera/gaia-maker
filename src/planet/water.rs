use super::misc::{bisection, linear_interpolation};
use super::*;
use geom::Direction;
use rayon::prelude::*;
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
        if v > 0.0 { v } else { 0.0 }
    }
}

pub fn sim_water(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    update_sea_level(planet, sim, params);
    advance_rainfall_calc(planet, sim, params);
    snow_calc(planet, sim, params);
}

pub fn update_sea_level(planet: &mut Planet, sim: &Sim, params: &Params) {
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

pub fn advance_rainfall_calc(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let map_iter_idx = planet.map.iter_idx();
    let size = planet.map.size();
    let coords_converter = sim.coords_converter();

    // Calculate new vapor amount of tiles
    for _ in 0..params.sim.n_loop_vapor_calc {
        let par_iter = sim.vapor_new.par_iter_mut().enumerate();
        par_iter.for_each(|(i, vapor_new)| {
            let p = Coords::from_index_size(i, size);
            let biome = planet.map[p].biome;
            let building_effect = planet.working_building_effect(p, params);

            if biome == Biome::Ocean {
                *vapor_new =
                    linear_interpolation(&params.sim.ocean_vaporization_table, planet.map[p].temp)
                        / RAINFALL_DURATION;
            } else if let Some(BuildingEffect::Vapor { value }) = building_effect {
                *vapor_new = value / RAINFALL_DURATION;
            } else {
                let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
                    .into_iter()
                    .map(|dir| {
                        if let Some(adjacent_tile) = coords_converter.conv(p + dir.as_coords()) {
                            let diff_height =
                                (planet.map[adjacent_tile].height - planet.map[p].height).max(0.0);
                            let d = params.sim.vapor_diffusion_factor
                                / (1.0
                                    + diff_height
                                        * params.sim.coeff_vapor_diffusion_adjust_by_h_diff);
                            let delta_vapor = (sim.vapor[adjacent_tile] - sim.vapor[p]).max(0.0);
                            0.5 * d * delta_vapor
                        } else {
                            0.0
                        }
                    })
                    .sum();
                let loss = sim.vapor[p]
                    * params.sim.vapor_loss_ratio
                    * (1.0 - params.biomes[&biome].revaporization_ratio);
                *vapor_new = sim.vapor[p] + adjacent_tile_flow - loss;
            }
        });
        std::mem::swap(&mut sim.vapor, &mut sim.vapor_new);
    }

    let mut sum_rainfall = 0.0;

    // Set calculated new rainfall
    for p in map_iter_idx {
        planet.map[p].vapor = sim.vapor[p];
        let rainfall = sim.vapor[p] * RAINFALL_DURATION;
        planet.map[p].rainfall = rainfall;
        sum_rainfall += rainfall as f64;
        let temp = (planet.map[p].temp - KELVIN_CELSIUS).max(0.0);
        sim.humidity[p] = (rainfall
            - params.sim.drying_factors.0 * (temp - params.sim.drying_factors.1))
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
            let a = tile.rainfall + 0.1 * -(t - KELVIN_CELSIUS);
            debug_assert!(a >= 0.0);
            let ice_limit = linear_interpolation(&params.sim.ice_thickness_limit_table, a);
            let mass = tile.rainfall * params.sim.fallen_snow_factor;
            if ice_limit > tile.ice + mass {
                tile.ice += mass;
            } else if ice_limit > tile.ice {
                tile.ice = ice_limit;
            }
        }

        ice_height_sum += tile.ice as f64;
    }

    planet.water.ice_volume = ice_height_sum as f32 * sim.tile_area;
}
