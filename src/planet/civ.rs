use std::sync::OnceLock;

use arrayvec::ArrayVec;
use geom::Coords;
use num_traits::FromPrimitive;
use rand::{Rng, distr::Distribution, seq::IndexedRandom};

use super::*;

pub type Civs = fnv::FnvHashMap<AnimalId, Civilization>;

const SETTLEMENT_STATE_UPDATE_INTERVAL_CYCLES: u16 = 8;
const SETTLEMENT_RANDOM_EVENT_INTERVAL_CYCLES: u64 = 10;

pub fn sim_civs(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let exodus_civ_id = planet.events.in_exodus_civ();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            continue;
        };
        let animal_id = settlement.id;
        let animal_attr = &params.animals[&animal_id];
        let cr = sim.settlement_cr[p];

        // Delete settlement if the biome is unhabitable for the animal
        if !animal_attr.habitat.match_biome(planet.map[p].biome) {
            planet.map[p].structure = None;
            continue;
        }

        // Energy
        super::civ_energy::process_settlement_energy(planet, sim, p, &mut settlement, params, cr);

        // Skip by exodus
        if exodus_civ_id.is_some_and(|exodus_civ_id| animal_id == exodus_civ_id) {
            let civ_sum_values = sim.civ_sum.get_mut(animal_id);
            civ_sum_values.total_settlement[settlement.age as usize] += 1;
            civ_sum_values.total_pop += settlement.pop as f64;
            continue;
        }

        // Settlement state update
        settlement.since_state_changed = settlement.since_state_changed.saturating_add(1);
        if settlement.since_state_changed % SETTLEMENT_STATE_UPDATE_INTERVAL_CYCLES == 0 {
            update_state(planet, sim, p, &mut settlement, params, animal_attr);
        }

        // Tech exp
        if planet.cycles % params.sim.advance_tech_interval_cycles == 0 {
            tech_exp(&mut settlement, planet, p, params);
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
        let cap = cap.max(1e-10);
        let dn = params.sim.base_pop_growth_speed * settlement.pop * (1.0 - settlement.pop / cap);

        let dn = if dn > 0.0 {
            let can_growth = !settlement_blocked_by_tile_event(&planet.map[p].tile_events);
            if can_growth {
                let control = planet
                    .civs
                    .get(&animal_id)
                    .map(|civ| civ.civ_control.pop_growth)
                    .unwrap_or_default();
                super::misc::apply_control_value(dn, 1.0, control)
            } else {
                0.0
            }
        } else {
            dn
        };
        settlement.pop += dn;

        // Settlement extinction
        if settlement.pop
            < params.sim.settlement_init_pop[settlement.age as usize]
                * params.sim.settlement_extinction_threshold
        {
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
        if let Some(exodus_civ_id) = exodus_civ_id
            && exodus_civ_id == id
        {
            continue;
        }

        if sum_values.total_settlement.iter().copied().sum::<u32>() == 0 && sum_values.n_moving == 0
        {
            let name = planet.civ_name(id);
            if planet.civs.remove(&id).is_some() {
                planet
                    .reports
                    .append(planet.cycles, ReportContent::EventCivExtinct { id, name })
            }
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
    if exodus_civ_id.is_none() {
        super::war::sim_settlement_str(planet, sim, params);
        cause_random_events(planet, sim, params);
    }
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
        && (settlement.biomass_consumption
            > planet.map[p].biomass
                * density_to_mass
                * params.sim.settlement_deserted_by_biomass_factor
            || sim.energy_eff[p] < params.sim.energy_efficiency_required[settlement.age as usize])
    {
        settlement.change_state(SettlementState::Deserted);
        return;
    }
    let biomass_decrease = sim.diff_biomass[p] < params.sim.settlement_biomass_decrease_threshold;
    if settlement.since_state_changed < params.sim.settlement_state_changeable_cycles
        || !(biomass_decrease && settlement.state != SettlementState::Growing)
    {
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
    let change_weight_decrease = CHANGE_WEIGHT.get_or_init(|| {
        params
            .sim
            .settlement_state_change_weight_table_decrease
            .iter()
            .map(|v| WeightedIndex::new(v).expect("invalid settlement_state_change_weight_table"))
            .collect()
    });
    let table = if biomass_decrease {
        change_weight
    } else {
        change_weight_decrease
    };
    let new_state =
        SettlementState::from_usize(table[settlement.state as usize].sample(&mut sim.rng)).unwrap();
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
    let pop_growth_control = planet
        .civs
        .get(&settlement.id)
        .map(|civ| civ.civ_control.pop_growth)
        .unwrap_or_default();
    let normalized_pop =
        (settlement.pop / params.sim.settlement_spread_pop[settlement.age as usize]).min(2.0);
    let prob = (params.sim.base_settlement_spreading_prob * normalized_pop).clamp(0.0, 1.0);
    let prob = super::misc::apply_control_value(prob, 1.0, pop_growth_control);
    if !sim.rng.random_bool(prob.into()) {
        return;
    }
    let mut target_tiles: ArrayVec<Coords, 16> = ArrayVec::new();
    for d in geom::CHEBYSHEV_DISTANCE_2_COORDS {
        let Some(q) = sim.convert_p_cyclic(p + *d) else {
            continue;
        };
        if !animal_attr.habitat.match_biome(planet.map[q].biome)
            || settlement_blocked_by_tile_event(&planet.map[q].tile_events)
        {
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
        } else if let Some(Structure::Settlement(s)) = &mut planet.map[q].structure
            && s.age < settlement.age
            && s.id == settlement.id
            && sim.rng.random_bool(params.sim.technology_propagation_prob)
        {
            s.age = settlement.age;
            s.tech_exp = 0.0;
        }
    }
    if let Some(p_target) = target_tiles.choose(&mut sim.rng) {
        planet.map[*p_target].structure = Some(Structure::Settlement(Settlement {
            pop: params.sim.settlement_init_pop[settlement.age as usize],
            ..*settlement
        }));
    }
}

fn tech_exp(settlement: &mut Settlement, planet: &mut Planet, p: Coords, params: &Params) {
    let age = settlement.age as usize;
    let normalized_pop = settlement.pop / params.sim.settlement_init_pop[age];
    let civ = planet.civs.get_mut(&settlement.id).unwrap();

    let diff = if matches!(
        settlement.state,
        SettlementState::Growing | SettlementState::Stable
    ) && normalized_pop > 1.0
    {
        let total_pop_factor = (civ.total_pop
            / params.sim.tech_exp_total_pop_factor[settlement.age as usize])
            .min(2.0);
        let diff = params.sim.base_tech_exp * normalized_pop.sqrt() * total_pop_factor;
        super::misc::apply_control_value(diff, 1.0, civ.civ_control.tech_development)
    } else {
        -params.sim.tech_exp_declining_speed
    };
    settlement.tech_exp += diff;

    if age < (CivilizationAge::LEN - 1) && settlement.tech_exp > params.sim.tech_exp_evolution[age]
    {
        let new_age = CivilizationAge::from_usize(age + 1).unwrap();
        settlement.age = new_age;
        settlement.tech_exp = 0.0;
        settlement.change_state(SettlementState::Growing);

        // Check this age advance is the first for this civilization
        if civ.most_advanced_age < new_age {
            civ.most_advanced_age = new_age;
            planet.reports.append(
                planet.cycles,
                ReportContent::EventCivAdvance {
                    id: settlement.id,
                    name: planet.civ_name(settlement.id),
                    age: new_age,
                    pos: p,
                },
            );
        }
    } else if age > 0 && settlement.tech_exp < -100.0 {
        settlement.age = CivilizationAge::from_usize(age - 1).unwrap();
        settlement.tech_exp = 0.0;
        settlement.change_state(SettlementState::Stable);
    }
}

fn cause_random_events(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    match planet.cycles % SETTLEMENT_RANDOM_EVENT_INTERVAL_CYCLES {
        1 => spawn_vehicles(planet, sim, params),
        2 => super::plague::cause_plague_random(planet, sim, params),
        3 => super::decadence::cause_decadence_random(planet, sim, params),
        4 => super::war::cause_war_random(planet, sim, params),
        _ => (),
    }

    super::war::spawn_troops(planet, sim, params);
}

fn spawn_vehicles(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let exodus_civ_id = planet.events.in_progress_iter().find_map(|e| {
        if let EventInProgress {
            event: PlanetEvent::Exodus(ExodusEvent { id }),
            ..
        } = e
        {
            Some(*id)
        } else {
            None
        }
    });

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = planet.map[p].structure else {
            continue;
        };

        if let Some(exodus_civ_id) = exodus_civ_id
            && settlement.id == exodus_civ_id
        {
            continue;
        }

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
            if planet.map[p_adj].biome == Biome::Ocean && planet.map[p].biome.is_land() {
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
                    moved_counter: 0,
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
                    moved_counter: 0,
                });
            }
        }
    }
}

pub fn civilize_animal(
    planet: &mut Planet,
    params: &Params,
    pos: Coords,
    animal_id: AnimalId,
    manual: bool,
) {
    if manual {
        planet.reports.append(
            planet.cycles,
            ReportContent::EventCivilized {
                animal: animal_id,
                pos,
            },
        )
    } else {
        planet.reports.append(
            planet.cycles,
            ReportContent::EventAchiveCivilization {
                animal: animal_id,
                pos,
            },
        )
    }
    planet.civs.insert(animal_id, Civilization::default());
    let settlement = Settlement {
        id: animal_id,
        age: CivilizationAge::Stone,
        pop: params.sim.settlement_init_pop[CivilizationAge::Stone as usize],
        ..Default::default()
    };
    for p in tile_geom::SpiralIter::new(pos).take(0xFF) {
        if planet.map.in_range(p) && planet.map[p].structure.is_none() {
            planet.map[p].structure = Some(Structure::Settlement(settlement));
            break;
        }
    }
}

fn settlement_blocked_by_tile_event(tile_events: &TileEvents) -> bool {
    tile_events.list().iter().any(|tile_event| match tile_event {
        TileEvent::Fire
        | TileEvent::BlackDust { .. }
        | TileEvent::War { .. }
        | TileEvent::NuclearExplosion { .. }
        | TileEvent::VolcanicEruption { .. }
        | TileEvent::SolarRay { .. } => true,
        TileEvent::Plague { cured, .. } => !cured,
        _ => false,
    })
}

impl Settlement {
    pub fn change_state(&mut self, new_state: SettlementState) {
        self.state = new_state;
        self.since_state_changed = 0;
    }

    pub fn change_state_after_bad_event(&mut self, sim: &mut Sim, params: &Params) {
        use rand::distr::weighted::WeightedIndex;
        static CHANGE_WEIGHT: OnceLock<Vec<WeightedIndex<u32>>> = OnceLock::new();
        let table = CHANGE_WEIGHT.get_or_init(|| {
            params
                .sim
                .settlement_state_change_weight_after_bad_event
                .iter()
                .map(|v| {
                    WeightedIndex::new(v).expect("invalid settlement_state_change_weight_table")
                })
                .collect()
        });
        let new_state =
            SettlementState::from_usize(table[self.state as usize].sample(&mut sim.rng)).unwrap();
        if new_state != self.state {
            self.change_state(new_state);
        }
    }
}

pub fn civ_name(civs: &Civs, id: AnimalId) -> String {
    if let Some(civ) = civs.get(&id) {
        if let Some(name) = &civ.name {
            name.into()
        } else {
            t!("civ", id)
        }
    } else {
        id.to_string()
    }
}

impl Planet {
    pub fn civ_name(&self, id: AnimalId) -> String {
        civ_name(&self.civs, id)
    }
}

impl Civilization {
    pub fn current_age(&self) -> CivilizationAge {
        let max = self
            .total_settlement
            .iter()
            .enumerate()
            .map(|(age, n)| if *n > 0 { age } else { 0 })
            .max()
            .unwrap_or_default();
        CivilizationAge::from_usize(max).unwrap_or_default()
    }
}
