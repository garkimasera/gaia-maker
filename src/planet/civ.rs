use arrayvec::ArrayVec;
use geom::Coords;
use num_traits::FromPrimitive;
use rand::{seq::SliceRandom, Rng};
use strum::IntoEnumIterator;

use super::{defs::*, misc::calc_congestion_rate, Planet, Sim};

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

pub fn sim_civs(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    sim.civ_sum.reset(planet.civs.keys().copied());

    let planet_size = planet.map.size();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            continue;
        };
        let animal_id = settlement.id;
        let animal_attr = &params.animals[&settlement.id];

        // Delete settlement if the biome is unhabitable for the animal
        if !animal_attr.habitat.match_biome(planet.map[p].biome) {
            planet.map[p].structure = None;
            continue;
        }

        // Caclulate settlement congestion rate
        let cr = calc_congestion_rate(p, planet_size, |p| {
            if matches!(planet.map[p].structure, Some(Structure::Settlement { .. })) {
                1.0
            } else {
                0.0
            }
        });

        // Energy
        let resource_availability = consume_energy(planet, sim, p, &settlement, params, cr);
        super::debug::tile_log(p, "ra", |_| resource_availability);

        // Tech exp
        tech_exp(&mut settlement, params);

        // Pop growth & decline
        let civ_temp_bonus = params.sim.civ_temp_bonus[settlement.age as usize];
        let cap_animal =
            super::animal::calc_cap_without_biomass(planet, p, animal_attr, params, civ_temp_bonus);
        let cap = params.sim.settlement_max_pop[settlement.age as usize]
            * cap_animal
            * resource_availability;

        let growth_speed = params.sim.base_pop_growth_speed;
        let ratio = settlement.pop / cap.max(1e-10);
        let dn = growth_speed * ratio * (-ratio + 1.0);
        settlement.pop += dn;

        // Settlement extinction
        if settlement.pop < params.sim.settlement_extinction_threshold {
            planet.map[p].structure = None;
            continue;
        } else {
            planet.map[p].structure = Some(Structure::Settlement(settlement));
        };

        // Settlement spreading
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

        debug_assert!(settlement.pop > 0.0, "{}", settlement.pop);
        let civ_sum_values = sim.civ_sum.get_mut(animal_id);
        civ_sum_values.total_settlement[settlement.age as usize] += 1;
        civ_sum_values.total_pop += settlement.pop as f64;
    }

    for (id, sum_values) in sim.civ_sum.iter() {
        if sum_values.total_settlement.iter().copied().sum::<u32>() == 0 {
            let _ = planet.civs.remove(id);
            continue;
        }
        let c = planet.civs.entry(*id).or_default();
        c.total_settlement = sum_values.total_settlement;
        c.total_pop = sum_values.total_pop as f32;
        for (src, e) in sum_values.total_energy_consumption.iter().enumerate() {
            c.total_energy_consumption[src] = *e as f32;
        }
    }
}

fn consume_energy(
    planet: &mut Planet,
    sim: &mut Sim,
    p: Coords,
    settlement: &Settlement,
    params: &Params,
    cr: f32,
) -> f32 {
    let age = settlement.age as usize;
    let animal_id = settlement.id;

    let demand = settlement.pop * params.sim.energy_demand_per_pop[age];
    let mut supply = [0.0; EnergySource::LEN];
    let mut consume = [0.0; EnergySource::LEN];

    // Calculate sparse energy supply
    let mut surrounding_wind_solar = 0.0;
    let mut surrounding_hydro_geothermal = 0.0;

    for p_adj in geom::CHEBYSHEV_DISTANCE_1_COORDS {
        if let Some(p_adj) = sim.convert_p_cyclic(p + *p_adj) {
            if !matches!(planet.map[p_adj].structure, Some(Structure::Settlement(_))) {
                surrounding_wind_solar += sim.energy_wind_solar;
                surrounding_hydro_geothermal += sim.energy_hydro_geothermal[p_adj];
            }
        }
    }
    supply[EnergySource::WindSolar as usize] =
        surrounding_wind_solar * (1.0 - cr) + sim.energy_wind_solar;
    supply[EnergySource::HydroGeothermal as usize] =
        surrounding_hydro_geothermal * (1.0 - cr) + sim.energy_hydro_geothermal[p];

    // Calculate energy distribution
    let priority = [
        EnergySource::Gift,
        EnergySource::HydroGeothermal,
        EnergySource::Nuclear,
        EnergySource::WindSolar,
        EnergySource::FossilFuel,
    ];
    let mut remaining = demand;
    for src in priority {
        let src = src as usize;
        debug_assert!(supply[src] >= 0.0);
        consume[src] += (demand * params.sim.energy_source_limit_by_age[age][src])
            .min(supply[src])
            .min(remaining);
        remaining -= consume[src];
    }
    consume[EnergySource::Biomass as usize] = remaining;

    // Add minimum required or waste energy consume
    for src in EnergySource::iter() {
        let src = src as usize;
        let req = demand * params.sim.energy_source_min_by_age[age][src];
        let supply = supply[src] - consume[src];
        if src == 0 || supply > req {
            consume[src] += req;
        } else {
            consume[src] += supply.max(0.0);
        }
    }

    // Record
    let sum_values = sim.civ_sum.get_mut(animal_id);
    for src in EnergySource::iter() {
        sum_values.total_energy_consumption[src as usize] += consume[src as usize] as f64;
    }

    // Consume biomass from a tile that has maximum biomass
    let impact_on_biomass: f32 = params
        .sim
        .energy_source_biomass_impact
        .iter()
        .enumerate()
        .map(|(src, a)| a * consume[src])
        .sum();
    if impact_on_biomass <= 0.0 {
        return 1.0;
    }
    let biomass_to_consume = impact_on_biomass / params.sim.biomass_energy_factor;
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

    // Decrease biomass
    let total_biomass = total_biomass * sim.biomass_density_to_mass();
    let max_biomass = max_biomass * sim.biomass_density_to_mass();
    let available_biomass_ratio = if biomass_to_consume > 0.0 {
        total_biomass / biomass_to_consume
    } else {
        return 1.0;
    };

    let new_biomass = (max_biomass - biomass_to_consume).max(0.0);
    let diff_biomass = max_biomass - new_biomass;
    planet.map[p_max_biomass].biomass = new_biomass / sim.biomass_density_to_mass();
    planet.atmo.release_carbon(diff_biomass);

    let x = available_biomass_ratio * params.sim.resource_availability_factor;
    if x < 1.0 {
        x * x
    } else {
        x.min(1.0)
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

        planet.civs.insert(animal_id, Civilization::default());
    }
}
