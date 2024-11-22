use geom::{CyclicMode, Direction};
use misc::range_to_livability_trapezoid;
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
    let (attr, n) = if let Some(ref mut animal) = planet.map[p].animal[size as usize] {
        (&params.animals[&animal.id], animal.n)
    } else {
        return;
    };

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

    let cap_biomass = (tile.biomass / params.sim.animal_cap_max_biomass).clamp(0.0, 1.0);
    let cap_temp = range_to_livability_trapezoid(attr.temp, 5.0, tile.temp);

    cap_biomass * cap_temp
}

impl AnimalHabitat {
    pub fn match_biome(&self, biome: Biome) -> bool {
        match self {
            Self::Land => biome.is_land(),
            Self::Sea => biome.is_sea(),
            Self::Biomes(biomes) => biomes.contains(&biome),
        }
    }
}
