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
        start_at: Some(p),
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
            let power = base_settlement_force_power(&target_settlement, p, sim);
            planet.map[p].tile_events.insert(TileEvent::War {
                i,
                defence_power: power,
                offence_power: power * params.event.civil_war_offence_factor,
                offence: settlement.id,
            });
        }
    }
}

fn base_settlement_force_power(settlement: &Settlement, p: Coords, sim: &Sim) -> f32 {
    settlement.pop * sim.energy_eff[p] * 0.01
}

/// Execution combat and returns damage and finished or not
pub fn exec_combat(
    defence_power: &mut f32,
    offence_power: &mut f32,
    params: &Params,
) -> (f32, bool) {
    let d_defence = *offence_power * params.event.base_combat_speed;
    let d_offence = *defence_power * params.event.base_combat_speed;
    *defence_power -= d_defence;
    *offence_power -= d_offence;
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
