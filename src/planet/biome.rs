use super::{misc::linear_interpolation, *};
use geom::{CDistRangeIter, Direction};
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

        let fertility = planet.map[p].fertility;

        let diff = if temp_factor >= 0.0 && rainfall_factor >= 0.0 {
            let fertility_from_adjacent_tiles = Direction::FOUR_DIRS
                .iter()
                .filter_map(|direction| {
                    let p_adj = p + direction.as_coords();
                    if planet.map.in_range(p_adj) {
                        Some((planet.map[p_adj].fertility - fertility).max(0.0))
                    } else {
                        None
                    }
                })
                .sum::<f32>()
                * params.sim.fertility_adjacent_factor;
            (sim.fertility_effect[p] + fertility_from_adjacent_tiles) * temp_factor
        } else {
            params.sim.fertility_base_decrement * temp_factor.min(rainfall_factor)
        };

        planet.map[p].fertility = (fertility + diff).clamp(FERTILITY_MIN, FERTILITY_MAX);
    }

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

        if current_biome == Biome::Ocean {
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
        && req.fertility.0 <= tile.fertility
        && tile.fertility <= req.fertility.1
}
