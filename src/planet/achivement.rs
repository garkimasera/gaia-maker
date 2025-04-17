use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumString};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(Serialize, Deserialize, AsRefStr, EnumString, EnumIter)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Achivement {
    Grasslands,
    Forests,
}

pub fn check_achivements(
    planet: &Planet,
    unlocked_achivements: &FnvHashSet<Achivement>,
    new_achivements: &mut FnvHashSet<Achivement>,
) {
    for achivement in Achivement::iter() {
        if !unlocked_achivements.contains(&achivement) && achivement.check(planet) {
            new_achivements.insert(achivement);
        }
    }
}

impl Achivement {
    fn check(self, planet: &Planet) -> bool {
        match self {
            Achivement::Grasslands => Requirement::BiomeTiles {
                biomes: vec![Biome::Grassland],
                n: 10,
            }
            .check(planet),
            Achivement::Forests => Requirement::BiomeTiles {
                biomes: vec![
                    Biome::BorealForest,
                    Biome::TemperateForest,
                    Biome::TropicalRainforest,
                ],
                n: 50,
            }
            .check(planet),
        }
    }
}
