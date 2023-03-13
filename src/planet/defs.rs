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
    Default,
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
    #[default]
    Rock,
    Ocean,
    SeaIce,
    Desert,
    IceField,
    Tundra,
    Grassland,
    BorealForest,
    TemperateForest,
    TropicalRainforest,
}

impl Biome {
    pub fn is_land(&self) -> bool {
        !self.is_sea()
    }

    pub fn is_sea(&self) -> bool {
        matches!(*self, Biome::Ocean | Biome::SeaIce)
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
    /// Required temprature [°C]
    pub temprature: (f32, f32),
    /// Required rainfall [mm/year]
    pub rainfall: (f32, f32),
    /// Required fertility [%]
    pub fertility: f32,
    /// Required carbon biomass [kg/m2]
    pub biomass: f32,
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
    OxygenGenerator { state: StructureState },
    Rainmaker { state: StructureState },
    FertilizationPlant { state: StructureState },
    Heater { state: StructureState },
}

impl Structure {
    pub fn kind(&self) -> StructureKind {
        self.into()
    }

    pub fn _state(&mut self) -> Option<StructureState> {
        match self {
            Self::OxygenGenerator { state } => Some(*state),
            Self::Rainmaker { state } => Some(*state),
            Self::FertilizationPlant { state } => Some(*state),
            Self::Heater { state } => Some(*state),
            _ => None,
        }
    }

    pub fn _state_mut(&mut self) -> Option<&mut StructureState> {
        match self {
            Self::OxygenGenerator { state } => Some(state),
            Self::Rainmaker { state } => Some(state),
            Self::FertilizationPlant { state } => Some(state),
            Self::Heater { state } => Some(state),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StructureState {
    Working,
    Stopped,
    Disabled,
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
    pub sim: SimParams,
    pub new_planet: NewPlanetParams,
    pub default_start_params: StartParams,
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
    pub size: (u32, u32),
    pub difference_in_elevation: f32,
    pub resources: ResourceMap,
    pub atmo_mass: FnvHashMap<GasKind, f32>,
    pub water_volume: f32,
    pub orbital_buildings: FnvHashMap<OrbitalBuildingKind, u32>,
    pub star_system_buildings: FnvHashMap<StarSystemBuildingKind, u32>,
    pub cycles_before_start: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimParams {
    pub sim_normal_loop_duration_ms: u64,
    pub sim_fast_loop_duration_ms: u64,
    pub total_mass_per_atm: f32,
    pub secs_per_cycle: f32,
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
    /// Greeh house effect decrease by height at 1atm
    pub green_house_effect_height_decrease: f32,
    /// The number of loop of vapor transfer calculation
    pub n_loop_vapor_calc: usize,
    /// The ratio of tile vapor diffusion
    pub vapor_diffusion_factor: f32,
    /// The ratio of vapor loss
    pub vapor_loss_ratio: f32,
    /// Vaporizaion from ocean tile - °C table
    pub ocean_vaporization_table: Vec<(f32, f32)>,
    /// Max fertility table by temprature
    pub temprature_fertility_table: Vec<(f32, f32)>,
    /// Max fertility table by rainfall
    pub rainfall_fertility_table: Vec<(f32, f32)>,
    /// Fertility growth from biomass
    pub fertility_growth_from_biomass_table: Vec<(f32, f32)>,
    /// Base decrement value of fertility
    pub fertility_base_decrement: f32,
    /// Factor of fertility from adjacent tiles
    pub fertility_adjacent_factor: f32,
    /// Fertility attenuation factor in sea
    pub sea_fertility_attenuation_factor: f32,
    /// Fertility factor when changed from ocean
    pub change_from_ocean_fertility_factor: f32,
    /// Max biomass by fertility
    pub max_biomass_fertility_table: Vec<(f32, f32)>,
    /// Base biomass increase speed
    pub base_biomass_increase_speed: f32,
    /// Base biomass decrease speed
    pub base_biomass_decrease_speed: f32,
    /// Biomass growth speed factor by atm
    pub biomass_growth_speed_atm_table: Vec<(f32, f32)>,
    /// Biomass growth speed factor by CO2
    pub biomass_growth_speed_co2_table: Vec<(f32, f32)>,
    /// Ratio of biomass to buried carbon on decreasing
    pub decreased_biomass_to_buried_carbon_ratio: f32,
    /// Sea biomass factor compared to land
    pub sea_biomass_factor: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPlanetRangedParam {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPlanetParams {
    pub solar_constant: NewPlanetRangedParam,
    pub difference_in_elevation: NewPlanetRangedParam,
    pub water_volume_max: f32,
    pub nitrogen_max: f32,
    pub carbon_dioxide_max: f32,
}
