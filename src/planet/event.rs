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
    duration: u64,
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut completed_events = Vec::new();

    for ein in &mut planet.events.in_progress {
        ein.progress += 1;
        // Event complete
        if ein.progress >= params.sim.event_duration[&ein.event.kind()] {
            completed_events.push(ein.event.clone());
        }
    }

    planet
        .events
        .in_progress
        .retain(|ein| ein.progress < ein.duration);

    for event in completed_events {
        match event {
            PlanetEvent::Civilize { target } => {
                civilize_animal(planet, sim, params, target);
            }
        }
    }
}

pub fn start_event(planet: &mut Planet, event: PlanetEvent, _sim: &mut Sim, params: &Params) {
    if let Some(duration) = params.sim.event_duration.get(&event.kind()).copied() {
        planet.events.in_progress.push(EventInProgress {
            event,
            duration,
            progress: 0,
        });
    }
}
