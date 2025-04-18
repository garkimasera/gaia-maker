use fnv::FnvHashSet;
use strum::{AsRefStr, EnumIter, EnumString};

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(AsRefStr, EnumString, EnumIter, num_derive::FromPrimitive)]
#[strum(serialize_all = "kebab-case")]
#[repr(u16)]
pub enum Achivement {
    Grasslands = 1,
    Forests,
    Animals,
    Civilize,
    GreenPlanet,
    MeltedIce = 101,
    DesertGreening,
    IndustrialRevolution = 201,
    StepTowardEcumenopolis,
    AbundantPower = 301,
    GiantMirror,
}

pub static ACHIVEMENTS: std::sync::LazyLock<Vec<Achivement>> =
    std::sync::LazyLock::new(|| Achivement::iter().collect());

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
            Achivement::Animals => planet
                .map
                .iter()
                .flat_map(|tile| &tile.animal)
                .any(|animal| animal.is_some()),
            Achivement::Civilize => planet
                .map
                .iter()
                .any(|tile| matches!(tile.structure, Some(Structure::Settlement(_)))),
            Achivement::GreenPlanet => planet.stat.sum_biomass > 2_000_000.0,
            Achivement::MeltedIce => {
                planet.basics.origin == "ice"
                    && planet
                        .map
                        .iter()
                        .filter(|tile| matches!(tile.biome, Biome::IceSheet | Biome::SeaIce))
                        .count()
                        == 0
            }
            Achivement::DesertGreening => {
                planet.basics.origin == "desert" && planet.stat.sum_biomass > 600_000.0
            }
            Achivement::IndustrialRevolution => planet.map.iter().any(|tile| {
                if let Some(Structure::Settlement(settlement)) = &tile.structure {
                    settlement.age >= CivilizationAge::Industrial
                } else {
                    false
                }
            }),
            Achivement::StepTowardEcumenopolis => {
                planet
                    .map
                    .iter()
                    .filter(|tile| matches!(tile.structure, Some(Structure::Settlement(_))))
                    .count()
                    > (planet.map.size().0 * planet.map.size().1 / 2) as usize
                    && planet.civs.iter().map(|civ| civ.1.total_pop).sum::<f32>() > 10_000_000.0
            }
            Achivement::AbundantPower => planet.res.power >= 10000.0,
            Achivement::GiantMirror => {
                planet.space_building(SpaceBuildingKind::OrbitalMirror).n > 0
            }
        }
    }
}
