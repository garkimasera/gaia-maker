use super::*;
use geom::Direction;

/// Stefan-Boltzmann Constant [W/(m2*K4)]
pub const STEFAN_BOLTZMANN_CONSTANT: f32 = 5.67E-8;

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let size = planet.map.size();
    let map_iter_idx = planet.map.iter_idx();
    let atmo_mass_per_tile = planet.atmo.total_mass() / (size.0 as f32 * size.1 as f32);
    let air_heat_cap_per_tile = atmo_mass_per_tile * params.sim.air_heat_cap * 1.0E+9;

    // Calculate heat capacity of tiles
    for p in map_iter_idx {
        sim.atmo_heat_cap[p] = air_heat_cap_per_tile + params.sim.surface_heat_cap * sim.tile_area;
    }

    // Calculate albedo of tiles
    for p in map_iter_idx {
        sim.albedo[p] = 0.3;
    }

    // Set temprature for simulation
    for p in map_iter_idx {
        sim.atemp[p] = planet.map[p].temp;
    }

    let secs_per_loop = params.sim.secs_per_day / params.sim.n_loop_atmo_heat_calc as f32;

    // Calculate new atmosphere temprature of tiles
    for _ in 0..params.sim.n_loop_atmo_heat_calc {
        for p in map_iter_idx {
            let old_heat_amount = sim.atmo_heat_cap[p] * sim.atemp[p];

            let solar_power = planet.basics.solar_constant
                * planet.calc_longitude_latitude(p).1.cos()
                * (1.0 - sim.albedo[p])
                * params.sim.sunlight_day_averaging_factor;

            let inflow = solar_power * sim.tile_area;

            let outflow = STEFAN_BOLTZMANN_CONSTANT * sim.atemp[p].powi(4) * sim.tile_area;

            let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
                .into_iter()
                .map(|dir| {
                    if let Some(adjacent_tile) = planet.cyclic_tile_coords(p + dir.as_coords()) {
                        let delta_temp = sim.atemp[adjacent_tile] - sim.atemp[p];
                        0.5 * params.sim.air_diffusion_factor * air_heat_cap_per_tile * delta_temp
                    } else {
                        0.0
                    }
                })
                .sum();

            let heat_amount =
                old_heat_amount + (inflow - outflow) * secs_per_loop + adjacent_tile_flow;
            sim.atemp_new[p] = heat_amount / sim.atmo_heat_cap[p];
        }
        std::mem::swap(&mut sim.atemp, &mut sim.atemp_new);
    }

    // Set calculated new temprature
    for p in map_iter_idx {
        planet.map[p].temp = sim.atemp[p];
    }
}
