use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{planet::Planet, sim::SaveState};

pub const TUTORIAL_PLANET: &str = "tutorial";

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, strum::EnumDiscriminants, Resource,
)]
pub enum TutorialState {
    Start(usize),
    TryingMapMove,
    DescFertile,
    WaitingGrasslandSpawn,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self::Start(0)
    }
}

pub fn update_tutorial(mut save_state: ResMut<SaveState>, planet: Res<Planet>) {
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };
    if tutorial_state.check_complete(&planet) {
        *tutorial_state = tutorial_state.next().unwrap();
    }
}

impl TutorialState {
    const ORDER: &[Self] = &[
        Self::Start(0),
        Self::Start(1),
        Self::TryingMapMove,
        Self::DescFertile,
        Self::WaitingGrasslandSpawn,
    ];

    pub fn next(&self) -> Option<Self> {
        if let Some((i, _)) = Self::ORDER
            .iter()
            .enumerate()
            .find(|(_, value)| self == *value)
        {
            Self::ORDER.get(i + 1).copied()
        } else {
            None
        }
    }

    pub fn next_by_manual(&self) -> bool {
        matches!(*self, Self::TryingMapMove)
    }

    pub fn has_next_popup_page(&self) -> bool {
        let d: TutorialStateDiscriminants = self.into();
        let next: TutorialStateDiscriminants = if let Some(next) = self.next() {
            next.into()
        } else {
            return false;
        };
        d == next
    }

    pub fn check_complete(&self, _planet: &Planet) -> bool {
        match *self {
            Self::WaitingGrasslandSpawn => false,
            _ => false,
        }
    }
}
