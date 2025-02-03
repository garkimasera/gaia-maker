use arrayvec::ArrayVec;
use geom::{Array2d, Coords};
use rand::{seq::SliceRandom, Rng};

use super::{defs::*, Planet, Sim};

pub fn cause_plague(planet: &mut Planet, _sim: &mut Sim, params: &Params, p: Coords) {
    let plague_event: &mut PlagueEvent = 'a: {
        for event in planet.events.in_progress_iter_mut(PlanetEventKind::Plague) {
            if let PlanetEvent::Plague(plague_event) = &mut *event {
                break 'a plague_event;
            }
        }
        // Start new plague
        let size = planet.map.size();
        let plague_event = PlagueEvent {
            i: 0,
            start_at: p,
            map: Array2d::new(size.0, size.1, PlagueStatus::None),
        };
        planet
            .events
            .start_event(PlanetEvent::Plague(plague_event), params);
        for event in planet.events.in_progress_iter_mut(PlanetEventKind::Plague) {
            if let PlanetEvent::Plague(plague_event) = &mut *event {
                break 'a plague_event;
            }
        }
        unreachable!();
    };
    let plague_params = &params.event.plague_list[plague_event.i];
    if let Some(Structure::Settlement(settlement)) = planet.map[p].structure {
        plague_event.map[p] = PlagueStatus::Infected {
            target_pop: settlement.pop * (1.0 - plague_params.lethality),
        };
    }
}

/// Simutate plague, return true if the processing plague is completed
pub fn sim_plague(planet: &mut Planet, sim: &mut Sim, params: &Params) -> bool {
    let plague_event: &mut PlagueEvent = 'a: {
        for event in planet.events.in_progress_iter_mut(PlanetEventKind::Plague) {
            if let PlanetEvent::Plague(plague_event) = &mut *event {
                break 'a plague_event;
            }
        }
        unreachable!();
    };
    let plague_params = &params.event.plague_list[plague_event.i];
    let mut count_infected = 0;

    let mut pop_max_uninfected_settlement = 0.0;
    let mut p_pop_max_uninfected_settlement = None;

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(ref mut settlement)) = planet.map[p].structure else {
            if matches!(planet.map[p].event, Some(TileEvent::Plague)) {
                planet.map[p].event = None;
            }
            continue;
        };

        match plague_event.map[p] {
            PlagueStatus::Infected { target_pop } => {
                count_infected += 1;
                settlement.pop -=
                    (settlement.pop - target_pop / 2.0) * params.event.plague_base_lethality_speed;
                if settlement.pop < target_pop {
                    plague_event.map[p] = PlagueStatus::Cured;
                    continue;
                }

                if planet.map[p].event.is_none() {
                    planet.map[p].event = Some(TileEvent::Plague);
                }

                // Spread plague
                if sim.rng.gen_bool(
                    (params.event.plague_spread_base_probability * plague_params.infectivity)
                        .min(1.0)
                        .into(),
                ) {
                    let mut target_tiles: ArrayVec<(Coords, f32), 8> = ArrayVec::new();
                    for d in geom::CHEBYSHEV_DISTANCE_1_COORDS {
                        if let Some(p_adj) = sim.convert_p_cyclic(p + *d) {
                            if let Some(Structure::Settlement(target_settlement)) =
                                &planet.map[p_adj].structure
                            {
                                target_tiles.push((p_adj, target_settlement.pop));
                            }
                        }
                    }
                    if let Some((p_target, pop)) = target_tiles.choose(&mut sim.rng) {
                        if plague_event.map[*p_target] == PlagueStatus::None {
                            plague_event.map[*p_target] = PlagueStatus::Infected {
                                target_pop: pop * (1.0 - plague_params.lethality),
                            };
                        }
                    }
                }
            }
            PlagueStatus::None => {
                if settlement.age >= CivilizationAge::Industrial
                    && settlement.pop > pop_max_uninfected_settlement
                {
                    pop_max_uninfected_settlement = settlement.pop;
                    p_pop_max_uninfected_settlement = Some(p);
                }
            }
            PlagueStatus::Cured => {
                if matches!(planet.map[p].event, Some(TileEvent::Plague)) {
                    planet.map[p].event = None;
                }
            }
        }
    }

    if count_infected == 0 {
        // Clear remaining plague tile events
        for p in planet.map.iter_idx() {
            if matches!(planet.map[p].event, Some(TileEvent::Plague)) {
                planet.map[p].event = None;
            }
        }
        true
    } else {
        // Spread to distant settlement
        if let Some(p) = p_pop_max_uninfected_settlement {
            if sim.rng.gen_bool(
                (params.event.plague_spread_base_probability * plague_params.distant_infectivity)
                    .min(1.0)
                    .into(),
            ) {
                plague_event.map[p] = PlagueStatus::Infected {
                    target_pop: pop_max_uninfected_settlement * (1.0 - plague_params.lethality),
                };
            }
        }
        false
    }
}
