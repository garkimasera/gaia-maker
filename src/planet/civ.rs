use arrayvec::ArrayVec;
use geom::Coords;
use num_traits::FromPrimitive;
use rand::{seq::SliceRandom, Rng};

use super::{defs::*, misc::calc_congestion_rate, Planet, Sim};

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

pub fn sim_civs(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let planet_size = planet.map.size();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            continue;
        };
        let animal_attr = &params.animals[&settlement.id];

        // Energy
        let resource_availability = consume_energy(planet, sim, p, &settlement, params);

        // Tech exp
        tech_exp(&mut settlement, params);

        // Pop growth & decline
        let cap_animal = super::animal::calc_cap_without_biomass(planet, p, animal_attr, params);
        let cap = params.sim.settlement_max_pop[settlement.age as usize]
            * cap_animal
            * resource_availability;

        let growth_speed = params.sim.base_pop_growth_speed;
        let ratio = settlement.pop / cap.max(1e-10);
        let dn = growth_speed * ratio * (-ratio + 1.0);
        settlement.pop += dn;

        planet.map[p].structure = Some(Structure::Settlement(settlement));

        // Settlement spreading
        let cr = calc_congestion_rate(p, planet_size, |p| {
            if matches!(planet.map[p].structure, Some(Structure::Settlement { .. })) {
                1.0
            } else {
                0.0
            }
        });
        let normalized_pop =
            (settlement.pop / params.sim.settlement_spread_pop[settlement.age as usize]).min(2.0);
        let prob = (params.sim.coef_settlement_spreading_a
            * (params.sim.coef_settlement_spreading_b * normalized_pop - cr))
            .clamp(0.0, 1.0);
        if sim.rng.gen_bool(prob.into()) {
            let mut target_tiles: ArrayVec<Coords, 16> = ArrayVec::new();
            for d in geom::CHEBYSHEV_DISTANCE_2_COORDS {
                if let Some(p_next) = sim.convert_p_cyclic(p + *d) {
                    if animal_attr.habitat.match_biome(planet.map[p_next].biome)
                        && planet.map[p_next].structure.is_none()
                    {
                        target_tiles.push(p_next);
                    }
                }
            }
            if let Some(p_target) = target_tiles.choose(&mut sim.rng) {
                planet.map[*p_target].structure = Some(Structure::Settlement(Settlement {
                    pop: params.sim.settlement_init_pop[settlement.age as usize],
                    ..settlement
                }));
            }
        }

        // Settlement extinction
        planet.map[p].structure = if settlement.pop < params.sim.settlement_extinction_threshold {
            None
        } else {
            Some(Structure::Settlement(settlement))
        };
    }
}

fn consume_energy(
    planet: &mut Planet,
    sim: &mut Sim,
    p: Coords,
    settlement: &Settlement,
    params: &Params,
) -> f32 {
    let age = settlement.age as usize;

    let energy_demand = settlement.pop * params.sim.energy_demand_per_pop[age];
    let biomass_to_consume = energy_demand / params.sim.biomass_energy_factor;

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

    let total_biomass = total_biomass * sim.biomass_density_to_mass();
    let max_biomass = max_biomass * sim.biomass_density_to_mass();
    let available_biomass_ratio = if biomass_to_consume > 0.0 {
        total_biomass / biomass_to_consume
    } else {
        0.0
    };

    let new_biomass = (max_biomass - biomass_to_consume).max(0.0);
    let diff_biomass = max_biomass - new_biomass;
    planet.map[p_max_biomass].biomass = new_biomass / sim.biomass_density_to_mass();
    planet.atmo.release_carbon(diff_biomass);

    let x = available_biomass_ratio * params.sim.resource_availability_factor;
    if x < 1.0 {
        x * x
    } else {
        ((x - 1.0).sqrt() + 1.0).min(2.0)
    }
}

fn tech_exp(settlement: &mut Settlement, params: &Params) {
    let age = settlement.age as usize;
    let normalized_pop = settlement.pop / params.sim.settlement_init_pop[age];
    settlement.tech_exp += (normalized_pop - 0.5) * params.sim.base_tech_exp;

    if age < (CivilizationAge::LEN - 1) && settlement.tech_exp > params.sim.tech_exp_evolution[age]
    {
        settlement.age = CivilizationAge::from_usize(age + 1).unwrap();
        settlement.tech_exp = 0.0;
    } else if age > 0 && settlement.tech_exp < -100.0 {
        settlement.age = CivilizationAge::from_usize(age - 1).unwrap();
        settlement.tech_exp = 0.0;
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
                planet.map[p].structure = Some(Structure::Settlement(settlement));
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
                    planet.map[p].structure = Some(Structure::Settlement(settlement));
                }
            }
        }

        planet.civs.insert(animal_id, Civilization {});
    }
}
