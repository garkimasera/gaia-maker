use super::*;

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "req")]
pub enum Requirement {
    StructureBuilt {
        kind: StructureKind,
        n: u32,
    },
    SpaceBuildingBuilt {
        kind: SpaceBuildingKind,
        n: u32,
    },
    BiomeTiles {
        biomes: Vec<Biome>,
        n: u32,
    },
    PartialPressureHigherThan {
        kind: GasKind,
        value: f32,
    },
    AnimalTiles {
        id: AnimalId,
        size: AnimalSize,
        n: u32,
    },
    Settlements {
        n: u32,
        animal_id: Option<AnimalId>,
    },
    CivPopGrowthAdjust {
        range: std::ops::RangeInclusive<i16>,
    },
    OrbitalMirrorAdjust {
        range: std::ops::RangeInclusive<i32>,
    },
}

impl Requirement {
    pub fn check(&self, planet: &Planet) -> bool {
        match self {
            Self::StructureBuilt { kind, n } => {
                planet
                    .map
                    .iter()
                    .filter(|tile| {
                        tile.structure.as_ref().map(|structure| structure.kind()) == Some(*kind)
                    })
                    .count()
                    >= *n as usize
            }
            Self::SpaceBuildingBuilt { kind, n } => planet.space_building(*kind).n >= *n,
            Self::BiomeTiles { biomes, n } => {
                planet
                    .map
                    .iter()
                    .filter(|tile| biomes.contains(&tile.biome))
                    .count()
                    >= *n as usize
            }
            Self::PartialPressureHigherThan { kind, value } => {
                planet.atmo.partial_pressure(*kind) >= *value
            }
            Self::AnimalTiles { id, size, n } => {
                planet
                    .map
                    .iter()
                    .filter(|tile| {
                        if let Some(animal) = tile.animal[*size as usize] {
                            &animal.id == id
                        } else {
                            false
                        }
                    })
                    .count()
                    >= *n as usize
            }
            Self::Settlements { n, animal_id } => {
                planet
                    .map
                    .iter()
                    .filter(|tile| {
                        if let Some(Structure::Settlement(settlement)) = tile.structure {
                            if let Some(animal_id) = animal_id {
                                if animal_id != &settlement.id {
                                    return false;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .count()
                    >= *n as usize
            }
            Self::CivPopGrowthAdjust { range } => {
                for civ in planet.civs.values() {
                    if range.contains(&civ.civ_control.pop_growth) {
                        return true;
                    }
                }
                false
            }
            Self::OrbitalMirrorAdjust { range } => {
                if let BuildingControlValue::IncreaseRate(rate) =
                    &planet.space_building(SpaceBuildingKind::OrbitalMirror).control
                {
                    range.contains(rate)
                } else {
                    false
                }
            }
        }
    }
}
