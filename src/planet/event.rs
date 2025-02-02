use civ::civilize_animal;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Events {
    in_progress: Vec<EventInProgress>,
}

impl Events {
    pub fn in_progress(&self, event: &PlanetEvent) -> bool {
        self.in_progress.iter().any(|e| e.event == *event)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventInProgress {
    event: PlanetEvent,
    progress: u64,
    duration: Option<u64>,
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut completed_events = Vec::new();

    let events = planet.events.in_progress.clone();
    for event in events {
        match event.event {
            PlanetEvent::Plague(plague_event) => {
                super::plague::sim_plague(planet, sim, params, plague_event);
            }
            PlanetEvent::War(_) => todo!(),
            _ => (),
        }
    }

    for ein in &mut planet.events.in_progress {
        ein.progress += 1;
        // Event complete
        if ein.progress >= params.sim.event_duration[&ein.event.kind()] {
            completed_events.push(ein.event.clone());
        }
    }

    planet.events.in_progress.retain(|ein| {
        if let Some(duration) = ein.duration {
            ein.progress < duration
        } else {
            true
        }
    });

    for event in completed_events {
        #[allow(clippy::single_match)]
        match event {
            PlanetEvent::Civilize { target } => {
                civilize_animal(planet, sim, params, target);
            }
            _ => (),
        }
    }
}

pub fn start_event(planet: &mut Planet, event: PlanetEvent, _sim: &mut Sim, params: &Params) {
    let duration = params.sim.event_duration.get(&event.kind()).copied();
    planet.events.in_progress.push(EventInProgress {
        event,
        duration,
        progress: 0,
    });
}
