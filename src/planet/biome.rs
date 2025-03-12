use geom::{CDistRangeIter, Direction};
use rand::Rng;

use super::misc::linear_interpolation;
use super::*;

const FERTILITY_MAX: f32 = 100.0;
const FERTILITY_MIN: f32 = 0.0;

pub fn sim_biome(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let map_iter_idx = planet.map.iter_idx();

    // Fertility
    sim.fertility_effect.fill(0.0);

    for p in map_iter_idx {
        if let Some(&BuildingEffect::Fertilize {
            increment,
            max,
            range,
        }) = planet.working_building_effect(p, params)
        {
            for (_, p) in CDistRangeIter::new(p, range as _) {
                if planet.map.in_range(p) && planet.map[p].fertility < max {
                    sim.fertility_effect[p] += increment;
                }
            }
        }
    }

    let nitrogen_factor = linear_interpolation(
        &params.sim.nitrogen_fertility_table,
        planet.atmo.partial_pressure(GasKind::Nitrogen),
    );

    let mut sum_diff = 0.0;

    for p in map_iter_idx {
        let temp_factor =
            linear_interpolation(&params.sim.temperature_fertility_table, planet.map[p].temp);
        let rainfall_factor =
            linear_interpolation(&params.sim.humidity_fertility_table, sim.humidity[p]);
        let max_fertility = 100.0 * temp_factor * rainfall_factor * nitrogen_factor;

        let fertility = planet.map[p].fertility;
        let diff = max_fertility - fertility;

        let diff = if diff > 0.0 {
            let fertility_from_adjacent_tiles = Direction::FOUR_DIRS
                .iter()
                .filter_map(|dir| {
                    let p_adj = sim.convert_p_cyclic(p + dir.as_coords());
                    if let Some(p_adj) = p_adj {
                        if planet.map.in_range(p_adj) {
                            Some((planet.map[p_adj].fertility - fertility).max(0.0))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .sum::<f32>()
                * params.sim.fertility_adjacent_factor;
            let fertility_growth_from_biomass = linear_interpolation(
                &params.sim.fertility_growth_from_biomass_table,
                planet.map[p].biomass,
            );

            fertility_from_adjacent_tiles + fertility_growth_from_biomass
        } else {
            diff * params.sim.fertility_base_decrement
        };

        let sea_effect = if planet.map[p].biome.is_sea() {
            -params.sim.sea_fertility_attenuation_factor * fertility
        } else {
            0.0
        };

        let mut new_fertility = (fertility + diff + sea_effect).clamp(FERTILITY_MIN, FERTILITY_MAX);
        if new_fertility < max_fertility {
            new_fertility = (new_fertility + sim.fertility_effect[p]).min(max_fertility);
        }
        sum_diff += (new_fertility - fertility) as f64;
        planet.map[p].fertility = new_fertility;
    }
    planet.atmo.add(
        GasKind::Nitrogen,
        -sum_diff * params.sim.soil_nitrogen as f64 * sim.tile_area as f64,
    );

    // Biomass
    calc_biomass_consumption_dist_by_settlements(planet, sim);
    let mut sum_biomass = 0.0;
    let mut sum_buried_carbon = 0.0;
    let density_to_mass = sim.biomass_density_to_mass();
    let speed_factor_by_atmo = linear_interpolation(
        &params.sim.biomass_growth_speed_atm_table,
        planet.atmo.atm(),
    )
    .min(linear_interpolation(
        &params.sim.biomass_growth_speed_co2_table,
        planet.atmo.partial_pressure(GasKind::CarbonDioxide),
    ));
    let biomass_to_buried_carbon_ratio = linear_interpolation(
        &params.sim.biomass_to_buried_carbon_ratio_o2_table,
        planet.atmo.partial_pressure(GasKind::Oxygen),
    )
    .min(linear_interpolation(
        &params.sim.biomass_to_buried_carbon_ratio_co2_table,
        planet.atmo.partial_pressure(GasKind::CarbonDioxide),
    ));
    let max_biomass_density_planet_factor = calc_max_biomass_density_planet_factor(planet, params);

    for p in map_iter_idx {
        let max = calc_tile_max_biomass_density(
            planet,
            sim,
            params,
            p,
            max_biomass_density_planet_factor,
        );
        let biomass = &mut planet.map[p].biomass;
        let diff_to_max = max - *biomass;
        let diff = if diff_to_max > 0.0 && speed_factor_by_atmo > 0.0 {
            params.sim.base_biomass_increase_speed * speed_factor_by_atmo
        } else {
            diff_to_max * params.sim.base_biomass_decrease_speed
        };
        let diff = (diff - sim.biomass_consumption[p] / density_to_mass).max(-*biomass);
        sim.diff_biomass[p] = diff;

        let carbon_weight = density_to_mass * diff.abs();
        if diff > 0.0 {
            if planet.atmo.remove_carbon(carbon_weight) {
                *biomass += diff;
            }
            sum_biomass += *biomass as f64;
        } else {
            planet
                .atmo
                .release_carbon(carbon_weight * (1.0 - biomass_to_buried_carbon_ratio));
            *biomass += diff;
            sum_biomass += *biomass as f64;
            planet.map[p].buried_carbon += carbon_weight * biomass_to_buried_carbon_ratio;
        }
        sum_buried_carbon += planet.map[p].buried_carbon as f64;
    }
    planet.stat.sum_biomass = sum_biomass as f32 * density_to_mass;
    planet.stat.sum_buried_carbon = sum_buried_carbon as f32;

    // Biome transistion
    process_biome_transition(planet, sim, params);
}

fn process_biome_transition(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let rng = &mut sim.rng;

    for p in planet.map.iter_idx() {
        let tile = &planet.map[p];
        let current_biome = tile.biome;
        let current_priority = if check_requirements(tile, current_biome, params) {
            params.biomes[&current_biome].priority
        } else {
            0
        };

        if current_biome.is_sea() {
            let sea_ice_temp = params.biomes[&Biome::SeaIce].requirements.temp.1;
            let next_biome = if tile.temp < sea_ice_temp {
                Biome::SeaIce
            } else {
                Biome::Ocean
            };
            if current_biome != next_biome {
                let transition_probability = if sim.before_start {
                    params.sim.before_start_biome_transition_probability
                } else {
                    1.0 / params.biomes[&next_biome].mean_transition_time
                };
                if rng.random_bool(transition_probability as f64) {
                    planet.map[p].biome = next_biome;
                }
            }
            continue;
        }

        if tile.ice >= params.sim.ice_thickness_of_ice_sheet {
            if current_biome != Biome::IceSheet {
                planet.map[p].biome = Biome::IceSheet;
            }
            continue;
        }

        let Some((_, next_biome)) = Biome::iter()
            .filter_map(|biome| {
                let priority = params.biomes[&biome].priority;
                if biome != current_biome
                    && priority > current_priority
                    && check_requirements(tile, biome, params)
                {
                    Some((priority, biome))
                } else {
                    None
                }
            })
            .max_by_key(|(priority, _)| *priority)
        else {
            continue;
        };

        // Specific tile events cause biome transition
        let transition_probability = if planet.map[p].tile_events.get(TileEventKind::Fire).is_some()
        {
            1.0
        } else if sim.before_start {
            params.sim.before_start_biome_transition_probability
        } else {
            1.0 / params.biomes[&next_biome].mean_transition_time
        };
        if rng.random_bool(transition_probability as f64) {
            planet.map[p].biome = next_biome;
        }
    }
}

fn check_requirements(tile: &Tile, biome: Biome, params: &Params) -> bool {
    if biome == Biome::IceSheet && tile.ice <= params.sim.ice_thickness_of_ice_sheet {
        return false;
    }

    let req = &params.biomes[&biome].requirements;

    let temp = tile.temp;

    req.temp.0 <= temp
        && temp <= req.temp.1
        && req.rainfall.0 <= tile.rainfall
        && tile.rainfall <= req.rainfall.1
        && req.fertility <= tile.fertility
        && req.biomass <= tile.biomass
}

fn calc_tile_max_biomass_density(
    planet: &Planet,
    sim: &Sim,
    params: &Params,
    p: Coords,
    planet_factor: f32,
) -> f32 {
    let max_by_fertility = linear_interpolation(
        &params.sim.max_biomass_fertility_table,
        planet.map[p].fertility,
    );
    let max_by_humidity =
        linear_interpolation(&params.sim.max_biomass_humidity_table, sim.humidity[p]);
    let land_or_sea_factor = if planet.map[p].biome.is_land() {
        1.0
    } else {
        params.sim.sea_biomass_factor
    };

    (max_by_fertility * planet_factor * land_or_sea_factor).min(max_by_humidity)
}

fn calc_max_biomass_density_planet_factor(planet: &Planet, params: &Params) -> f32 {
    linear_interpolation(
        &params.sim.max_biomass_factor_o2_table,
        planet.atmo.partial_pressure(GasKind::Oxygen),
    )
}

fn calc_biomass_consumption_dist_by_settlements(planet: &Planet, sim: &mut Sim) {
    sim.biomass_consumption.fill(0.0);

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(Settlement {
            biomass_consumption,
            ..
        })) = planet.map[p].structure
        else {
            continue;
        };

        // Consume biomass from a tile that has maximum biomass
        let mut p_max_biomass = p;
        let mut total_biomass = planet.map[p].biomass;
        let mut max_biomass = total_biomass;
        for p_adj in geom::CHEBYSHEV_DISTANCE_1_COORDS {
            if let Some(p_adj) = sim.convert_p_cyclic(p + *p_adj) {
                if !matches!(planet.map[p_adj].structure, Some(Structure::Settlement(_))) {
                    let biomass = planet.map[p_adj].biomass;
                    if biomass > max_biomass {
                        max_biomass = biomass;
                        total_biomass += biomass;
                        p_max_biomass = p_adj;
                    }
                }
            }
        }
        sim.biomass_consumption[p_max_biomass] += biomass_consumption;
    }
}
