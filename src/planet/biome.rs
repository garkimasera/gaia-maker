use super::{misc::linear_interpolation, *};
use geom::{CDistRangeIter, CyclicMode, Direction};
use rand::{thread_rng, Rng};

const FERTILITY_MAX: f32 = 100.0;
const FERTILITY_MIN: f32 = 0.0;

pub fn sim_biome(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let map_iter_idx = planet.map.iter_idx();

    // Fertility
    sim.fertility_effect.fill(0.0);

    for p in map_iter_idx {
        if let Some(structure_param) = params.structures.get(&planet.map[p].structure.kind()) {
            if let Some(BuildingEffect::Fertilize {
                increment,
                max,
                range,
            }) = structure_param.building.effect
            {
                for (_, p) in CDistRangeIter::new(p, range as _) {
                    if planet.map.in_range(p) && planet.map[p].fertility < max {
                        sim.fertility_effect[p] += increment;
                    }
                }
            }
        }
    }

    for p in map_iter_idx {
        let temp_factor = linear_interpolation(
            &params.sim.temprature_fertility_table,
            planet.map[p].temp - KELVIN_CELSIUS,
        );
        let rainfall_factor =
            linear_interpolation(&params.sim.rainfall_fertility_table, planet.map[p].rainfall);
        let max_fertility = (temp_factor + rainfall_factor) / 2.0;

        let fertility = planet.map[p].fertility;
        let diff = max_fertility - fertility;

        let diff = if diff > 0.0 {
            let fertility_from_adjacent_tiles = Direction::FOUR_DIRS
                .iter()
                .filter_map(|dir| {
                    let p_adj =
                        CyclicMode::X.convert_coords(planet.map.size(), p + dir.as_coords());
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

        planet.map[p].fertility =
            (fertility + diff + sea_effect).clamp(FERTILITY_MIN, FERTILITY_MAX);

        if planet.map[p].fertility < max_fertility {
            planet.map[p].fertility =
                (planet.map[p].fertility + sim.fertility_effect[p]).min(max_fertility);
        }
    }

    // Biomass
    let speed_factor_by_atmo = 0.5
        * linear_interpolation(&params.sim.biomass_growth_speed_atm_table, planet.atmo.atm)
        + 0.5
            * linear_interpolation(
                &params.sim.biomass_growth_speed_co2_table,
                planet.atmo.partial_pressure(GasKind::CarbonDioxide),
            );

    for p in map_iter_idx {
        let max = calc_max_biomass(planet, params, p);
        let diff = max - planet.map[p].biomass;
        let v = if diff > 0.0 && speed_factor_by_atmo > 0.0 {
            params.sim.base_biomass_increase_speed * speed_factor_by_atmo
        } else {
            diff * params.sim.base_biomass_decrease_speed
        };

        let carbon_weight = sim.tile_area * v * 1.0e-9;
        if v > 0.0 {
            if planet.atmo.remove_carbon(carbon_weight) {
                planet.map[p].biomass += v;
            }
        } else {
            planet.atmo.release_carbon(
                carbon_weight * (1.0 - params.sim.decreased_biomass_to_buried_carbon_ratio),
            );
            planet.map[p].buried_carbon +=
                carbon_weight * params.sim.decreased_biomass_to_buried_carbon_ratio;
        }
    }

    // Biome transistion
    process_biome_transition(planet, params);
}

fn process_biome_transition(planet: &mut Planet, params: &Params) {
    for p in planet.map.iter_idx() {
        let tile = &planet.map[p];
        let current_biome = tile.biome;
        let current_priority = if check_requirements(tile, current_biome, params) {
            params.biomes[&current_biome].priority
        } else {
            0
        };

        if current_biome.is_sea() {
            let sea_ice_temp = params.biomes[&Biome::SeaIce].requirements.temprature.1;
            let next_biome = if tile.temp - KELVIN_CELSIUS < sea_ice_temp {
                Biome::SeaIce
            } else {
                Biome::Ocean
            };
            if current_biome != next_biome {
                let transition_probability =
                    1.0 / params.biomes[&next_biome].mean_transition_time as f64;
                if thread_rng().gen_bool(transition_probability) {
                    planet.map[p].biome = next_biome;
                }
            }
            continue;
        }

        let Some((_, next_biome)) = Biome::iter()
            .filter_map(|biome| {
                let priority = params.biomes[&biome].priority;
                if biome != current_biome && priority > current_priority
                    && check_requirements(tile, biome, params) {
                    Some((priority, biome))
                } else {
                    None
                }
            })
            .max_by_key(|(priority, _)| *priority) else {
                continue;
            };

        let transition_probability = 1.0 / params.biomes[&next_biome].mean_transition_time as f64;
        if thread_rng().gen_bool(transition_probability) {
            planet.map[p].biome = next_biome;
        }
    }
}

fn check_requirements(tile: &Tile, biome: Biome, params: &Params) -> bool {
    let req = &params.biomes[&biome].requirements;

    let temp = tile.temp - KELVIN_CELSIUS;

    req.temprature.0 <= temp
        && temp <= req.temprature.1
        && req.rainfall.0 <= tile.rainfall
        && tile.rainfall <= req.rainfall.1
        && req.fertility <= tile.fertility
        && req.biomass <= tile.biomass
}

fn calc_max_biomass(planet: &Planet, params: &Params, p: Coords) -> f32 {
    linear_interpolation(
        &params.sim.max_biomass_fertility_table,
        planet.map[p].fertility,
    ) * if planet.map[p].biome.is_land() {
        1.0
    } else {
        params.sim.sea_biomass_factor
    }
}
