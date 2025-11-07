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
    GiantMirror,
    GreenPlanet,
    MeltedIce = 101,
    DesertGreening,
    ArtificialBlueSky,
    OceanParadise,
    IndustrialRevolution = 201,
    InterSpeciesWar,
    Pandemic,
    StepTowardEcumenopolis,
    MagicalEnergy,
    Exodus,
    AbundantPower = 301,
    LowCarbonDioxide,
    DestroyPlanet,
    DarkClouds,
    PlanetSculpting,
}

pub static ACHIVEMENTS: std::sync::LazyLock<Vec<Achivement>> =
    std::sync::LazyLock::new(|| Achivement::iter().collect());

impl Achivement {
    pub fn upper_snake_case(&self) -> String {
        let s: &str = self.as_ref();
        s.replace('-', "_").to_uppercase()
    }
}

pub fn check_achivements(
    planet: &Planet,
    unlocked_achivements: &FnvHashSet<Achivement>,
    new_achivements: &mut FnvHashSet<Achivement>,
    params: &Params,
) {
    for achivement in Achivement::iter() {
        if !unlocked_achivements.contains(&achivement) && achivement.check(planet, params) {
            new_achivements.insert(achivement);
        }
    }
}

impl Achivement {
    fn check(self, planet: &Planet, params: &Params) -> bool {
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
            Achivement::ArtificialBlueSky => {
                planet.basics.origin == "barren" && planet.atmo.atm() >= 1.0
            }
            Achivement::OceanParadise => {
                planet.basics.origin == "archipelago"
                    && planet
                        .map
                        .iter()
                        .filter(|tile| {
                            if tile.biome.is_land() {
                                return false;
                            }
                            for animal in tile.animal {
                                let Some(animal) = animal else {
                                    continue;
                                };
                                let Some(animal_attr) = params.animals.get(&animal.id) else {
                                    continue;
                                };
                                if animal_attr.habitat.match_biome(Biome::Ocean) {
                                    return true;
                                }
                            }
                            false
                        })
                        .count()
                        > (planet.map.size().0 * planet.map.size().1 / 2) as usize
            }
            Achivement::IndustrialRevolution => planet.map.iter().any(|tile| {
                if let Some(Structure::Settlement(settlement)) = &tile.structure {
                    settlement.age >= CivilizationAge::Industrial
                } else {
                    false
                }
            }),
            Achivement::Pandemic => {
                planet
                    .map
                    .iter()
                    .filter(|tile| tile.tile_events.get(TileEventKind::Plague).is_some())
                    .count()
                    > 1000
            }
            Achivement::StepTowardEcumenopolis => {
                planet
                    .map
                    .iter()
                    .filter(|tile| matches!(tile.structure, Some(Structure::Settlement(_))))
                    .count()
                    > (planet.map.size().0 * planet.map.size().1 / 2) as usize
                    && planet.civs.iter().map(|civ| civ.1.total_pop).sum::<f32>() > 7500000.0
            }
            Achivement::MagicalEnergy => planet.civs.iter().any(|(_, civ)| {
                civ.current_age() == CivilizationAge::Iron
                    && civ.total_energy_consumption[EnergySource::Gift as usize] > 1.0
            }),
            Achivement::AbundantPower => planet.res.power >= 30000.0,
            Achivement::GiantMirror => Requirement::SpaceBuildingBuilt {
                kind: SpaceBuildingKind::OrbitalMirror,
                n: 1,
            }
            .check(planet),
            Achivement::LowCarbonDioxide => {
                planet.atmo.partial_pressure(GasKind::CarbonDioxide) < 30.0 * 1.0e-6
            }
            Achivement::DestroyPlanet => {
                if planet.stat.sum_biomass < 1.0 && planet.civs.is_empty() {
                    if let Some(record) = planet.stat.record(3000, params) {
                        record.biomass > 500_000.0 && record.pop.values().sum::<f32>() > 1000.0
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Achivement::DarkClouds => planet.cloud_albedo(params) >= 0.79,
            _ => false,
        }
    }
}
