use arrayvec::ArrayVec;
use geom::Direction;
use misc::{calc_congestion_rate, range_to_livability_trapezoid};
use rand::{Rng, seq::IndexedRandom};

use super::*;

impl Tile {
    pub fn largest_animal(&self) -> Option<&Animal> {
        match self.animal {
            [_, _, Some(ref animal)] => Some(animal),
            [_, Some(ref animal), None] => Some(animal),
            [Some(ref animal), None, None] => Some(animal),
            _ => None,
        }
    }
}

pub fn sim_animal(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    if planet.cycles % params.sim.animal_sim_interval as u64 != 0 {
        return;
    }

    for p in planet.map.iter_idx() {
        for size in AnimalSize::iter() {
            process_each_animal(planet, sim, p, size, params);
        }
    }

    // Count
    for n in planet.stat.animals.values_mut() {
        *n = 0.0;
    }

    for p in planet.map.iter_idx() {
        for size in AnimalSize::iter() {
            if let Some(animal) = &planet.map[p].animal[size as usize] {
                *planet.stat.animals.entry(animal.id).or_default() += animal.n;
            }
        }
    }
}

fn process_each_animal(
    planet: &mut Planet,
    sim: &mut Sim,
    p: Coords,
    size: AnimalSize,
    params: &Params,
) {
    let (animal_id, attr, n) = if let Some(ref mut animal) = planet.map[p].animal[size as usize] {
        (animal.id, &params.animals[&animal.id], animal.n)
    } else {
        return;
    };
    let planet_size = planet.map.size();

    // Animal growth
    let growth_speed = params.sim.animal_growth_speed;
    let cap = calc_cap(planet, p, attr, params);
    let ratio = n / cap;
    let dn = growth_speed * ratio * (-ratio + 1.0);
    let dn = dn.clamp(
        -params.sim.animal_growth_speed_max,
        params.sim.animal_growth_speed_max,
    );
    let new_n = (n + dn).min(1.0);

    if new_n < params.sim.animal_extinction_threshold {
        planet.map[p].animal[size as usize] = None;
        return;
    }

    planet.map[p].animal[size as usize].as_mut().unwrap().n = new_n;

    // Fission
    let cr = calc_congestion_rate(p, planet_size, |p| {
        if let Some(other_animal) = &planet.map[p].animal[size as usize] {
            if animal_id == other_animal.id {
                1.0
            } else {
                let other_attr = &params.animals[&other_animal.id];
                if attr
                    .habitat
                    .compete_at_biome(&other_attr.habitat, planet.map[p].biome)
                {
                    0.8
                } else {
                    0.0
                }
            }
        } else {
            0.0
        }
    });
    let prob = (params.sim.coef_animal_fisson_a * (params.sim.coef_animal_fisson_b * new_n - cr))
        .clamp(0.0, 1.0);
    if sim.rng.random_bool(prob.into()) {
        let mut target_tiles: ArrayVec<Coords, 8> = ArrayVec::new();
        for d in Direction::EIGHT_DIRS {
            if let Some(p_next) = sim.convert_p_cyclic(p + d.as_coords()) {
                if planet.map[p_next].animal[size as usize].is_none()
                    && calc_cap(planet, p_next, attr, params)
                        > params.sim.animal_extinction_threshold
                {
                    target_tiles.push(p_next);
                }
            }
        }
        if let Some(p_target) = target_tiles.choose(&mut sim.rng) {
            let animal = planet.map[p].animal[size as usize].as_mut().unwrap();
            animal.n /= 2.0;
            planet.map[*p_target].animal[size as usize] = Some(*animal);
        }
    }

    // Evolution
    let animal = planet.map[p].animal[size as usize].as_mut().unwrap();
    animal.evo_exp += animal.n;
    if animal.evo_exp >= params.sim.needed_evo_exp_to_evolve {
        animal.evo_exp = 0.0;
        let evolve_prob = params.sim.base_evolution_prob;
        if sim.rng.random_bool(evolve_prob.into())
            && let Some(evolve_to) = sim.animal_evolution_table.evolve_to(&animal.id, &mut sim.rng)
        {
            for &d in [Coords(0, 0)]
                .iter()
                .chain(geom::CHEBYSHEV_DISTANCE_1_COORDS)
                .chain(geom::CHEBYSHEV_DISTANCE_2_COORDS)
            {
                let Some(p_adj) = sim.convert_p_cyclic(p + d) else {
                    continue;
                };
                let target_attr = &params.animals[&evolve_to];
                let cap = calc_cap(planet, p_adj, target_attr, params);
                if cap > 0.0 {
                    planet.map[p_adj].animal[target_attr.size as usize] = Some(Animal {
                        id: evolve_to,
                        n: 1.0,
                        evo_exp: 0.0,
                    });
                    if !planet.stat.animals.contains_key(&evolve_to) {
                        planet.reports.append(
                            planet.cycles,
                            ReportContent::EventAnimalBorn {
                                animal: evolve_to,
                                pos: p,
                            },
                        );
                    }
                    break;
                }
            }

            return;
        } else if sim.rng.random_bool(calc_civ_prob(planet, attr, params).into()) {
            // Civilize
            if sim.domain[p].is_none() {
                super::civ::civilize_animal(planet, params, p, animal_id, false);
            }
        }
    }

    // Random kill by congestion
    let prob = (params.sim.coef_animal_kill_by_congestion_a
        * (cr - params.sim.coef_animal_kill_by_congestion_b))
        .clamp(0.0, 1.0);
    if sim.rng.random_bool(prob.into()) {
        planet.map[p].animal[size as usize] = None;
        return;
    }

    // Random walk
    if sim.rng.random_bool(params.sim.animal_move_weight) {
        let dir = *Direction::EIGHT_DIRS.choose(&mut sim.rng).unwrap();
        if let Some(p_dest) = sim.convert_p_cyclic(p + dir.as_coords()) {
            // If the destination is empty
            if planet.map[p_dest].animal[size as usize].is_none() {
                let cap_dest = calc_cap(planet, p_dest, attr, params);
                let move_probability = (cap_dest / (cap + 0.001)).clamp(0.0, 1.0);
                if sim.rng.random_bool(move_probability.into()) {
                    planet.map[p_dest].animal[size as usize] =
                        planet.map[p].animal[size as usize].take();
                }
            }
        }
    }
}

pub fn calc_cap_by_atmo_temp(
    planet: &Planet,
    p: Coords,
    attr: &AnimalAttr,
    params: &Params,
    temp_bonus: f32,
) -> f32 {
    let tile = &planet.map[p];

    let cap_temp = range_to_livability_trapezoid(
        (attr.temp.0 - temp_bonus, attr.temp.1 + temp_bonus),
        5.0,
        tile.temp,
    );
    let cap_oxygen = range_to_livability_trapezoid(
        params.sim.livable_oxygen_range[attr.size as usize],
        5.0,
        planet.atmo.partial_pressure(GasKind::Oxygen),
    );

    cap_temp * cap_oxygen
}

fn calc_cap(planet: &Planet, p: Coords, attr: &AnimalAttr, params: &Params) -> f32 {
    let tile = &planet.map[p];

    if !attr.habitat.match_biome(tile.biome) {
        return 0.0;
    }

    let cap_biomass_or_fertility = if tile.biome.is_land() {
        (tile.biomass / params.sim.animal_cap_max_biomass).clamp(0.0, 1.0)
    } else {
        tile.fertility.min(params.sim.animal_cap_max_fertility)
            / params.sim.animal_cap_max_fertility
    };

    let settlement_effect = if matches!(planet.map[p].structure, Some(Structure::Settlement(_))) {
        attr.settlement_effect
    } else {
        1.0
    };

    cap_biomass_or_fertility
        * calc_cap_by_atmo_temp(planet, p, attr, params, 0.0)
        * settlement_effect
}

fn calc_civ_prob(planet: &Planet, attr: &AnimalAttr, params: &Params) -> f32 {
    let mut existing_civ_factor = 1.0;
    for civ in planet.civs.values() {
        existing_civ_factor *=
            params.sim.civ_prob_factor_by_existing_civs[civ.current_age() as usize]
    }

    let biome_factor = if attr.habitat.match_biome(Biome::Ocean) {
        params.sim.civ_prob_aquatic
    } else {
        1.0
    };

    params.sim.base_civ_prob
        * attr.civ_prob
        * biome_factor
        * params.sim.civ_prob_factor_by_size[attr.size as usize]
        * existing_civ_factor
}

impl AnimalHabitat {
    pub fn match_biome(&self, biome: Biome) -> bool {
        match self {
            Self::Land => biome.is_land(),
            Self::Sea => biome.is_sea(),
            Self::Biomes(biomes) => biomes.contains(&biome),
        }
    }

    pub fn compete_at_biome(&self, other: &Self, biome: Biome) -> bool {
        self.match_biome(biome) && other.match_biome(biome)
    }
}
