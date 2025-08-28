use serde::{Deserialize, Serialize};

use super::*;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Events {
    in_progress: Vec<EventInProgress>,
}

impl Events {
    pub fn start_event(&mut self, event: PlanetEvent, duration: impl Into<Option<u64>>) {
        self.in_progress.push(EventInProgress {
            event,
            duration: duration.into(),
            progress: 0,
        });
    }

    pub fn in_progress_iter(&self) -> impl Iterator<Item = &EventInProgress> {
        self.in_progress.iter()
    }

    pub fn in_progress_iter_mut(&mut self) -> impl Iterator<Item = &mut EventInProgress> {
        self.in_progress.iter_mut()
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

    pub fn in_war(&self, a: AnimalId, b: AnimalId) -> Option<u32> {
        for e in self.in_progress_iter() {
            if let PlanetEvent::War(war_event) = &e.event
                && !war_event.ceased
                && let WarKind::InterSpecies(id0, id1) = &war_event.kind
                && ((*id0 == a && *id1 == b) || (*id0 == b && *id1 == a))
            {
                return Some(war_event.i);
            }
        }
        None
    }

    pub fn in_exodus_civ(&self) -> Option<AnimalId> {
        self.in_progress_iter().find_map(|e| {
            if let EventInProgress {
                event: PlanetEvent::Exodus(ExodusEvent { id }),
                ..
            } = e
            {
                Some(*id)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventInProgress {
    pub event: PlanetEvent,
    pub progress: u64,
    pub duration: Option<u64>,
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut completed_events = Vec::new();
    let mut plague_ended = false;
    let mut exodus_ended = false;

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
            PlanetEventKind::Decadence => {
                super::decadence::sim_decadence(planet, sim, params);
            }
            PlanetEventKind::War => {
                super::war::sim_war(planet, sim, params);
            }
            PlanetEventKind::Exodus => {
                exodus_ended = super::exodus::sim_exodus(planet, sim, params);
            }
        }
    }

    for ein in &mut planet.events.in_progress {
        ein.progress += 1;
    }

    if exodus_ended && let Some(id) = planet.events.in_exodus_civ() {
        let name = planet.civ_name(id);
        planet.reports.append(
            planet.cycles,
            ReportContent::EventExodusCompleted { id, name },
        );
        planet.civs.remove(&id);
    }

    planet.events.in_progress.retain(|ein| {
        // Check the event is completed by the duration
        if let Some(duration) = ein.duration
            && ein.progress >= duration
        {
            completed_events.push(ein.event.clone());
            return false;
        }
        // Check plague event is ended
        if plague_ended && ein.event.kind() == PlanetEventKind::Plague {
            return false;
        }

        // Check civil war is ended
        if let PlanetEvent::War(WarEvent { i, kind, .. }) = &ein.event
            && *kind == WarKind::CivilWar
            && matches!(sim.war_counter.get(i), Some(0) | None)
        {
            return false;
        }

        // Check exodus event is ended
        if exodus_ended && ein.event.kind() == PlanetEventKind::Exodus {
            return false;
        }

        true
    });

    super::exodus::cause_exodus(planet, sim, params);
}
