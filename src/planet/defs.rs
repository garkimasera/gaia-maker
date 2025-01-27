use std::collections::{BTreeMap, HashMap};

use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, Same};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumIter, EnumString};

use super::serde_with_types::*;

pub const TILE_SIZE: f32 = 48.0;
pub const PIECE_SIZE: f32 = TILE_SIZE / 2.0;

pub const KELVIN_CELSIUS: f32 = 273.15;
pub const RAINFALL_DURATION: f32 = 360.0;

pub type AnimalId = arrayvec::ArrayString<20>;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Basics {
    /// Planet name
    #[serde(default)]
    pub name: String,
    /// Planet origin id
    #[serde(default)]
    pub origin: String,
    /// Planet radius [m]
    pub radius: f32,
    /// Solar constant at the planet [W/m^2]
    pub solar_constant: f32,
    /// Geothermal power from the planet core [W]
    pub geothermal_power: f32,
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

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BiomeRequirements {
    /// Required temperature [°C]
    #[serde_as(as = "(Celsius, Celsius)")]
    pub temp: (f32, f32),
    /// Required rainfall [mm/year]
    pub rainfall: (f32, f32),
    /// Required fertility [%]
    pub fertility: f32,
    /// Required carbon biomass [kg/m2]
    pub biomass: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructureAttrs {
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
    OxygenGenerator,
    Rainmaker,
    FertilizationPlant,
    Heater,
    CarbonCapturer,
    Settlement(Settlement),
}

impl Structure {
    pub fn kind(&self) -> StructureKind {
        self.into()
    }
}

impl StructureKind {
    pub fn buildable_by_player(self) -> bool {
        !matches!(self, Self::Settlement)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StructureBuildingState {
    Working,
    Stopped,
    Disabled,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumDiscriminants)]
#[strum_discriminants(name(TileEventKind))]
#[strum_discriminants(derive(
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    AsRefStr,
    Display,
    EnumIter
))]
#[strum_discriminants(serde(rename_all = "snake_case"))]
#[strum_discriminants(strum(serialize_all = "kebab-case"))]
pub enum TileEvent {
    Fire,
    BlackDust { remaining_cycles: u32 },
    AerosolInjection { remaining_cycles: u32 },
    Plague,
}

impl TileEvent {
    pub fn kind(&self) -> TileEventKind {
        self.into()
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimalAttr {
    pub size: AnimalSize,
    pub cost: f32,
    pub habitat: AnimalHabitat,
    #[serde(default = "animal_ratio_attr_default")]
    #[serde_as(as = "Percent")]
    pub growth_speed: f32,
    /// Livable temperature range
    #[serde_as(as = "(Celsius, Celsius)")]
    pub temp: (f32, f32),
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub civ: Option<AnimalCivParams>,
}

fn animal_ratio_attr_default() -> f32 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimalCivParams {
    pub civilize_cost: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, AsRefStr)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum AnimalSize {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl AnimalSize {
    pub const LEN: usize = AnimalSize::Large as usize + 1;

    pub fn iter() -> [Self; Self::LEN] {
        [Self::Small, Self::Medium, Self::Large]
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnimalHabitat {
    Land,
    Sea,
    Biomes(Vec<Biome>),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Animal {
    pub id: AnimalId,
    pub n: f32,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Civilization {
    pub total_pop: f32,
    pub total_settlement: [u32; CivilizationAge::LEN],
    pub total_energy_consumption: [f32; EnergySource::LEN],
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Settlement {
    pub id: AnimalId,
    pub age: CivilizationAge,
    pub pop: f32,
    pub tech_exp: f32,
}

#[allow(clippy::enum_variant_names)]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumIter,
    num_derive::FromPrimitive,
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

impl CivilizationAge {
    pub const LEN: usize = Self::EarlySpaceAge as usize + 1;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize, AsRefStr, EnumIter)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum EnergySource {
    Biomass = 0,
    WindSolar,
    HydroGeothermal,
    FossilFuel,
    Nuclear,
    Gift,
}

impl EnergySource {
    pub const LEN: usize = Self::Gift as usize + 1;
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
    IceImporter,
    NitrogenImporter,
    CarbonImporter,
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
        limit_atm: f32,
    },
    AddWater {
        value: f32,
    },
    Vapor {
        value: f32,
    },
    Heater {
        heat: f32,
    },
    Fertilize {
        increment: f32,
        max: f32,
        range: u32,
    },
    CaptureCarbonDioxide {
        mass: f32,
        limit_atm: f32,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumDiscriminants)]
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
    Civilize { target: AnimalId },
}

impl PlanetEvent {
    pub fn kind(&self) -> PlanetEventKind {
        self.into()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Params {
    pub sim: SimParams,
    pub event: EventParams,
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
    #[serde(skip)]
    pub animals: HashMap<AnimalId, AnimalAttr>,
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
    pub initial_conditions: Vec<InitialCondition>,
    pub height_table: Vec<(f32, f32)>,
    pub target_sea_level: Option<f32>,
}

#[serde_as]
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
    /// Albedo by cloud table
    pub cloud_albedo_table: Vec<(f32, f32)>,
    /// Aerosol remaining rate per one cycle
    pub aerosol_remaining_rate: f32,
    /// Aerosol to cloud table
    pub aerosol_cloud_table: Vec<(f32, f32)>,
    /// Greeh house effect table of CO2
    pub co2_green_house_effect_table: Vec<(f32, f32)>,
    /// Greeh house effect table of cloud
    pub cloud_green_house_effect_table: Vec<(f32, f32)>,
    /// Greeh house effect decrease by height at 1atm
    pub green_house_effect_height_decrease: f32,
    /// The number of loop of vapor transfer calculation
    pub n_loop_vapor_calc: usize,
    /// The ratio of tile vapor diffusion
    pub vapor_diffusion_factor: f32,
    /// Coefficent to adjust vapor diffusion by height difference
    pub coeff_vapor_diffusion_adjust_by_h_diff: f32,
    /// The ratio of vapor loss
    pub vapor_loss_ratio: f32,
    /// Vaporizaion from ocean tile - °C table
    #[serde_as(as = "Vec<(Celsius, Same)>")]
    pub ocean_vaporization_table: Vec<(f32, f32)>,
    /// Drying factors for humidity calculation (humidity = rainfall - factors.0 * (temperature + factors.1))
    pub drying_factors: (f32, f32),
    /// Max fertility table by temperature
    #[serde_as(as = "Vec<(Celsius, Same)>")]
    pub temperature_fertility_table: Vec<(f32, f32)>,
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
    /// Max biomass by humidity
    pub max_biomass_humidity_table: Vec<(f32, f32)>,
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
    /// Ice melting temperature [K]
    pub ice_melting_temp: f32,
    /// Ice melting speed [m/K]
    pub ice_melting_height_per_temp: f32,
    /// Table to calculate (rainfall - 0.1 * temp[°C]) -> ice thickness limit [m]
    pub ice_thickness_limit_table: Vec<(f32, f32)>,
    /// Factor for adding ice height from rainfall [m/(rainfall)mm]
    pub fallen_snow_factor: f32,
    /// Biome transition probability before start simulation
    pub before_start_biome_transition_probability: f32,
    /// Animal simulation interval cycles
    pub animal_sim_interval: u32,
    /// Animal extinction threshold
    pub animal_extinction_threshold: f32,
    /// Basic animal growth speed
    pub animal_growth_speed: f32,
    /// Maximum animal growth speed
    pub animal_growth_speed_max: f32,
    /// Animal capacity effect by biomass
    pub animal_cap_max_biomass: f32,
    /// Animal capacity effect by fertility in sea tiles
    pub animal_cap_max_fertility: f32,
    /// Probability of animal moving
    pub animal_move_weight: f64,
    /// Coefficent to calculate animal fission probability
    pub coef_animal_fisson_a: f32,
    /// Coefficent to calculate animal fission probability
    pub coef_animal_fisson_b: f32,
    /// Coefficent to calculate animal random kill probability by congestion rate
    pub coef_animal_kill_by_congestion_a: f32,
    /// Coefficent to calculate animal random kill probability by congestion rate
    pub coef_animal_kill_by_congestion_b: f32,
    /// Animal livable oxygen range by size
    pub livable_oxygen_range: [(f32, f32); AnimalSize::LEN],
    /// Coefficent to calulate gene point income.
    pub coef_gene_point_income: f32,
    /// Initial population of settlements
    pub settlement_init_pop: [f32; CivilizationAge::LEN],
    /// Max population of settlements
    pub settlement_max_pop: [f32; CivilizationAge::LEN],
    /// Livable temperature bonus by civilization
    pub civ_temp_bonus: [f32; CivilizationAge::LEN],
    /// Population of settlements to calulate spread probability
    pub settlement_spread_pop: [f32; CivilizationAge::LEN],
    /// Base population growth speed
    pub base_pop_growth_speed: f32,
    /// Coefficent to calculate settlement spreading probability
    pub coef_settlement_spreading_a: f32,
    /// Coefficent to calculate animal fission probability
    pub coef_settlement_spreading_b: f32,
    /// Settlement population to extinction
    pub settlement_extinction_threshold: f32,
    /// Energy demand per pop [GJ]
    pub energy_demand_per_pop: [f32; CivilizationAge::LEN],
    /// Consumed biomass to energy factor [GJ/Mt]
    pub biomass_energy_factor: f32,
    /// Resource availability factor
    pub resource_availability_factor: f32,
    /// Base tech exp
    pub base_tech_exp: f32,
    /// Required tech exp to evolve the age
    pub tech_exp_evolution: [f32; CivilizationAge::LEN - 1],
    /// Rainfall to hydro energy source table [mm] - [GJ/m^2]
    pub table_rainfall_hydro: Vec<(f32, f32)>,
    /// Available geothermal ratio by civilization
    pub available_geothermal_ratio: f32,
    /// Solar constant to wind & solar energy source table [W/m^2] - [GJ/m^2]
    pub table_solar_constant_wind_solar: Vec<(f32, f32)>,
    /// Energy source limit by settlement age
    pub energy_source_limit_by_age: [[f32; EnergySource::LEN]; CivilizationAge::LEN],
    /// Factor to calculate impact on biomass by energy source
    pub energy_source_biomass_impact: [f32; EnergySource::LEN],
    /// Duration of events
    pub event_duration: HashMap<PlanetEventKind, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventParams {
    /// Resource cost for tile event
    pub tile_event_costs: BTreeMap<TileEventKind, Cost>,
    /// The ratio of biomass burn at one cycle
    pub fire_burn_ratio: f32,
    /// Biomass at fire extinction [kg/m2]
    pub biomass_at_fire_extinction_range: (f32, f32),
    /// Aerosol supply by fire
    pub fire_aerosol: f32,
    /// Black dust albedo
    pub black_dust_albedo: f32,
    /// Black dust cycles
    pub black_dust_cycles: u32,
    /// Additional decrease of black dust cycles by rainfall
    pub black_dust_decrease_by_rainfall: f32,
    /// Aerosol injection cycles
    pub aerosol_injection_cycles: u32,
    /// Aerosol injection amount
    pub aerosol_injection_amount: f32,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Cost {
    /// Needed surplus energy and cycles
    Energy(f32, u32),
    Material(f32),
    GenePoint(f32),
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
    pub habitability: PlanetHabitability,
    /// Planet radius [km]
    pub radius: (f32, f32),
    pub solar_constant: (f32, f32),
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub geothermal_power: Option<(f32, f32)>,
    pub elevation: (f32, f32),
    pub water_volume: (f32, f32),
    pub nitrogen: (f32, f32),
    pub carbon_dioxide: (f32, f32),
    pub initial_conditions: Vec<InitialCondition>,
    #[serde(default)]
    pub height_table: Vec<(f32, f32)>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    pub target_sea_level: Option<f32>,
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, EnumString, AsRefStr,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "kebab-case")]
pub enum PlanetHabitability {
    Ideal,
    Adequate,
    Poor,
    Hostile,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitoringParams {
    pub interval_cycles: u64,
    pub warn_high_temp_threshold: f32,
    pub warn_low_temp_threshold: f32,
    pub warn_low_oxygen_threshold: f32,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum InitialCondition {
    Snowball { thickness: (f32, f32) },
}
