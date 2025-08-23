use std::sync::atomic::AtomicBool;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants};

use crate::{manage_planet::SaveState, planet::*};

pub const TUTORIAL_PLANET: &str = "tutorial";

pub static CIVILIZEABLE: AtomicBool = AtomicBool::new(true);

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, Resource)]
pub struct TutorialState {
    current: TutorialStep,
    checklist: Vec<(ChecklistItem, bool)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, EnumDiscriminants)]
pub enum TutorialStep {
    Start(usize),
    Power(usize),
    Fertilize(usize),
    BuildOxygen(usize),
    WaitOxygen(usize),
    Carbon(usize),
    Animal(usize),
    Civilize(usize),
    ControlCiv(usize),
    OrbitalMirror(usize),
    Complete(bool),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, AsRefStr, Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ChecklistItem {
    PowerChecklist1,
    PowerChecklist2,
    FertilizeChecklist1,
    FertilizeChecklist2,
    BuildOxygenChecklist1,
    BuildOxygenChecklist2,
    WaitOxygenChecklist1,
    WaitOxygenChecklist2,
    CarbonChecklist1,
    AnimalChecklist1,
    CivilizeChecklist1,
    ControlCivChecklist1,
    OrbitalMirrorChecklist1,
    OrbitalMirrorChecklist2,
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
    CIVILIZEABLE.store(true, std::sync::atomic::Ordering::Relaxed);
    let Some(tutorial_state) = &mut save_state.save_file_metadata.tutorial_state else {
        return;
    };

    if tutorial_state.current == TutorialStep::Complete(true) {
        save_state.save_file_metadata.tutorial_state = None;
        return;
    }

    if matches!(
        tutorial_state.current,
        TutorialStep::Start(_)
            | TutorialStep::Power(_)
            | TutorialStep::Fertilize(_)
            | TutorialStep::BuildOxygen(_)
            | TutorialStep::WaitOxygen(_)
            | TutorialStep::Carbon(_)
            | TutorialStep::Animal(_)
    ) {
        CIVILIZEABLE.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    for (item, checked) in &mut tutorial_state.checklist {
        *checked = check(&planet, *item);
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

    pub fn complete(&mut self) {
        if let TutorialStep::Complete(complete) = &mut self.current {
            *complete = true;
        }
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
        Self::Fertilize(0),
        Self::Fertilize(1),
        Self::BuildOxygen(0),
        Self::BuildOxygen(1),
        Self::WaitOxygen(0),
        Self::Carbon(0),
        Self::Animal(0),
        Self::Animal(1),
        Self::Civilize(0),
        Self::ControlCiv(0),
        Self::OrbitalMirror(0),
        Self::Complete(false),
        Self::Complete(true),
    ];

    fn next(&self) -> Option<Self> {
        if let Some((i, _)) = Self::ORDER.iter().enumerate().find(|(_, value)| self == *value) {
            Self::ORDER.get(i + 1).copied()
        } else {
            None
        }
    }

    fn back(&self) -> Option<Self> {
        if let Some((i, _)) = Self::ORDER.iter().enumerate().find(|(_, value)| self == *value) {
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

    pub fn can_complete(&self) -> bool {
        matches!(self, TutorialStep::Complete(_))
    }
}

fn check(planet: &Planet, item: ChecklistItem) -> bool {
    let requirement = match item {
        ChecklistItem::PowerChecklist1 => Requirement::SpaceBuildingBuilt {
            kind: SpaceBuildingKind::FusionReactor,
            n: 5,
        },
        ChecklistItem::PowerChecklist2 => Requirement::SpaceBuildingBuilt {
            kind: SpaceBuildingKind::AsteroidMiningStation,
            n: 5,
        },
        ChecklistItem::FertilizeChecklist1 => Requirement::StructureBuilt {
            kind: StructureKind::FertilizationPlant,
            n: 3,
        },
        ChecklistItem::FertilizeChecklist2 => Requirement::BiomeTiles {
            biomes: vec![Biome::Grassland],
            n: 10,
        },
        ChecklistItem::BuildOxygenChecklist1 => Requirement::SpaceBuildingBuilt {
            kind: SpaceBuildingKind::DysonSwarmUnit,
            n: 5,
        },
        ChecklistItem::BuildOxygenChecklist2 => Requirement::StructureBuilt {
            kind: StructureKind::OxygenGenerator,
            n: 8,
        },
        ChecklistItem::WaitOxygenChecklist1 => Requirement::PartialPressureHigherThan {
            kind: GasKind::Oxygen,
            value: 0.12,
        },
        ChecklistItem::WaitOxygenChecklist2 => Requirement::BiomeTiles {
            biomes: vec![
                Biome::BorealForest,
                Biome::TemperateForest,
                Biome::TropicalRainforest,
            ],
            n: 50,
        },
        ChecklistItem::CarbonChecklist1 => Requirement::StructureBuilt {
            kind: StructureKind::CarbonCapturer,
            n: 2,
        },
        ChecklistItem::AnimalChecklist1 => Requirement::AnimalTiles { n: 30 },
        ChecklistItem::CivilizeChecklist1 => Requirement::Settlements {
            n: 1,
            animal_id: None,
        },
        ChecklistItem::OrbitalMirrorChecklist1 => Requirement::SpaceBuildingBuilt {
            kind: SpaceBuildingKind::OrbitalMirror,
            n: 1,
        },
        ChecklistItem::ControlCivChecklist1 => Requirement::CivPopGrowthAdjust { range: 150..=200 },
        ChecklistItem::OrbitalMirrorChecklist2 => {
            Requirement::OrbitalMirrorAdjust { range: -10..=-5 }
        }
    };
    requirement.check(planet)
}

fn checklist(d: TutorialStepDiscriminants) -> &'static [ChecklistItem] {
    match d {
        TutorialStepDiscriminants::Power => &[
            ChecklistItem::PowerChecklist1,
            ChecklistItem::PowerChecklist2,
        ],
        TutorialStepDiscriminants::Fertilize => &[
            ChecklistItem::FertilizeChecklist1,
            ChecklistItem::FertilizeChecklist2,
        ],
        TutorialStepDiscriminants::BuildOxygen => &[
            ChecklistItem::BuildOxygenChecklist1,
            ChecklistItem::BuildOxygenChecklist2,
        ],
        TutorialStepDiscriminants::WaitOxygen => &[
            ChecklistItem::WaitOxygenChecklist1,
            ChecklistItem::WaitOxygenChecklist2,
        ],
        TutorialStepDiscriminants::Carbon => &[ChecklistItem::CarbonChecklist1],
        TutorialStepDiscriminants::Animal => &[ChecklistItem::AnimalChecklist1],
        TutorialStepDiscriminants::Civilize => &[ChecklistItem::CivilizeChecklist1],
        TutorialStepDiscriminants::ControlCiv => &[ChecklistItem::ControlCivChecklist1],
        TutorialStepDiscriminants::OrbitalMirror => &[
            ChecklistItem::OrbitalMirrorChecklist1,
            ChecklistItem::OrbitalMirrorChecklist2,
        ],
        _ => &[],
    }
}
