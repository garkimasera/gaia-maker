use arrayvec::ArrayVec;
use geom::Coords;
use num_traits::FromPrimitive;
use rand::{Rng, seq::IndexedRandom};

use super::{Planet, ReportContent, Sim, defs::*, misc::calc_congestion_rate};

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

pub fn sim_civs(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let planet_size = planet.map.size();

    // Update settlement congestion rate
    for p in planet.map.iter_idx() {
        sim.settlement_cr[p] = calc_congestion_rate(p, planet_size, |p| {
            if matches!(planet.map[p].structure, Some(Structure::Settlement { .. })) {
                1.0
            } else {
                0.0
            }
        });
    }

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            continue;
        };
        let animal_id = settlement.id;
        let animal_attr = &params.animals[&settlement.id];
        let cr = sim.settlement_cr[p];

        // Delete settlement if the biome is unhabitable for the animal
        if !animal_attr.habitat.match_biome(planet.map[p].biome) {
            planet.map[p].structure = None;
            continue;
        }

        // Energy
        let resource_availability =
            super::civ_energy::process_settlement_energy(planet, sim, p, &settlement, params, cr);

        // Soil erosion
        planet.map[p].fertility *=
            1.0 - params.sim.soil_erosion_effect_by_settlement[settlement.age as usize];

        // Tech exp
        if planet.cycles % params.sim.advance_tech_interval_cycles == 0 {
            tech_exp(&mut settlement, params);
        }

        // Pop growth & decline
        let civ_temp_bonus = params.sim.civ_temp_bonus[settlement.age as usize];
        let cap_animal =
            super::animal::calc_cap_by_atmo_temp(planet, p, animal_attr, params, civ_temp_bonus);
        let cap = params.sim.settlement_max_pop[settlement.age as usize]
            * cap_animal
            * resource_availability;

        let growth_speed = params.sim.base_pop_growth_speed;
        let ratio = settlement.pop / cap.max(1e-10);
        let dn = growth_speed * ratio * (-ratio + 1.0);

        let can_growth = !planet.map[p]
            .tile_events
            .list()
            .iter()
            .any(growth_blocked_by_tile_event);
        if dn < 0.0 || can_growth {
            settlement.pop += dn;
        }

        // Settlement extinction
        if settlement.pop < params.sim.settlement_extinction_threshold {
            planet.map[p].structure = None;
            continue;
        } else {
            planet.map[p].structure = Some(Structure::Settlement(settlement));
        };

        // Settlement spreading
        if planet.cycles % params.sim.settlement_spread_interval_cycles == 0 {
            spread_settlement(planet, sim, p, &settlement, params, animal_attr);
        }

        debug_assert!(settlement.pop > 0.0, "{}", settlement.pop);
        let civ_sum_values = sim.civ_sum.get_mut(animal_id);
        civ_sum_values.total_settlement[settlement.age as usize] += 1;
        civ_sum_values.total_pop += settlement.pop as f64;
    }

    super::civ_energy::consume_buried_carbon(planet, sim, params);

    for (id, sum_values) in sim.civ_sum.iter() {
        if sum_values.total_settlement.iter().copied().sum::<u32>() == 0 {
            let _ = planet.civs.remove(&id);
            continue;
        }
        let c = planet.civs.entry(id).or_default();
        c.total_settlement = sum_values.total_settlement;
        c.total_pop = sum_values.total_pop as f32;
        for (src, e) in sum_values.total_energy_consumption.iter().enumerate() {
            c.total_energy_consumption[src] = *e as f32;
        }
    }
}

fn spread_settlement(
    planet: &mut Planet,
    sim: &mut Sim,
    p: Coords,
    settlement: &Settlement,
    params: &Params,
    animal_attr: &AnimalAttr,
) {
    let normalized_pop =
        (settlement.pop / params.sim.settlement_spread_pop[settlement.age as usize]).min(2.0);
    let prob = (params.sim.base_settlement_spreading_prob * normalized_pop).clamp(0.0, 1.0);
    if !sim.rng.random_bool(prob.into()) {
        return;
    }
    let mut target_tiles: ArrayVec<Coords, 16> = ArrayVec::new();
    for d in geom::CHEBYSHEV_DISTANCE_2_COORDS {
        let Some(q) = sim.convert_p_cyclic(p + *d) else {
            continue;
        };
        if !animal_attr.habitat.match_biome(planet.map[q].biome) {
            continue;
        }
        if planet.map[q].structure.is_none() {
            let cap_animal = super::animal::calc_cap_by_atmo_temp(
                planet,
                p,
                animal_attr,
                params,
                params.sim.civ_temp_bonus[settlement.age as usize],
            );
            if sim.settlement_cr[q]
                < params.sim.base_settlement_spreading_threshold
                    * (planet.map[q].fertility / 100.0)
                    * cap_animal
            {
                target_tiles.push(q);
            }
        } else if let Some(Structure::Settlement(s)) = &mut planet.map[q].structure {
            if s.age < settlement.age && sim.rng.random_bool(params.sim.technology_propagation_prob)
            {
                s.age = settlement.age;
                s.tech_exp = 0.0;
            }
        }
    }
    if let Some(p_target) = target_tiles.choose(&mut sim.rng) {
        planet.map[*p_target].structure = Some(Structure::Settlement(Settlement {
            pop: params.sim.settlement_init_pop[settlement.age as usize],
            ..*settlement
        }));
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
            age: CivilizationAge::Stone,
            pop: params.sim.settlement_init_pop[CivilizationAge::Stone as usize],
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

            planet.reports.append(
                planet.cycles,
                ReportContent::EventCivilized {
                    animal: animal_id,
                    pos: p_center,
                },
            );
            planet.civs.insert(animal_id, Civilization::default());
        }
    }
}

fn growth_blocked_by_tile_event(tile_event: &TileEvent) -> bool {
    match tile_event {
        TileEvent::Fire | TileEvent::BlackDust { .. } => true,
        TileEvent::Plague { cured, .. } => !cured,
        _ => false,
    }
}

impl Planet {
    pub fn can_civilize(&self, id: AnimalId, params: &Params) -> Result<(), &'static str> {
        let Some(civ) = &params.animals[&id].civ else {
            unreachable!()
        };

        let sum: f32 = self
            .map
            .iter()
            .map(|tile| {
                tile.get_animal(id, params)
                    .map(|animal| animal.n)
                    .unwrap_or_default()
            })
            .sum();
        if sum < params.event.n_animal_to_civilize {
            return Err("animal-insufficient-population");
        }

        if self.res.gene_point < civ.civilize_cost {
            return Err("lack-of-gene-points");
        }

        Ok(())
    }
}
