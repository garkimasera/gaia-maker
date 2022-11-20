use fnv::FnvHashMap;
use geom::Coords;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

/// Speed of simulation
pub const SPEED: f32 = 1.0 / 10.0;

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
    AsRefStr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ResourceKind {
    Energy,
    Material,
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
#[serde(rename_all = "kebab-case")]
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
    #[serde(default)]
    pub cost: FnvHashMap<ResourceKind, f32>,
    #[serde(default)]
    pub produces: FnvHashMap<ResourceKind, f32>,
    #[serde(default)]
    pub upkeeps: FnvHashMap<ResourceKind, f32>,
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
#[strum_discriminants(derive(Hash, Serialize, Deserialize, EnumIter, AsRefStr, Display))]
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

#[derive(Clone, Debug)]
pub struct Params {
    pub biomes: FnvHashMap<Biome, BiomeAttrs>,
    pub structures: FnvHashMap<StructureKind, StructureAttrs>,
}
