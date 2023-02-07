use fnv::FnvHashMap;
use geom::Coords;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

pub const KELVIN_CELSIUS: f32 = 273.15;
pub const RAINFALL_DURATION: f32 = 360.0;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PlanetBasics {
    /// Planet density [kg/m^3]
    pub density: f32,
    /// Planet radius [m]
    pub radius: f32,
    /// Solar constant at the planet [W/m^2]
    pub solar_constant: f32,
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
pub enum ResourceKind {
    Energy,
    Material,
    Ice,
    Nitrogen,
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
    pub albedo: f32,
    pub revaporization_ratio: f32,
    pub priority: u32,
    pub mean_transition_time: f32,
    pub requirements: BiomeRequirements,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeRequirements {
    pub temprature: (f32, f32),
    pub rainfall: (f32, f32),
    pub fertility: (f32, f32),
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
    Rainmaker,
    FertilizationPlant,
    Heater,
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingAttrs {
    #[serde(default)]
    pub cost: ResourceMap,
    #[serde(default)]
    pub upkeep: ResourceMap,
    #[serde(default)]
    pub produces: ResourceMap,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub build_max: Option<u32>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub effect: Option<BuildingEffect>,
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
    IceMiningStation,
    DysonSwarmUnit,
    AmmoniaExtractor,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum BuildingEffect {
    SprayToAtmo {
        kind: GasKind,
        mass: f32,
    },
    Vapor {
        value: f32,
        additional_water: f32,
    },
    Heater {
        heat: f32,
    },
    Fertilize {
        increment: f32,
        max: f32,
        range: u32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub start: StartParams,
    pub sim: SimParams,
    #[serde(skip)]
    pub biomes: FnvHashMap<Biome, BiomeAttrs>,
    #[serde(skip)]
    pub structures: FnvHashMap<StructureKind, StructureAttrs>,
    pub orbital_buildings: FnvHashMap<OrbitalBuildingKind, BuildingAttrs>,
    pub star_system_buildings: FnvHashMap<StarSystemBuildingKind, BuildingAttrs>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartParams {
    pub basics: PlanetBasics,
    pub default_size: (u32, u32),
    pub max_height: f32,
    pub resources: ResourceMap,
    pub atmo_mass: FnvHashMap<GasKind, f32>,
    pub water_volume: f32,
    pub orbital_buildings: FnvHashMap<OrbitalBuildingKind, u32>,
    pub star_system_buildings: FnvHashMap<StarSystemBuildingKind, u32>,
    pub days_before_start: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimParams {
    pub sim_normal_loop_duration_ms: u64,
    pub sim_fast_loop_duration_ms: u64,
    pub total_mass_per_atm: f32,
    pub secs_per_day: f32,
    /// Heat capacity of air [J/(kg*K)]
    pub air_heat_cap: f32,
    /// Heat capacity of planet surface [J/(kg*m^3)]
    pub surface_heat_cap: f32,
    /// Factor to average sunlight power per day
    pub sunlight_day_averaging_factor: f32,
    /// The ratio of tile air diffusion
    pub air_diffusion_factor: f32,
    /// The number of loop of atmosphere heat transfer calculation
    pub n_loop_atmo_heat_calc: usize,
    /// Greeh house effect table of CO2
    pub co2_green_house_effect_table: Vec<(f32, f32)>,
    /// The number of loop of vapor transfer calculation
    pub n_loop_vapor_calc: usize,
    /// The ratio of tile vapor diffusion
    pub vapor_diffusion_factor: f32,
    /// The ratio of vapor loss
    pub vapor_loss_ratio: f32,
    /// Vaporizaion from ocean tile - °C table
    pub ocean_vaporization_table: Vec<(f32, f32)>,
    /// Fertility factor when changed from ocean
    pub change_from_ocean_fertility_factor: f32,
}
