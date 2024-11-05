use std::collections::HashMap;

use fnv::FnvHashMap;
use geom::Coords;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

pub const KELVIN_CELSIUS: f32 = 273.15;
pub const RAINFALL_DURATION: f32 = 360.0;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Basics {
    /// Planet density [kg/m^3]
    pub density: f32,
    /// Planet radius [m]
    pub radius: f32,
    /// Solar constant at the planet [W/m^2]
    pub solar_constant: f32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct State {
    /// Multiplier for solar power
    pub solar_power_multiplier: f32,
    /// Solar power at the planet [W/m^2]
    pub solar_power: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            solar_power_multiplier: 1.0,
            solar_power: 0.0,
        }
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
    pub color: [u8; 3],
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
    OxygenGenerator { state: StructureBuildingState },
    Rainmaker { state: StructureBuildingState },
    FertilizationPlant { state: StructureBuildingState },
    Heater { state: StructureBuildingState },
    Settlement { settlement: Settlement },
}

impl Structure {
    pub fn kind(&self) -> StructureKind {
        self.into()
    }

    pub fn building_state(&self) -> Option<&StructureBuildingState> {
        match self {
            Self::OxygenGenerator { state } => Some(state),
            Self::Rainmaker { state } => Some(state),
            Self::FertilizationPlant { state } => Some(state),
            Self::Heater { state } => Some(state),
            _ => None,
        }
    }

    // pub fn building_state_mut(&mut self) -> Option<&mut StructureBuildingState> {
    //     match self {
    //         Self::OxygenGenerator { state } => Some(state),
    //         Self::Rainmaker { state } => Some(state),
    //         Self::FertilizationPlant { state } => Some(state),
    //         Self::Heater { state } => Some(state),
    //         _ => None,
    //     }
    // }
}

impl StructureKind {
    pub fn buildable_by_player(self) -> bool {
        matches!(
            self,
            Self::OxygenGenerator | Self::Rainmaker | Self::FertilizationPlant | Self::Heater
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StructureBuildingState {
    Working,
    Stopped,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Race {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    pub race: Race,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Settlement {
    pub age: CivilizationAge,
}

#[allow(clippy::enum_variant_names)]
#[derive(
    Clone, Copy, PartialEq, Eq, Default, Hash, Debug, Serialize, Deserialize, AsRefStr, EnumIter,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum CivilizationAge {
    #[default]
    StoneAge = 0,
    BronzeAge,
    IronAge,
    IndustrialAge,
    AtomicAge,
    EarlySpaceAge,
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
    Argon,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingAttrs {
    #[serde(default)]
    pub energy: f32,
    #[serde(default)]
    pub cost: f32,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub build_max: Option<u32>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub effect: Option<BuildingEffect>,
    #[serde(default)]
    pub control: BuildingControl,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize, AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum BuildingControl {
    #[default]
    AlwaysEnabled,
    EnabledNumber,
    IncreaseRate,
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
pub enum SpaceBuildingKind {
    FusionReactor,
    AsteroidMiningStation,
    DysonSwarmUnit,
    OrbitalMirror,
    NitrogenSprayer,
    CarbonDioxideSprayer,
    IonIrradiator,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum BuildingKind {
    Structure(StructureKind),
    Space(SpaceBuildingKind),
}

impl From<StructureKind> for BuildingKind {
    fn from(kind: StructureKind) -> Self {
        BuildingKind::Structure(kind)
    }
}

impl<T: Into<SpaceBuildingKind>> From<T> for BuildingKind {
    fn from(kind: T) -> Self {
        BuildingKind::Space(kind.into())
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum BuildingEffect {
    ProduceMaterial {
        mass: f32,
    },
    AdjustSolarPower,
    RemoveAtmo {
        mass: f32,
        efficiency_table: Vec<(f32, f32)>,
    },
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

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(PlanetEventKind))]
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
pub enum PlanetEvent {
    Civilize { target: u8 },
}

impl PlanetEvent {
    pub fn kind(&self) -> PlanetEventKind {
        self.into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub sim: SimParams,
    pub custom_planet: CustomPlanetParams,
    pub default_start_params: StartParams,
    pub history: HistoryParams,
    #[serde(skip)]
    pub biomes: FnvHashMap<Biome, BiomeAttrs>,
    #[serde(skip)]
    pub structures: FnvHashMap<StructureKind, StructureAttrs>,
    pub space_buildings: FnvHashMap<SpaceBuildingKind, BuildingAttrs>,
    #[serde(skip)]
    pub start_planets: Vec<StartPlanet>,
    pub monitoring: MonitoringParams,
}

impl Params {
    pub fn building_attrs<T: Into<BuildingKind>>(&self, kind: T) -> &BuildingAttrs {
        match kind.into() {
            BuildingKind::Structure(kind) => &self.structures[&kind].building,
            BuildingKind::Space(kind) => &self.space_buildings[&kind],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartParams {
    pub basics: Basics,
    pub size: (u32, u32),
    pub difference_in_elevation: f32,
    pub material: f32,
    pub atmo: FnvHashMap<GasKind, f64>,
    pub water_volume: f32,
    pub space_buildings: FnvHashMap<SpaceBuildingKind, u32>,
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
    /// Heat capacity of planet land surface [J/(K*m^2)]
    pub land_surface_heat_cap: f32,
    /// Heat capacity of planet sea [J/(K*m^3)]
    pub sea_heat_cap: f32,
    /// Depth of sea surface [m]
    pub sea_surface_depth: f32,
    /// Thickness of sea heat transfer layer [m]
    pub sea_heat_transfer_layer_thickness: f32,
    /// Max thickness of deep sea layer [m]
    pub max_deep_sea_layer_thickness: f32,
    /// Latitude and averaged insolation table
    pub latitude_insolation_table: Vec<(f32, f32)>,
    /// The ratio of tile air diffusion
    pub air_diffusion_factor: f32,
    /// The ratio of tile sea diffusion
    pub sea_diffusion_factor: f32,
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
    /// Humidity calculation factors (humidity = rainfall - factors.0 * (temprature + factors.1))
    pub humidity_factors: (f32, f32),
    /// Max fertility table by temprature
    pub temprature_fertility_table: Vec<(f32, f32)>,
    /// Max fertility table by humidity
    pub humidity_fertility_table: Vec<(f32, f32)>,
    /// Max fertility table by nitrogen atm
    pub nitrogen_fertility_table: Vec<(f32, f32)>,
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
    /// Nitrogen in soil per area [m2] each percent
    pub soil_nitrogen: f32,
    /// Max biomass by fertility
    pub max_biomass_fertility_table: Vec<(f32, f32)>,
    /// Max biomass by O2
    pub max_biomass_factor_o2_table: Vec<(f32, f32)>,
    /// Base biomass increase speed
    pub base_biomass_increase_speed: f32,
    /// Base biomass decrease speed
    pub base_biomass_decrease_speed: f32,
    /// Biomass growth speed factor by atm
    pub biomass_growth_speed_atm_table: Vec<(f32, f32)>,
    /// Biomass growth speed factor by CO2
    pub biomass_growth_speed_co2_table: Vec<(f32, f32)>,
    /// Table of decreased biomass to buried carbon ratio by oxygen atm
    pub biomass_to_buried_carbon_ratio_o2_table: Vec<(f32, f32)>,
    /// Table of decreased biomass to buried carbon ratio by carbon dioxide atm
    pub biomass_to_buried_carbon_ratio_co2_table: Vec<(f32, f32)>,
    /// Sea biomass factor compared to land
    pub sea_biomass_factor: f32,
    /// Required thickness of ice for ice field [m]
    pub ice_thickness_of_ice_field: f32,
    /// Ice melting temprature [K]
    pub ice_melting_temprature: f32,
    /// Ice melting speed [m/K]
    pub ice_melting_height_per_temp: f32,
    /// Factor for adding ice height from rainfall [m/(rainfall)mm]
    pub fallen_snow_factor: f32,
    /// Biome transition probability before start simulation
    pub before_start_biome_transition_probability: f32,
    /// Duration of events
    pub event_duration: HashMap<PlanetEventKind, u64>,
    /// The max number of civilizations
    pub max_civs: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPlanetRangedParam {
    pub min: f32,
    pub max: f32,
    pub default: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewPlanetPercentageParam {
    pub default_percentage: f32,
    pub max: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomPlanetParams {
    pub solar_constant: NewPlanetRangedParam,
    pub difference_in_elevation: NewPlanetRangedParam,
    pub water_volume: NewPlanetPercentageParam,
    pub nitrogen: NewPlanetPercentageParam,
    pub carbon_dioxide: NewPlanetPercentageParam,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryParams {
    pub max_record: usize,
    pub interval_cycles: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartPlanet {
    pub id: String,
    pub difficulty: PlanetDifficulty,
    pub solar_constant: (f32, f32),
    pub elevation: (f32, f32),
    pub water_volume: (f32, f32),
    pub nitrogen: (f32, f32),
    pub carbon_dioxide: (f32, f32),
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, EnumString, AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum PlanetDifficulty {
    VeryEasy,
    Easy,
    Normal,
    Hard,
    VeryHard,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringParams {
    pub interval_cycles: u64,
    pub warn_high_temp_threshold: f32,
    pub warn_low_temp_threshold: f32,
}
