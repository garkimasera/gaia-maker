use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{planet::Planet, sim::SaveState};

pub const TUTORIAL_PLANET: &str = "tutorial";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Resource)]
pub struct TutorialState {
    current: TutorialStep,
    checked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, strum::EnumDiscriminants)]
pub enum TutorialStep {
    Start(usize),
    Fertilize,
    GenOxygen(usize),
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            current: TutorialStep::Start(0),
            checked: false,
        }
    }
}

pub fn update_tutorial(mut save_state: ResMut<SaveState>, planet: Res<Planet>) {
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };
    if !tutorial_state.checked && tutorial_state.current.check_complete(&planet) {
        tutorial_state.checked = true;
    }
}

impl TutorialState {
    pub fn move_next(&mut self) {
        self.current = self.current.next().unwrap();
        log::info!("change tutorial step to {:?}", self.current);
    }

    pub fn move_back(&mut self) {
        self.current = self.current.back().unwrap();
        log::info!("change tutorial step to {:?}", self.current);
    }

    pub fn current_step(&self) -> TutorialStep {
        self.current
    }

    pub fn checked(&self) -> bool {
        self.checked
    }
}

impl TutorialStep {
    const ORDER: &[Self] = &[Self::Start(0), Self::Start(1), Self::Fertilize];

    fn next(&self) -> Option<Self> {
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

    fn back(&self) -> Option<Self> {
        if let Some((i, _)) = Self::ORDER
            .iter()
            .enumerate()
            .find(|(_, value)| self == *value)
        {
            if i > 0 {
                Self::ORDER.get(i - 1).copied()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn has_next_tutorial(&self) -> bool {
        let d: TutorialStepDiscriminants = self.into();
        let next: TutorialStepDiscriminants = if let Some(next) = self.next() {
            next.into()
        } else {
            return false;
        };
        d != next
    }

    pub fn can_back(&self) -> bool {
        let d: TutorialStepDiscriminants = self.into();
        let back: TutorialStepDiscriminants = if let Some(back) = self.back() {
            back.into()
        } else {
            return false;
        };
        d == back
    }

    fn check_complete(&self, planet: &Planet) -> bool {
        match *self {
            Self::Fertilize => check_fertilize(planet),
            Self::GenOxygen(1) => check_gen_oxygen(planet),
            _ => true,
        }
    }
}

fn check_fertilize(_planet: &Planet) -> bool {
    false
}

fn check_gen_oxygen(_planet: &Planet) -> bool {
    false
}
