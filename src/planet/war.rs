use rand::seq::IndexedRandom;

use super::*;

const SETTLEMENT_STR_SUPPLY_INTERVAL_CYCLES: u64 = 3;
const TROOP_STR_THRESHOLD: f32 = 0.01;

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
            params.event.inter_species_war_interval_cycles.0
                ..params.event.inter_species_war_interval_cycles.1,
        ) + params.event.inter_species_war_duration_cycles.1;
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
            ReportContent::EventInterSpeciesWar {
                id_a,
                id_b,
                name_a: planet.civ_name(id_a),
                name_b: planet.civ_name(id_b),
            },
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

    for (e, progress, id_a, id_b) in planet.events.in_progress_iter_mut().filter_map(|e| {
        if let PlanetEvent::War(event) = &mut e.event {
            if let WarKind::InterSpecies(id_a, id_b) = event.kind {
                if !event.ceased {
                    Some((event, e.progress, id_a, id_b))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }) {
        let extinct = !planet.civs.contains_key(&id_a) || !planet.civs.contains_key(&id_b);
        if progress > params.event.inter_species_war_duration_cycles.0 || extinct {
            let prob = if progress > params.event.inter_species_war_duration_cycles.1 {
                1.0
            } else {
                1.0 / (params.event.inter_species_war_duration_cycles.1
                    - params.event.inter_species_war_duration_cycles.0) as f64
            };
            if sim.rng.random_bool(prob) {
                e.ceased = true;
                if !extinct {
                    planet.reports.append(
                        planet.cycles,
                        ReportContent::EventInterSpeciesWarCeased {
                            id_a,
                            id_b,
                            name_a: super::civ::civ_name(&planet.civs, id_a),
                            name_b: super::civ::civ_name(&planet.civs, id_b),
                        },
                    );
                }
            }
        }
    }
}

pub fn sim_settlement_str(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    if planet.cycles % SETTLEMENT_STR_SUPPLY_INTERVAL_CYCLES != 0 {
        return;
    }

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &mut planet.map[p].structure else {
            continue;
        };
        let max = base_settlement_strength(settlement, p, sim);
        if settlement.str < max {
            settlement.str += max * params.sim.settlement_str_supply_ratio;
        } else {
            settlement.str *= params.sim.garrison_troop_str_remaing_rate;
        }
    }
}

pub fn spawn_troops(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    if planet.cycles % SETTLEMENT_STR_SUPPLY_INTERVAL_CYCLES != 1 {
        return;
    }
    update_target_settlements(planet, sim, params);

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(mut settlement)) = planet.map[p].structure else {
            continue;
        };

        if !sim.rng.random_bool(params.event.spawn_troop_prob)
            || settlement.str < 0.5 * base_settlement_strength(&settlement, p, sim)
        {
            continue;
        }

        if let Some(dest) = choose_target(planet, sim, p, settlement.id) {
            let str = 0.5 * settlement.str;
            settlement.str = str;
            let tile = &mut planet.map[p];

            tile.tile_events.insert(TileEvent::Troop {
                id: settlement.id,
                age: settlement.age,
                dest,
                str,
            });
            tile.structure = Some(Structure::Settlement(settlement));
        }
    }
}

fn choose_target(planet: &Planet, sim: &mut Sim, p: Coords, src_id: AnimalId) -> Option<Coords> {
    let adj_iter = geom::CHEBYSHEV_DISTANCE_1_COORDS
        .iter()
        .chain(geom::CHEBYSHEV_DISTANCE_2_COORDS)
        .filter_map(|d| sim.convert_p_cyclic(p + *d));
    for p_adj in adj_iter {
        if let Some(Structure::Settlement(Settlement { id, .. })) = &planet.map[p_adj].structure {
            if src_id != *id && planet.events.in_war(src_id, *id).is_some() {
                return Some(p_adj);
            }
        }
    }

    if let Some(enemy_id) = get_enemies(&planet.events, src_id).choose(&mut sim.rng) {
        sim.war_target_settlements.get(enemy_id).map(|(_, p)| *p)
    } else {
        None
    }
}

fn update_target_settlements(planet: &Planet, sim: &mut Sim, params: &Params) {
    sim.war_target_settlements.clear();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(Settlement { id, pop, .. })) = &planet.map[p].structure
        else {
            continue;
        };

        let mut energy_score = 0.0;
        for &d in [Coords(0, 0)].iter().chain(geom::CHEBYSHEV_DISTANCE_1_COORDS) {
            if let Some(p_adj) = sim.convert_p_cyclic(p + d) {
                let tile = &planet.map[p_adj];
                if tile.buried_carbon > params.sim.buried_carbon_energy_threshold {
                    energy_score += (0.25 * tile.buried_carbon
                        / params.sim.buried_carbon_energy_threshold)
                        .min(2.0);
                }
                if matches!(tile.structure, Some(Structure::GiftTower)) {
                    energy_score += 1000.0;
                }
            }
        }
        let pop_score = pop / params.sim.settlement_max_pop[CivilizationAge::EarlySpace as usize];
        let score = energy_score + pop_score;

        sim.war_target_settlements
            .entry(*id)
            .and_modify(|s| {
                if score > s.0 {
                    s.0 = score;
                    s.1 = p;
                }
            })
            .or_insert((score, p));
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
            planet.map[p].tile_events.insert(TileEvent::War {
                i,
                offence_str: settlement.str * params.event.civil_war_offence_factor,
                offence: settlement.id,
            });
        }
    }
}

pub fn advance_troops(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut moved_troops = Vec::new();

    for p_prev in planet.map.iter_idx() {
        let tile = &mut planet.map[p_prev];
        let Some(TileEvent::Troop { id, str, dest, age }) =
            tile.tile_events.get(TileEventKind::Troop).copied()
        else {
            continue;
        };
        tile.tile_events.remove(TileEventKind::Troop);

        let str = str * params.sim.moved_troop_str_remaing_rate;

        let d = direction(p_prev, dest, sim.size.0);
        if d.0 == 0 && d.1 == 0 {
            // Choose new target
            if let Some(dest) = choose_target(planet, sim, p_prev, id) {
                moved_troops.push((p_prev, id, str, dest, age));
            }
        } else {
            let p = p_prev + d;
            moved_troops.push((p, id, str, dest, age));
        }
    }

    for (p, troop_id, troop_str, troop_dest, troop_age) in moved_troops {
        if troop_str < TROOP_STR_THRESHOLD {
            continue;
        }
        let p = sim.convert_p_cyclic(p).unwrap();
        let tile = &mut planet.map[p];
        tile.tile_events.insert(TileEvent::Troop {
            id: troop_id,
            age: troop_age,
            dest: troop_dest,
            str: troop_str,
        });

        if let Some(Structure::Settlement(target_settlement)) = tile.structure {
            let war_i = if troop_id != target_settlement.id {
                planet.events.in_war(troop_id, target_settlement.id)
            } else {
                None
            };
            if let Some(war_i) = war_i {
                if let Some(TileEvent::War {
                    i,
                    offence,
                    offence_str,
                }) = &tile.tile_events.get(TileEventKind::War)
                {
                    let (offence, offence_str) = if *offence == troop_id {
                        // Merge troop
                        (troop_id, offence_str + troop_str)
                    } else {
                        // Use larger strength
                        if troop_str >= *offence_str {
                            (troop_id, troop_str)
                        } else {
                            (*offence, *offence_str)
                        }
                    };
                    tile.tile_events.insert(TileEvent::War {
                        i: *i,
                        offence,
                        offence_str,
                    });
                } else {
                    tile.tile_events.insert(TileEvent::War {
                        i: war_i,
                        offence: troop_id,
                        offence_str: troop_str,
                    });
                }
            } else {
                let (id, age, dest, str) = if let Some(TileEvent::Troop { id, age, str, dest }) =
                    &tile.tile_events.get(TileEventKind::Troop)
                {
                    if *id == troop_id {
                        // Merge troop
                        if troop_str > *str {
                            (troop_id, troop_age, troop_dest, troop_str + str)
                        } else {
                            (troop_id, *age, *dest, troop_str + *str)
                        }
                    } else if planet.events.in_war(troop_id, *id).is_some() {
                        // Execute combat
                        let mut troop_str = troop_str;
                        let mut str = *str;
                        exec_combat_until_finish(&mut troop_str, &mut str);
                        if troop_str > TROOP_STR_THRESHOLD {
                            (troop_id, troop_age, troop_dest, troop_str)
                        } else if str > TROOP_STR_THRESHOLD {
                            (*id, *age, *dest, str)
                        } else {
                            continue;
                        }
                    } else {
                        // Use larger strength
                        if troop_str > *str {
                            (troop_id, troop_age, troop_dest, troop_str)
                        } else {
                            (*id, *age, *dest, *str)
                        }
                    }
                } else {
                    (troop_id, troop_age, troop_dest, troop_str)
                };
                tile.tile_events.insert(TileEvent::Troop { id, age, dest, str });
            }
        }
    }
}

fn direction(p: Coords, dest: Coords, w: u32) -> Coords {
    let w = w as i32;
    let dest = [dest, dest + Coords::new(w, 0), dest + Coords::new(-w, 0)]
        .into_iter()
        .min_by_key(|dest| dest.cdistance(p))
        .unwrap();
    let d = dest - p;
    Coords::new(
        if d.0 > 0 {
            1
        } else if d.0 < 0 {
            -1
        } else {
            0
        },
        if d.1 > 0 {
            1
        } else if d.1 < 0 {
            -1
        } else {
            0
        },
    )
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

/// Execution combat until finish
pub fn exec_combat_until_finish(defence_str: &mut f32, offence_str: &mut f32) {
    if *defence_str >= *offence_str {
        *defence_str = (defence_str.powi(2) - offence_str.powi(2)).sqrt();
        *offence_str = 0.0;
    } else {
        *defence_str = 0.0;
        *offence_str = (offence_str.powi(2) - defence_str.powi(2)).sqrt();
    }
}

fn get_enemies(events: &Events, id: AnimalId) -> smallvec::SmallVec<[AnimalId; 2]> {
    let mut enemies = smallvec::SmallVec::new();
    for e in events.in_progress_iter() {
        if let PlanetEvent::War(event) = &e.event {
            if !event.ceased {
                match event.kind {
                    WarKind::InterSpecies(aid, enemy_id) if aid == id => {
                        enemies.push(enemy_id);
                    }
                    WarKind::InterSpecies(enemy_id, aid) if aid == id => {
                        enemies.push(enemy_id);
                    }
                    _ => (),
                }
            }
        }
    }
    enemies
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
