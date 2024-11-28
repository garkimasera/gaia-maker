use arrayvec::ArrayVec;
use geom::{CyclicMode, Direction};
use misc::{calc_congestion_rate, range_to_livability_trapezoid};
use rand::{seq::SliceRandom, Rng};

use super::*;

pub fn sim_animal(planet: &mut Planet, _sim: &mut Sim, params: &Params) {
    if planet.cycles % params.sim.animal_sim_interval as u64 != 0 {
        return;
    }

    let mut rng = get_rng();

    for p in planet.map.iter_idx() {
        for size in AnimalSize::iter() {
            process_each_animal(planet, p, size, params, &mut rng);
        }
    }
}

fn process_each_animal(
    planet: &mut Planet,
    p: Coords,
    size: AnimalSize,
    params: &Params,
    rng: &mut impl Rng,
) {
    let (animal_id, attr, n) = if let Some(ref mut animal) = planet.map[p].animal[size as usize] {
        (animal.id.clone(), &params.animals[&animal.id], animal.n)
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
    if rng.gen_bool(prob.into()) {
        let mut target_tiles: ArrayVec<Coords, 8> = ArrayVec::new();
        for d in Direction::EIGHT_DIRS {
            if let Some(p_next) = CyclicMode::X.convert_coords(planet_size, p + d.as_coords()) {
                if planet.map[p_next].animal[size as usize].is_none()
                    && calc_cap(planet, p_next, attr, params)
                        > params.sim.animal_extinction_threshold
                {
                    target_tiles.push(p_next);
                }
            }
        }
        if let Some(p_target) = target_tiles.choose(rng) {
            let animal = planet.map[p].animal[size as usize].as_mut().unwrap();
            animal.n /= 2.0;
            planet.map[*p_target].animal[size as usize] = Some(animal.clone());
        }
    }

    // Random kill by congestion
    let prob = (params.sim.coef_animal_kill_by_congestion_a
        * (cr - params.sim.coef_animal_kill_by_congestion_b))
        .clamp(0.0, 1.0);
    if rng.gen_bool(prob.into()) {
        planet.map[p].animal[size as usize] = None;
        return;
    }

    // Random walk
    if rng.gen_bool(params.sim.animal_move_weight) {
        let dir = *Direction::EIGHT_DIRS.choose(rng).unwrap();
        if let Some(p_dest) = CyclicMode::X.convert_coords(planet.map.size(), p + dir.as_coords()) {
            // If the destination is empty
            if planet.map[p_dest].animal[size as usize].is_none() {
                let cap_dest = calc_cap(planet, p_dest, attr, params);
                let move_probability = cap_dest / (cap + 0.001);
                if (0.0..=1.0).contains(&move_probability) && rng.gen_bool(move_probability.into())
                {
                    planet.map[p_dest].animal[size as usize] =
                        planet.map[p].animal[size as usize].take();
                }
            }
        }
    }
}

fn calc_cap(planet: &Planet, p: Coords, attr: &AnimalAttr, params: &Params) -> f32 {
    let tile = &planet.map[p];

    if !attr.habitat.match_biome(tile.biome) {
        return 0.0;
    }

    let cap_biomass_or_fertility = if attr.habitat != AnimalHabitat::Sea {
        (tile.biomass / params.sim.animal_cap_max_biomass).clamp(0.0, 1.0)
    } else {
        tile.fertility.min(params.sim.animal_cap_max_fertility)
            / params.sim.animal_cap_max_fertility
    };
    let cap_temp = range_to_livability_trapezoid(attr.temp, 5.0, tile.temp);
    let cap_oxygen = range_to_livability_trapezoid(
        params.sim.livable_oxygen_range[attr.size as usize],
        5.0,
        planet.atmo.partial_pressure(GasKind::Oxygen),
    );

    cap_biomass_or_fertility * cap_temp * cap_oxygen
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
