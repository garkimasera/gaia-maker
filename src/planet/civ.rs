use rand::seq::SliceRandom;

use super::{defs::*, Planet, Sim};

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

pub fn sim_civs(planet: &mut Planet, _sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement { mut settlement }) = planet.map[p].structure else {
            continue;
        };
        let animal_attr = &params.animals[&settlement.id];
        let cap_animal = super::animal::calc_cap_without_biomass(planet, p, animal_attr, params);
        let cap = params.sim.settlement_max_pop[settlement.age as usize] * cap_animal;

        // Pop growth & decline
        let growth_speed = params.sim.base_pop_growth_speed;
        let ratio = settlement.pop / cap;
        let dn = growth_speed * ratio * (-ratio + 1.0);
        settlement.pop += dn;

        planet.map[p].structure = Some(Structure::Settlement { settlement });
    }
}

pub fn civilize_animal(planet: &mut Planet, sim: &mut Sim, params: &Params, animal_id: AnimalId) {
    let mut p_max_animal = None;
    let mut n = 0.0;
    let size = params.animals[&animal_id].size;

    for p in planet.map.iter_idx() {
        if let Some(tile_animal) = &planet.map[p].animal[size as usize] {
            if tile_animal.id == animal_id && tile_animal.n > n {
                n = tile_animal.n;
                p_max_animal = Some(p);
            }
        }
    }

    if let Some(p) = p_max_animal {
        planet.map[p].animal[size as usize] = None;

        let settlement = Settlement {
            id: animal_id,
            age: CivilizationAge::StoneAge,
            pop: params.sim.settlement_init_pop[CivilizationAge::StoneAge as usize],
            tech_exp: 0.0,
        };
        let mut p_settlement = None;
        for p in tile_geom::SpiralIter::new(p).take(0xFF) {
            if planet.map.in_range(p) && planet.map[p].structure.is_none() {
                planet.map[p].structure = Some(Structure::Settlement { settlement });
                p_settlement = Some(p);
                break;
            }
        }
        if let Some(p_center) = p_settlement {
            for _ in 0..2 {
                let p = p_center
                    + *tile_geom::CHEBYSHEV_DISTANCE_2_COORDS
                        .choose(&mut sim.rng)
                        .unwrap();
                if planet.map.in_range(p)
                    && planet.map[p].structure.is_none()
                    && params.animals[&animal_id]
                        .habitat
                        .match_biome(planet.map[p].biome)
                {
                    planet.map[p].structure = Some(Structure::Settlement { settlement });
                }
            }
        }

        planet.civs.insert(animal_id, Civilization {});
    }
}
