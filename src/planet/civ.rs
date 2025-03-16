use std::sync::OnceLock;

use arrayvec::ArrayVec;
use geom::Coords;
use num_traits::FromPrimitive;
use rand::{Rng, distr::Distribution, seq::IndexedRandom};

use super::{Planet, ReportContent, Sim, defs::*};

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

const SETTLEMENT_STATE_UPDATE_INTERVAL_CYCLES: u16 = 8;

pub fn sim_civs(planet: &mut Planet, sim: &mut Sim, params: &Params) {
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

        // Settlement state update
        settlement.since_state_changed = settlement.since_state_changed.saturating_add(1);
        if settlement.since_state_changed % SETTLEMENT_STATE_UPDATE_INTERVAL_CYCLES == 0 {
            update_state(planet, sim, p, &mut settlement, params, animal_attr);
        }

        // Energy
        super::civ_energy::process_settlement_energy(planet, sim, p, &mut settlement, params, cr);

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
            * sim.planet_area_factor
            * cap_animal;
        let cap = match settlement.state {
            SettlementState::Growing => cap,
            SettlementState::Stable => {
                let fluct = params.sim.settlement_stable_pop_fluctuation;
                settlement.pop.min(cap) * sim.rng.random_range((1.0 - fluct)..(1.0 + fluct))
            }
            SettlementState::Declining | SettlementState::Deserted => {
                settlement.pop
                    * params.sim.pop_factor_by_settlement_state[settlement.state as usize]
            }
        };

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
        if sum_values.total_settlement.iter().copied().sum::<u32>() == 0 && sum_values.n_moving == 0
        {
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

    // Cause settlement random events
    cause_random_events(planet, sim, params);
}

fn update_state(
    planet: &Planet,
    sim: &mut Sim,
    p: Coords,
    settlement: &mut Settlement,
    params: &Params,
    _animal_attr: &AnimalAttr,
) {
    use rand::distr::weighted::WeightedIndex;

    let density_to_mass = sim.biomass_density_to_mass();
    if settlement.state != SettlementState::Deserted
        && settlement.biomass_consumption
            > planet.map[p].biomass
                * density_to_mass
                * params.sim.settlement_deserted_by_biomass_factor
    {
        settlement.change_state(SettlementState::Deserted);
        return;
    }
    if settlement.state == SettlementState::Growing
        && sim.diff_biomass[p] < params.sim.settlement_stop_growing_biomass_threshold
        && sim
            .rng
            .random_bool(params.sim.settlement_stop_growing_biomass_prob as f64)
    {
        settlement.change_state(SettlementState::Stable);
        return;
    }
    if settlement.since_state_changed < params.sim.settlement_state_changeable_cycles {
        return;
    }

    static CHANGE_WEIGHT: OnceLock<Vec<WeightedIndex<u32>>> = OnceLock::new();
    let change_weight = CHANGE_WEIGHT.get_or_init(|| {
        params
            .sim
            .settlement_state_change_weight_table
            .iter()
            .map(|v| WeightedIndex::new(v).expect("invalid settlement_state_change_weight_table"))
            .collect()
    });
    let new_state =
        SettlementState::from_usize(change_weight[settlement.state as usize].sample(&mut sim.rng))
            .unwrap();
    if new_state != settlement.state {
        settlement.change_state(new_state);
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
            if s.age < settlement.age
                && s.id == settlement.id
                && sim.rng.random_bool(params.sim.technology_propagation_prob)
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

    let diff = match settlement.state {
        SettlementState::Growing | SettlementState::Stable => {
            (normalized_pop - 0.5) * params.sim.base_tech_exp
        }
        SettlementState::Declining | SettlementState::Deserted => {
            -params.sim.tech_exp_declining_speed
        }
    };
    settlement.tech_exp += diff;

    if age < (CivilizationAge::LEN - 1) && settlement.tech_exp > params.sim.tech_exp_evolution[age]
    {
        settlement.age = CivilizationAge::from_usize(age + 1).unwrap();
        settlement.tech_exp = 0.0;
        settlement.change_state(SettlementState::Growing);
    } else if age > 0 && settlement.tech_exp < -100.0 {
        settlement.age = CivilizationAge::from_usize(age - 1).unwrap();
        settlement.tech_exp = 0.0;
        settlement.change_state(SettlementState::Stable);
    }
}

fn cause_random_events(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    match planet.cycles % params.event.settlement_random_event_interval_cycles {
        1 => spawn_vehicles(planet, sim, params),
        2 => super::plague::cause_plague_random(planet, sim, params),
        3 => super::war::cause_war_random(planet, sim, params),
        _ => (),
    }
}

fn spawn_vehicles(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = planet.map[p].structure else {
            continue;
        };

        for d in geom::CHEBYSHEV_DISTANCE_1_COORDS {
            let Some(p_adj) = sim.convert_p_cyclic(p + *d) else {
                continue;
            };

            if planet.map[p_adj].tile_events.contains(TileEventKind::Vehicle)
                || !sim.rng.random_bool(params.event.vehicle_spawn_prob as f64)
            {
                continue;
            }

            let dx = match d.0 {
                -1 => -1,
                1 => 1,
                _ => {
                    if sim.rng.random_bool(0.5) {
                        -1
                    } else {
                        1
                    }
                }
            };
            if planet.map[p_adj].biome == Biome::Ocean {
                let kind = if settlement.age >= CivilizationAge::Atomic {
                    if sim.rng.random_bool(0.4) {
                        VehicleKind::AirPlane
                    } else {
                        VehicleKind::Ship
                    }
                } else if settlement.age >= CivilizationAge::Iron {
                    VehicleKind::Ship
                } else {
                    continue;
                };
                planet.map[p_adj].tile_events.insert(TileEvent::Vehicle {
                    kind,
                    id: settlement.id,
                    age: settlement.age,
                    direction: (dx, d.1 as _),
                });
            } else if settlement.age >= CivilizationAge::Atomic
                && planet.map[p_adj].structure.is_none()
                && sim.rng.random_bool(0.2)
            {
                planet.map[p_adj].tile_events.insert(TileEvent::Vehicle {
                    kind: VehicleKind::AirPlane,
                    id: settlement.id,
                    age: settlement.age,
                    direction: (dx, d.1 as _),
                });
            }
        }
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
            ..Default::default()
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
        TileEvent::Fire | TileEvent::BlackDust { .. } | TileEvent::War { .. } => true,
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

impl Settlement {
    fn change_state(&mut self, new_state: SettlementState) {
        self.state = new_state;
        self.since_state_changed = 0;
    }
}

impl Planet {
    pub fn civ_name(&self, id: AnimalId) -> Option<String> {
        if let Some(civ) = self.civs.get(&id) {
            if let Some(name) = &civ.name {
                Some(name.into())
            } else {
                Some(t!("civ", id))
            }
        } else {
            None
        }
    }
}
