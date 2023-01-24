use super::*;
use geom::{CyclicMode, Direction};

/// Stefan-Boltzmann Constant [W/(m2*K4)]
pub const STEFAN_BOLTZMANN_CONSTANT: f32 = 5.670E-8;

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

    let greenhouse_effect = greenhouse_effect(planet, params);

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

            let outflow = STEFAN_BOLTZMANN_CONSTANT
                * sim.atemp[p].powi(4)
                * sim.tile_area
                * (1.0 - greenhouse_effect);

            let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
                .into_iter()
                .map(|dir| {
                    if let Some(adjacent_tile) =
                        CyclicMode::X.convert_coords(planet.map.size(), p + dir.as_coords())
                    {
                        let delta_temp = sim.atemp[adjacent_tile] - sim.atemp[p];
                        0.5 * params.sim.air_diffusion_factor * air_heat_cap_per_tile * delta_temp
                    } else {
                        0.0
                    }
                })
                .sum();

            let structure_heat = if let Some(structure_param) =
                params.structures.get(&planet.map[p].structure.kind())
            {
                if let Some(BuildingEffect::Heater { heat }) = structure_param.building.effect {
                    heat
                } else {
                    0.0
                }
            } else {
                0.0
            };

            let heat_amount = old_heat_amount
                + (inflow - outflow) * secs_per_loop
                + adjacent_tile_flow
                + structure_heat;
            sim.atemp_new[p] = heat_amount / sim.atmo_heat_cap[p];
        }
        std::mem::swap(&mut sim.atemp, &mut sim.atemp_new);
    }

    // Set calculated new temprature
    for p in map_iter_idx {
        planet.map[p].temp = sim.atemp[p];
    }
}

fn greenhouse_effect(planet: &Planet, params: &Params) -> f32 {
    interpolation(
        &params.sim.co2_green_house_effect_table,
        planet.atmo.partial_pressure(GasKind::CarbonDioxide),
    )
}

fn interpolation(table: &[(f32, f32)], x: f32) -> f32 {
    assert!(table.len() > 2);
    let first = table.first().unwrap();
    let last = table.last().unwrap();
    if first.0 >= x {
        return first.1;
    } else if last.0 <= x {
        return last.1;
    }

    for i in 0..(table.len() - 1) {
        let x0 = table[i].0;
        let x1 = table[i + 1].0;
        if x0 < x && x <= x1 {
            let y0 = table[i].1;
            let y1 = table[i + 1].1;
            let a = (y1 - y0) / (x1 - x0);
            let b = (x1 * y0 + x0 * y1) / (x1 - x0);
            return a * x + b;
        }
    }

    panic!("invalid input for interpolation")
}
