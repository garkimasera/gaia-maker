use civ::civilize_animal;
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Events {
    in_progress: Vec<EventInProgress>,
}

impl Events {
    pub fn start_event(&mut self, event: PlanetEvent, params: &Params) {
        let duration = params.sim.event_duration.get(&event.kind()).copied();
        self.in_progress.push(EventInProgress {
            event,
            duration,
            progress: 0,
        });
    }

    pub fn in_progress_iter(&self, kind: PlanetEventKind) -> impl Iterator<Item = &PlanetEvent> {
        self.in_progress.iter().filter_map(move |e| {
            if e.event.kind() == kind {
                Some(&e.event)
            } else {
                None
            }
        })
    }

    pub fn in_progress_iter_mut(
        &mut self,
        kind: PlanetEventKind,
    ) -> impl Iterator<Item = &mut PlanetEvent> {
        self.in_progress.iter_mut().filter_map(move |e| {
            if e.event.kind() == kind {
                Some(&mut e.event)
            } else {
                None
            }
        })
    }

    pub fn in_progress_event_cycles(&mut self, kind: PlanetEventKind) -> impl Iterator<Item = u64> {
        self.in_progress.iter_mut().filter_map(move |e| {
            if e.event.kind() == kind {
                Some(e.progress)
            } else {
                None
            }
        })
    }

    pub fn in_progress_civilize_event(&self, animal_id: AnimalId) -> bool {
        self.in_progress_iter(PlanetEventKind::Civilize).any(|event| {
            if let PlanetEvent::Civilize { target } = event {
                *target == animal_id
            } else {
                false
            }
        })
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
    let mut plague_ended = false;

    let mut event_kind_list: Vec<_> = planet
        .events
        .in_progress
        .iter()
        .map(|e| e.event.kind())
        .collect();
    event_kind_list.sort();
    event_kind_list.dedup();
    for event_kind in event_kind_list {
        match event_kind {
            PlanetEventKind::Plague => {
                plague_ended = super::plague::sim_plague(planet, sim, params);
            }
            PlanetEventKind::War => (),
            _ => (),
        }
    }

    for ein in &mut planet.events.in_progress {
        ein.progress += 1;
        // Event complete
        if let Some(duration) = params.sim.event_duration.get(&ein.event.kind()) {
            if ein.progress >= *duration {
                completed_events.push(ein.event.clone());
            }
        }
    }

    planet.events.in_progress.retain(|ein| {
        // Check the event is completed by the duration
        if let Some(duration) = ein.duration {
            if ein.progress >= duration {
                return false;
            }
        }
        // Check plague event is ended
        if plague_ended && ein.event.kind() == PlanetEventKind::Plague {
            return false;
        }

        // Check civil war is ended
        if let PlanetEvent::War(WarEvent { i, kind, .. }) = &ein.event {
            if *kind == WarKind::CivilWar && *i == 0 {
                return false;
            }
        }

        true
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
