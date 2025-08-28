use arrayvec::ArrayVec;
use geom::Coords;
use rand::{Rng, seq::IndexedRandom};

use super::{Planet, Sim, defs::*};

pub fn cause_plague(planet: &mut Planet, _sim: &mut Sim, params: &Params, p: Coords) {
    let plague_event: &mut PlagueEvent = 'a: {
        for e in planet.events.in_progress_iter_mut() {
            if let PlanetEvent::Plague(plague_event) = &mut e.event {
                break 'a plague_event;
            }
        }
        // Start new plague
        let plague_event = PlagueEvent { i: 0, start_pos: p };
        planet
            .events
            .start_event(PlanetEvent::Plague(plague_event), None);
        for e in planet.events.in_progress_iter_mut() {
            if let PlanetEvent::Plague(plague_event) = &mut e.event {
                break 'a plague_event;
            }
        }
        unreachable!();
    };
    let plague_params = &params.event.plague_list[plague_event.i as usize];
    if let Some(Structure::Settlement(settlement)) = planet.map[p].structure {
        planet.map[p].tile_events.insert(TileEvent::Plague {
            i: plague_event.i,
            cured: false,
            target_pop: settlement.pop * (1.0 - plague_params.lethality),
        });
    }
}

pub fn cause_plague_random(_planet: &mut Planet, _sim: &mut Sim, _params: &Params) {}

/// Simutate plague, return true if the processing plague is completed
pub fn sim_plague(planet: &mut Planet, sim: &mut Sim, params: &Params) -> bool {
    let elapsed_cycles = planet
        .events
        .in_progress_event_cycles(PlanetEventKind::Plague)
        .next()
        .unwrap();
    let plague_event: &mut PlagueEvent = 'a: {
        for e in planet.events.in_progress_iter_mut() {
            if let PlanetEvent::Plague(plague_event) = &mut e.event {
                break 'a plague_event;
            }
        }
        unreachable!();
    };
    let plague_params = &params.event.plague_list[plague_event.i as usize];
    let infection_enabled_by_cycles = elapsed_cycles <= plague_params.infection_limit_cycles;
    let mut count_infected = 0;

    let mut pop_max_uninfected_settlement = 0.0;
    let mut p_pop_max_uninfected_settlement = None;

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            planet.map[p].tile_events.remove(TileEventKind::Plague);
            continue;
        };
        let Some(TileEvent::Plague {
            i,
            cured,
            target_pop,
        }) = planet.map[p]
            .tile_events
            .get_mut(TileEventKind::Plague)
            .copied()
        else {
            if settlement.age >= CivilizationAge::Industrial
                && settlement.pop > pop_max_uninfected_settlement
            {
                pop_max_uninfected_settlement = settlement.pop;
                p_pop_max_uninfected_settlement = Some(p);
            }
            continue;
        };

        if !cured {
            count_infected += 1;
            settlement.pop -=
                (settlement.pop - target_pop / 2.0) * params.event.plague_base_lethality_speed;
            planet.map[p].structure = Some(Structure::Settlement(settlement));

            if settlement.pop < target_pop {
                planet.map[p].tile_events.insert(TileEvent::Plague {
                    i,
                    cured: true,
                    target_pop,
                });
                settlement.change_state_after_bad_event(sim, params);
                planet.map[p].structure = Some(Structure::Settlement(settlement));
                continue;
            }

            // Spread plague
            if infection_enabled_by_cycles
                && sim.rng.random_bool(
                    (params.event.plague_spread_base_prob * plague_params.infectivity)
                        .min(1.0)
                        .into(),
                )
            {
                let mut target_tiles: ArrayVec<(Coords, f32), 8> = ArrayVec::new();
                for d in geom::CHEBYSHEV_DISTANCE_1_COORDS {
                    if let Some(p_adj) = sim.convert_p_cyclic(p + *d)
                        && let Some(Structure::Settlement(target_settlement)) =
                            &planet.map[p_adj].structure
                    {
                        target_tiles.push((p_adj, target_settlement.pop));
                    }
                }
                if let Some((p_target, pop)) = target_tiles.choose(&mut sim.rng) {
                    let tile_events = &mut planet.map[*p_target].tile_events;
                    if !tile_events.contains(TileEventKind::Plague) {
                        tile_events.insert(TileEvent::Plague {
                            i,
                            cured: false,
                            target_pop: pop * (1.0 - plague_params.lethality),
                        });
                    }
                }
            }
        }
    }

    if count_infected == 0 {
        // Clear remaining plague tile events
        for p in planet.map.iter_idx() {
            planet.map[p].tile_events.remove(TileEventKind::Plague);
        }
        true
    } else {
        // Spread to distant settlement
        if infection_enabled_by_cycles
            && let Some(p) = p_pop_max_uninfected_settlement
            && sim.rng.random_bool(
                (params.event.plague_spread_base_prob * plague_params.distant_infectivity)
                    .min(1.0)
                    .into(),
            )
        {
            planet.map[p].tile_events.insert(TileEvent::Plague {
                i: plague_event.i,
                cured: false,
                target_pop: pop_max_uninfected_settlement * (1.0 - plague_params.lethality),
            });
        }
        false
    }
}
