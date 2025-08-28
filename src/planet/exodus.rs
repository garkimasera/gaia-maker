use std::collections::HashMap;

use super::*;

const CAUSE_EXODUS_INTERVAL_CYCLES: u64 = 4;

pub fn sim_exodus(planet: &mut Planet, sim: &mut Sim, params: &Params) -> bool {
    let Some(id) = planet.events.in_exodus_civ() else {
        return true;
    };

    let mut remaining_settlements = 0;

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &planet.map[p].structure else {
            continue;
        };
        if settlement.id != id {
            continue;
        }

        remaining_settlements += 1;

        if !planet.map[p].tile_events.contains(TileEventKind::Exodus)
            && sim.rng.random_bool(params.event.settlement_exodus_prob.into())
        {
            planet.map[p].tile_events.insert(TileEvent::Exodus {
                remaining_cycles: params.event.settlement_exodus_cycles,
            });
        }

        if let Some(TileEvent::Exodus { remaining_cycles }) =
            planet.map[p].tile_events.get_mut(TileEventKind::Exodus)
        {
            *remaining_cycles -= 1;

            if *remaining_cycles == 0 {
                planet.map[p].structure = None;
                planet.map[p].tile_events.remove(TileEventKind::Exodus);
            }
        }
    }

    if remaining_settlements == 0 {
        delete_civ(planet, id);
        true
    } else {
        false
    }
}

pub fn cause_exodus(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    if planet.cycles % CAUSE_EXODUS_INTERVAL_CYCLES != 0 {
        return;
    }

    if planet
        .civs
        .values()
        .all(|civ| civ.most_advanced_age < CivilizationAge::EarlySpace)
    {
        return;
    }

    let mut tech_level_sum: HashMap<AnimalId, (u32, f32)> = HashMap::default();

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &planet.map[p].structure else {
            continue;
        };

        if settlement.age == CivilizationAge::EarlySpace {
            let e = tech_level_sum.entry(settlement.id).or_default();
            e.0 += 1;
            e.1 += settlement.tech_exp;
        }
    }

    for event in planet.events.in_progress_iter() {
        if matches!(
            event.event,
            PlanetEvent::Decadence(_)
                | PlanetEvent::Exodus(_)
                | PlanetEvent::War(_)
                | PlanetEvent::Plague(_)
        ) {
            return;
        }
    }

    for (&id, civ) in &planet.civs {
        let Some((n, tech_level_sum)) = tech_level_sum.remove(&id) else {
            continue;
        };
        let tech_level_average = tech_level_sum / n as f32;

        if tech_level_average < params.event.exodus_tech_level_threshold
            || civ.total_pop < params.event.exodus_pop_threshold
        {
            continue;
        }

        let tech_control_weight = (civ.civ_control.tech_development as f32 / 100.0).powi(2);
        let nuclear_control = civ.civ_control.energy_weight[&EnergySource::Nuclear] as f32 / 100.0;
        let atomic_weight = if nuclear_control < 0.5 {
            0.0
        } else {
            nuclear_control.powi(2)
        };
        let exodus_prob = params.event.base_exodus_prob
            * tech_control_weight
            * atomic_weight
            * (tech_level_average / params.event.exodus_tech_level_threshold)
            * (civ.total_pop / params.event.exodus_pop_threshold);

        if sim.rng.random_bool(exodus_prob.clamp(0.0, 1.0).into()) {
            planet
                .events
                .start_event(PlanetEvent::Exodus(ExodusEvent { id }), None);
            sim.new_achievements.insert(Achivement::Exodus);
            planet.reports.append(
                planet.cycles,
                ReportContent::EventExodus {
                    id,
                    name: planet.civ_name(id),
                },
            );
        }
    }
}

pub fn delete_civ(planet: &mut Planet, civ_id: AnimalId) {
    for p in planet.map.iter_idx() {
        if let Some(Structure::Settlement(settlement)) = planet.map[p].structure
            && settlement.id == civ_id
        {
            planet.map[p].structure = None;
        }

        planet.map[p].tile_events.retain(|tile_event| {
            if let TileEvent::Vehicle { id, .. } = tile_event
                && *id == civ_id
            {
                return false;
            }
            if let TileEvent::Troop { id, .. } = tile_event
                && *id == civ_id
            {
                return false;
            }
            true
        });
    }
}
