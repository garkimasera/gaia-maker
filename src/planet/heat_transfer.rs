use super::misc::linear_interpolation;
use super::*;
use geom::Direction;

/// Stefan-Boltzmann Constant [W/(m2*K4)]
pub const STEFAN_BOLTZMANN_CONSTANT: f32 = 5.670E-8;

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let size = planet.map.size();
    let map_iter_idx = planet.map.iter_idx();
    let atmo_mass_per_tile = planet.atmo.total_mass() / (size.0 as f32 * size.1 as f32);
    let air_heat_cap_per_tile = atmo_mass_per_tile * params.sim.air_heat_cap * 1.0E+9;

    calc_insolation(planet, sim, params);

    // Calculate heat capacity of tiles
    for p in map_iter_idx {
        let surface_heat_cap = match planet.map[p].biome {
            Biome::Ocean => params.sim.sea_heat_cap * params.sim.sea_surface_depth,
            _ => params.sim.land_surface_heat_cap,
        };
        sim.atmo_heat_cap[p] = air_heat_cap_per_tile + surface_heat_cap * sim.tile_area;

        let deep_layer_thickness = deep_sea_layer_thickness(p, planet, params);
        sim.sea_heat_cap[p] = params.sim.sea_heat_cap * deep_layer_thickness * sim.tile_area;
    }

    // Calculate albedo of tiles
    let cloud_albedo =
        linear_interpolation(&params.sim.cloud_albedo_table, planet.atmo.cloud_amount);
    for p in map_iter_idx {
        if planet.map[p]
            .tile_events
            .get(TileEventKind::BlackDust)
            .is_some()
        {
            sim.albedo[p] = params.event.black_dust_albedo;
        } else {
            let tile_albedo = params.biomes[&planet.map[p].biome].albedo;
            sim.albedo[p] = (tile_albedo * tile_albedo + cloud_albedo * cloud_albedo).sqrt();
        }
    }

    let greenhouse_effect = greenhouse_effect(planet, params);

    let secs_per_loop = params.sim.secs_per_cycle / params.sim.n_loop_atmo_heat_calc as f32;

    // Calculate new atmosphere temperature of tiles
    for _ in 0..params.sim.n_loop_atmo_heat_calc {
        for p in map_iter_idx {
            let old_heat_amount = sim.atmo_heat_cap[p] * sim.atemp[p];

            let insolation = sim.insolation[p] * (1.0 - sim.albedo[p]);

            let greenhouse_effect = greenhouse_effect
                * (1.0
                    - params.sim.green_house_effect_height_decrease
                        * planet.height_above_sea_level(p).max(0.0)
                        * planet.atmo.atm())
                .max(0.0);

            let inflow = insolation * sim.tile_area + sim.geothermal_power_per_tile;

            let outflow = STEFAN_BOLTZMANN_CONSTANT
                * sim.atemp[p].powi(4)
                * sim.tile_area
                * (1.0 - greenhouse_effect);

            let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
                .into_iter()
                .map(|dir| {
                    if let Some(adjacent_tile) = sim.convert_p_cyclic(p + dir.as_coords()) {
                        let delta_temp = sim.atemp[adjacent_tile] - sim.atemp[p];
                        0.5 * params.sim.air_diffusion_factor * air_heat_cap_per_tile * delta_temp
                    } else {
                        0.0
                    }
                })
                .sum();

            let structure_heat = if let Some(BuildingEffect::Heater { heat }) =
                planet.working_building_effect(p, params)
            {
                *heat
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

    // Sea heat diffusion
    for p in map_iter_idx {
        sim.stemp[p] = planet.map[p].sea_temp;
    }

    for p in map_iter_idx {
        if sim.sea_heat_cap[p] == 0.0 {
            continue;
        }
        let old_heat_amount = sim.sea_heat_cap[p] * sim.stemp[p];
        let adjacent_tile_flow: f32 = Direction::FOUR_DIRS
            .into_iter()
            .map(|dir| {
                if let Some(adjacent_tile) = sim.convert_p_cyclic(p + dir.as_coords()) {
                    if sim.sea_heat_cap[adjacent_tile] == 0.0 {
                        0.0
                    } else {
                        let delta_temp = sim.stemp[adjacent_tile] - sim.stemp[p];
                        let c = sim.sea_heat_cap[p].min(sim.sea_heat_cap[adjacent_tile]);
                        0.5 * params.sim.sea_diffusion_factor * c * delta_temp
                    }
                } else {
                    0.0
                }
            })
            .sum();
        let heat_amount = old_heat_amount + adjacent_tile_flow;
        planet.map[p].sea_temp = heat_amount / sim.sea_heat_cap[p];
    }

    // Heat transfer between atmosphere and sea
    let mut sum_sea_temp = 0.0;
    let mut n_sea_tile = 0;
    for p in map_iter_idx {
        let deep_layer_thickness = deep_sea_layer_thickness(p, planet, params);
        if deep_layer_thickness == 0.0 {
            planet.map[p].sea_temp = f32::NAN;
            continue;
        }
        let d = params
            .sim
            .sea_heat_transfer_layer_thickness
            .min(deep_layer_thickness);
        let t_surface = sim.atemp[p];
        let t_deep = planet.map[p].sea_temp;
        let t_deep = if t_deep.is_nan() { t_surface } else { t_deep };
        let delta_q = 0.25 * (t_deep - t_surface) * sim.tile_area * d * params.sim.sea_heat_cap;

        sim.atemp[p] = t_surface + delta_q / sim.atmo_heat_cap[p];

        let sea_temp = t_deep - delta_q / sim.sea_heat_cap[p];
        planet.map[p].sea_temp = sea_temp;
        sum_sea_temp += sea_temp as f64;
        n_sea_tile += 1;
    }

    // Set calculated new temperature
    let mut sum_temp = 0.0;
    for p in map_iter_idx {
        let t = sim.atemp[p];
        planet.map[p].temp = t;
        sum_temp += t as f64;
    }

    planet.stat.average_air_temp = sum_temp as f32 / planet.n_tile() as f32;
    if n_sea_tile > 0 {
        planet.stat.average_sea_temp = sum_sea_temp as f32 / n_sea_tile as f32;
    } else {
        planet.stat.average_sea_temp = KELVIN_CELSIUS;
    }
}

fn deep_sea_layer_thickness(p: Coords, planet: &Planet, params: &Params) -> f32 {
    (-planet.height_above_sea_level(p) - params.sim.sea_surface_depth)
        .min(params.sim.max_deep_sea_layer_thickness)
        .max(0.0)
}

fn greenhouse_effect(planet: &Planet, params: &Params) -> f32 {
    (linear_interpolation(
        &params.sim.co2_green_house_effect_table,
        planet.atmo.partial_pressure(GasKind::CarbonDioxide),
    ) + linear_interpolation(
        &params.sim.cloud_green_house_effect_table,
        planet.atmo.cloud_amount,
    ))
    .clamp(0.0, 1.0)
}

fn calc_insolation(planet: &Planet, sim: &mut Sim, params: &Params) {
    let solar_power = planet.state.solar_power;
    if (sim.solar_constant_before - solar_power).abs() < 1.0e-3 {
        return;
    }
    sim.solar_constant_before = solar_power;

    for p in planet.map.iter_idx() {
        let latitude = planet.calc_longitude_latitude(p).1;
        sim.insolation[p] = solar_power
            * linear_interpolation(&params.sim.latitude_insolation_table, latitude.abs());
    }
}

// Calculate initial temperature at the first simulation
pub fn init_temp(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let greenhouse_effect = greenhouse_effect(planet, params);
    let map_iter_idx = planet.map.iter_idx();

    for p in map_iter_idx {
        let t4 = (1.0 - sim.albedo[p]) * sim.insolation[p]
            / (STEFAN_BOLTZMANN_CONSTANT * (1.0 - greenhouse_effect));
        let t = t4.sqrt().sqrt();
        planet.map[p].temp = t;
        if sim.sea_heat_cap[p] > 0.0 {
            planet.map[p].sea_temp = t;
        }
    }
}
