use super::*;

/// Stefan-Boltzmann Constant [W/(m2*K4)]
pub const STEFAN_BOLTZMANN_CONSTANT: f32 = 5.67E-8;

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let size = planet.map.size();
    let map_iter_idx = planet.map.iter_idx();
    let atmo_mass_per_tile = planet.atmo.total_mass() / (size.0 as f32 * size.1 as f32);

    // Calculate heat capacity of tiles
    for p in map_iter_idx {
        sim.atmo_heat_cap[p] = atmo_mass_per_tile * params.sim.air_heat_cap * 1.0E+9
            + params.sim.surface_heat_cap * sim.tile_area;
    }

    // Set temprature for simulation
    for p in map_iter_idx {
        sim.atemp[p] = planet.map[p].temp;
    }

    // Calculate new atmosphere temprature of tiles
    for _ in 0..params.sim.n_loop_atmo_heat_calc {
        for p in map_iter_idx {
            let old_heat_amount = sim.atmo_heat_cap[p] * sim.atemp[p];

            let solar_power =
                planet.basics.solar_constant * planet.calc_longitude_latitude(p).1.cos() * 0.5;
            let inflow = solar_power * sim.tile_area;

            let outflow = STEFAN_BOLTZMANN_CONSTANT * sim.atemp[p].powi(4) * sim.tile_area;

            let heat_amount = old_heat_amount + (inflow - outflow) * params.sim.secs_per_day;
            sim.atemp_new[p] = heat_amount / sim.atmo_heat_cap[p];
        }
        std::mem::swap(&mut sim.atemp, &mut sim.atemp_new);
    }

    // Set calculated new temprature
    for p in map_iter_idx {
        planet.map[p].temp = sim.atemp[p];
    }
}
