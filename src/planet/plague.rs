use geom::{Array2d, Coords};

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
    if let Some(Structure::Settlement(settlement)) = planet.map[p].structure {
        plague_event.map[p] = PlagueStatus::Infected {
            start_pop: settlement.pop,
        };
    }
}

/// Simutate plague, return true if the processing plague is completed
pub fn sim_plague(_planet: &mut Planet, _sim: &mut Sim, _params: &Params) -> bool {
    false
}
