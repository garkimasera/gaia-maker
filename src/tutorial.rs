use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants};

use crate::{planet::Planet, sim::SaveState};

pub const TUTORIAL_PLANET: &str = "tutorial";

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Resource)]
pub struct TutorialState {
    current: TutorialStep,
    checklist: Vec<(ChecklistItem, bool)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, EnumDiscriminants)]
pub enum TutorialStep {
    Start(usize),
    Power(usize),
    Fertilize,
    GenOxygen(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ChecklistItem {
    PowerChecklist0,
    PowerChecklist1,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self {
            current: TutorialStep::Start(0),
            checklist: Vec::new(),
        }
    }
}

pub fn update_tutorial(mut save_state: ResMut<SaveState>, planet: Res<Planet>) {
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };

    for (item, checked) in &mut tutorial_state.checklist {
        if !*checked {
            *checked = check(&planet, *item);
        }
    }
}

impl TutorialState {
    pub fn move_next(&mut self) {
        self.move_to(self.current.next().unwrap());
    }

    pub fn move_back(&mut self) {
        self.move_to(self.current.back().unwrap());
    }

    pub fn current_step(&self) -> TutorialStep {
        self.current
    }

    pub fn checked(&self) -> bool {
        self.checklist.iter().all(|(_, checked)| *checked)
    }

    pub fn checklist(&self) -> &[(ChecklistItem, bool)] {
        &self.checklist
    }

    fn move_to(&mut self, step: TutorialStep) {
        let dstep = TutorialStepDiscriminants::from(step);
        if TutorialStepDiscriminants::from(self.current) != dstep {
            self.checklist = checklist(dstep).iter().map(|item| (*item, false)).collect();
        }
        log::info!("change tutorial step to {:?}", step);
        self.current = step;
    }
}

impl TutorialStep {
    const ORDER: &[Self] = &[
        Self::Start(0),
        Self::Start(1),
        Self::Power(0),
        Self::Power(1),
        Self::Fertilize,
    ];

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
}

fn check(_planet: &Planet, _item: ChecklistItem) -> bool {
    false
}

fn checklist(d: TutorialStepDiscriminants) -> Vec<ChecklistItem> {
    match d {
        TutorialStepDiscriminants::Power => {
            vec![
                ChecklistItem::PowerChecklist0,
                ChecklistItem::PowerChecklist1,
            ]
        }
        _ => Vec::new(),
    }
}
