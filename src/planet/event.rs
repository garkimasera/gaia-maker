use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Events {
    in_progress: Vec<EventInProgress>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventInProgress {
    event: PlanetEvent,
    progress: u64,
    duration: u64,
}

pub fn advance(planet: &mut Planet, _sim: &mut Sim, params: &Params) {
    for ein in &mut planet.events.in_progress {
        ein.progress += 1;
        // Event complete
        if ein.progress >= params.sim.event_duration[&ein.event.kind()] {}
    }

    planet
        .events
        .in_progress
        .retain(|ein| ein.progress < ein.duration);
}

pub fn start_event(planet: &mut Planet, event: PlanetEvent, _sim: &mut Sim, params: &Params) {
    // match &event {
    //     PlanetEvent::Civilize { target } => {

    //     }
    // }

    if let Some(duration) = params.sim.event_duration.get(&event.kind()).copied() {
        planet.events.in_progress.push(EventInProgress {
            event,
            duration,
            progress: 0,
        });
    }
}
