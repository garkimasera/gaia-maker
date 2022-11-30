use fnv::FnvHashMap;
use geom::Coords;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum ResourceKind {
    Energy,
    Material,
    Nitrogen,
    Helium,
}

pub type ResourceMap = fnv::FnvHashMap<ResourceKind, f32>;

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum Biome {
    Rock,
    Ocean,
    Desert,
    Grassland,
}

impl Default for Biome {
    fn default() -> Self {
        Self::Rock
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeAttrs {
    pub z: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructureAttrs {
    #[serde(default)]
    pub size: StructureSize,
    pub width: u32,
    pub height: u32,
    pub columns: usize,
    pub rows: usize,
    pub building: BuildingAttrs,
}

impl AsRef<BuildingAttrs> for StructureAttrs {
    fn as_ref(&self) -> &BuildingAttrs {
        &self.building
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructureSize {
    Small,
    Middle,
}

impl StructureSize {
    /// Additional occupiied tiles by a structure
    pub fn occupied_tiles(&self) -> Vec<Coords> {
        match self {
            StructureSize::Small => vec![],
            StructureSize::Middle => vec![Coords(1, 0), Coords(1, 1), Coords(0, 1)],
        }
    }
}

impl Default for StructureSize {
    fn default() -> Self {
        Self::Small
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(StructureKind))]
#[strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    EnumIter,
    AsRefStr,
    Display
))]
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum Structure {
    None,
    Occupied { by: Coords },
    OxygenGenerator,
    FertilizationPlant,
}

impl Structure {
    pub fn kind(&self) -> StructureKind {
        self.into()
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum GasKind {
    Oxygen,
    Nitrogen,
    CarbonDioxide,
    Helium,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingAttrs {
    #[serde(default)]
    pub cost: ResourceMap,
    #[serde(default)]
    pub upkeep: ResourceMap,
    #[serde(default)]
    pub produces: ResourceMap,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum OrbitalBuildingKind {
    FusionReactor,
    NitrogenSprayer,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum StarSystemBuildingKind {
    AsteroidMiningStation,
    DysonSwarmUnit,
    HeliumCollector,
    AmmoniaExtractor,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub start: StartParams,
    #[serde(skip)]
    pub biomes: FnvHashMap<Biome, BiomeAttrs>,
    #[serde(skip)]
    pub structures: FnvHashMap<StructureKind, StructureAttrs>,
    pub orbital_buildings: FnvHashMap<OrbitalBuildingKind, BuildingAttrs>,
    pub star_system_buildings: FnvHashMap<StarSystemBuildingKind, BuildingAttrs>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartParams {
    pub default_size: (u32, u32),
    pub resources: ResourceMap,
    pub atmo_mass: FnvHashMap<GasKind, f32>,
    pub orbital_buildings: FnvHashMap<OrbitalBuildingKind, u32>,
    pub star_system_buildings: FnvHashMap<StarSystemBuildingKind, u32>,
}
