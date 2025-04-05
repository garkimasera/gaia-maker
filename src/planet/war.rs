use super::*;

pub fn cause_war_random(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = planet.map[p].structure else {
            continue;
        };

        let prob = params.event.base_civil_war_prob;
        if sim.rng.random_bool(prob as f64) {
            start_civil_war(planet, sim, params, p, settlement);
        }
    }

    for (id_a, id_b) in civ_combinations(&planet.civs) {
        let civ0 = &planet.civs[&id_a];
        let civ1 = &planet.civs[&id_b];
        let age = civ0.current_age().max(civ1.current_age());

        if !sim
            .rng
            .random_bool(params.event.inter_species_war_prob[age as usize])
        {
            continue;
        }
        if planet.events.in_progress_iter().any(|e| {
            if let PlanetEvent::War(WarEvent { kind, .. }) = &e.event {
                *kind == WarKind::InterSpecies(id_a, id_b)
                    || *kind == WarKind::InterSpecies(id_b, id_a)
            } else {
                false
            }
        }) {
            continue;
        }

        // Start inter spieces war
        let duration = sim.rng.random_range(
            params.event.inter_species_war_duration_cycles.0
                ..params.event.inter_species_war_duration_cycles.1,
        );
        let planet_event = WarEvent {
            i: empty_war_id(planet),
            kind: WarKind::InterSpecies(id_a, id_b),
            start_pos: None,
            ceased: false,
        };
        planet
            .events
            .start_event(PlanetEvent::War(planet_event), duration);
        planet.reports.append(
            planet.cycles,
            ReportContent::EventInterSpeciesWar { id_a, id_b },
        );
    }

    if !planet.events.in_progress_iter().any(|e| {
        if let PlanetEvent::War(WarEvent { kind, .. }) = &e.event {
            *kind == WarKind::NuclearWar
        } else {
            false
        }
    }) {
        for civ in planet.civs.values() {
            let prob = params.event.nuclear_war_prob[civ.most_advanced_age as usize];

            if sim.rng.random_bool(prob) {
                // Start nuclear war
                let planet_event = WarEvent {
                    i: empty_war_id(planet),
                    kind: WarKind::NuclearWar,
                    start_pos: None,
                    ceased: false,
                };
                planet.events.start_event(
                    PlanetEvent::War(planet_event),
                    params.event.nuclear_war_interval_cycles,
                );
                planet
                    .reports
                    .append(planet.cycles, ReportContent::EventNuclearWar {});
                break;
            }
        }
    }
}

pub fn sim_war(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    if let Some((e, progress)) = planet.events.in_progress_iter_mut().find_map(|e| {
        if let PlanetEvent::War(event) = &mut e.event {
            if event.kind == WarKind::NuclearWar && !event.ceased {
                Some((event, e.progress))
            } else {
                None
            }
        } else {
            None
        }
    }) {
        if progress > params.event.nuclear_war_duration_cycles {
            e.ceased = true;
        }

        for p in planet.map.iter_idx() {
            let Some(Structure::Settlement(settlement)) = &planet.map[p].structure else {
                continue;
            };

            if settlement.age >= CivilizationAge::Industrial
                && sim.rng.random_bool(params.event.nuclear_war_bomb_prob)
            {
                planet.map[p].tile_events.insert(TileEvent::NuclearExplosion {
                    remaining_cycles: params.event.nuclear_explosion_cycles,
                });
            }
        }
    }
}

pub fn start_civil_war(
    planet: &mut Planet,
    sim: &mut Sim,
    params: &Params,
    p: Coords,
    settlement: Settlement,
) {
    let i = empty_war_id(planet);
    let planet_event = WarEvent {
        i,
        kind: WarKind::CivilWar,
        start_pos: Some(p),
        ceased: false,
    };
    planet.events.start_event(PlanetEvent::War(planet_event), None);

    let region1 = if settlement.age >= CivilizationAge::Iron {
        geom::CHEBYSHEV_DISTANCE_1_COORDS
    } else {
        &[]
    };
    let region2 = if settlement.age >= CivilizationAge::Industrial && sim.rng.random_bool(0.5) {
        geom::CHEBYSHEV_DISTANCE_2_COORDS
    } else {
        &[]
    };

    for &d in [Coords::new(0, 0)]
        .iter()
        .chain(region1.iter())
        .chain(region2.iter())
    {
        let Some(p) = sim.convert_p_cyclic(p + d) else {
            continue;
        };
        let Some(Structure::Settlement(target_settlement)) = planet.map[p].structure else {
            continue;
        };
        if target_settlement.id == settlement.id {
            let str = base_settlement_strength(&target_settlement, p, sim);
            planet.map[p].tile_events.insert(TileEvent::War {
                i,
                defence_str: str,
                offence_str: str * params.event.civil_war_offence_factor,
                offence: settlement.id,
            });
        }
    }
}

fn base_settlement_strength(settlement: &Settlement, p: Coords, sim: &Sim) -> f32 {
    settlement.pop * sim.energy_eff[p] * 0.01
}

/// Execution combat and returns damage and finished or not
pub fn exec_combat(defence_str: &mut f32, offence_str: &mut f32, params: &Params) -> (f32, bool) {
    let d_defence = *offence_str * params.event.base_combat_speed;
    let d_offence = *defence_str * params.event.base_combat_speed;
    *defence_str -= d_defence;
    *offence_str -= d_offence;
    (d_defence, d_defence <= 0.0 || d_offence <= 0.0)
}

fn empty_war_id(planet: &Planet) -> u32 {
    'i_loop: for a in 0.. {
        for e in planet.events.in_progress_iter() {
            if let PlanetEvent::War(WarEvent { i, .. }) = &e.event {
                if *i == a {
                    continue 'i_loop;
                }
            }
        }
        return a;
    }
    unreachable!()
}

fn civ_combinations(map: &Civs) -> Vec<(AnimalId, AnimalId)> {
    if map.len() < 2 {
        return Vec::new();
    }
    let keys: Vec<_> = map.keys().copied().collect();
    let n = keys.len();
    let ref_keys = &keys;
    (0..n)
        .flat_map(|i| ((i + 1)..n).map(move |j| (ref_keys[i], ref_keys[j])))
        .collect()
}
