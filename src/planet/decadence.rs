use std::collections::HashMap;

use arrayvec::ArrayVec;
use geom::Coords;
use rand::{Rng, seq::IndexedRandom};

use super::{Planet, ReportContent, Sim, defs::*};

pub fn cause_decadence_random(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let events: HashMap<_, _> = planet
        .events
        .in_progress_iter()
        .filter_map(|e| match &e.event {
            PlanetEvent::Decadence(decadence) => Some((decadence.id, decadence.clone())),
            _ => None,
        })
        .collect();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &mut planet.map[p].structure else {
            continue;
        };
        if events.contains_key(&settlement.id) {
            continue;
        }
        let civ_age = planet.civs[&settlement.id].most_advanced_age;

        if civ_age < CivilizationAge::Iron {
            continue;
        }

        if settlement.age == civ_age
            && matches!(
                settlement.state,
                SettlementState::Growing | SettlementState::Stable
            )
            && settlement.since_state_changed > params.sim.settlement_state_changeable_cycles
            && settlement.pop
                > params.sim.settlement_max_pop[civ_age as usize]
                    * params.event.decadence_pop_threshold
            && sim.rng.random_bool(params.event.decadence_prob)
        {
            cause_decadence(planet, sim, params, p);
        }
    }
}

pub fn cause_decadence(planet: &mut Planet, sim: &mut Sim, params: &Params, p: Coords) {
    let Some(Structure::Settlement(settlement)) = planet.map[p].structure else {
        return;
    };

    planet.map[p]
        .tile_events
        .insert(TileEvent::Decadence { cured: false });
    let remaining_cycles = sim
        .rng
        .random_range(params.event.decadence_cycles.0..params.event.decadence_cycles.1);
    let duration = remaining_cycles
        + sim.rng.random_range(
            params.event.decadence_interval_cycles.0..params.event.decadence_interval_cycles.1,
        );

    planet.events.start_event(
        PlanetEvent::Decadence(DecadenceEvent {
            id: settlement.id,
            start_pos: p,
            age: settlement.age,
            remaining_cycles: remaining_cycles as i32,
        }),
        duration as u64,
    );
    let name = planet.civ_name(settlement.id);
    planet.reports.append(
        planet.cycles,
        ReportContent::EventCivDecadence {
            id: settlement.id,
            name,
            pos: p,
        },
    );
}

pub fn sim_decadence(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let events: HashMap<_, _> = planet
        .events
        .in_progress_iter_mut()
        .filter_map(|e| match &mut e.event {
            PlanetEvent::Decadence(decadence) => {
                decadence.remaining_cycles -= 1;
                Some((decadence.id, decadence.clone()))
            }
            _ => None,
        })
        .collect();

    for p in planet.map.iter_idx() {
        let (id, age) = {
            let tile = &mut planet.map[p];
            let Some(TileEvent::Decadence { cured }) =
                tile.tile_events.get_mut(TileEventKind::Decadence)
            else {
                continue;
            };
            if *cured {
                continue;
            }
            let Some(Structure::Settlement(settlement)) = &mut tile.structure else {
                continue;
            };
            let DecadenceEvent {
                remaining_cycles,
                age,
                ..
            } = events[&settlement.id];

            if remaining_cycles <= 0 {
                tile.tile_events.remove(TileEventKind::Decadence);
                continue;
            }

            if settlement.age < age {
                *cured = true;
                continue;
            }

            if matches!(
                settlement.state,
                SettlementState::Growing | SettlementState::Stable
            ) {
                settlement.state = SettlementState::Declining;
            }

            (settlement.id, settlement.age)
        };

        if sim.rng.random_bool(params.event.decadence_infectivity) {
            let mut target_tiles: ArrayVec<Coords, 8> = ArrayVec::new();
            for d in geom::CHEBYSHEV_DISTANCE_1_COORDS {
                if let Some(p_adj) = sim.convert_p_cyclic(p + *d) {
                    if let Some(Structure::Settlement(settlement)) = &planet.map[p_adj].structure {
                        if settlement.id == id
                            && settlement.age == age
                            && planet.map[p_adj]
                                .tile_events
                                .list()
                                .iter()
                                .all(|te| !te.is_settlement_event())
                        {
                            target_tiles.push(p_adj);
                        }
                    }
                }
            }
            if let Some(p_target) = target_tiles.choose(&mut sim.rng) {
                planet.map[*p_target]
                    .tile_events
                    .insert(TileEvent::Decadence { cured: false });
            }
        }
    }
}
