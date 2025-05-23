use fnv::FnvHashSet;
use rand::rngs::SmallRng;

use super::achivement::Achivement;
use super::*;
use std::collections::{BTreeMap, HashMap};
use std::f32::consts::PI;

/// Holds data for simulation
pub struct Sim {
    /// Fast rng for simulation
    pub rng: SmallRng,
    /// Before start simulation or not
    pub before_start: bool,
    /// Planet size
    pub size: (u32, u32),
    /// Tile area [m^2]
    pub tile_area: f32,
    /// The number of tiles
    pub n_tiles: u32,
    /// Area factor relative to a planet of default size
    pub planet_area_factor: f32,
    /// Geothermal power per tile [W]
    pub geothermal_power_per_tile: f32,
    /// Tile insolation [J/m^2]
    pub insolation: Array2d<f32>,
    /// Solar constant at last insolation calculation
    pub solar_constant_before: f32,
    /// Atmosphere temperature
    pub atemp: Array2d<f32>,
    /// Atmosphere and surface temperature (used for calculation)
    pub atemp_new: Array2d<f32>,
    /// Atmosphere and surface heat capacity [J/K]
    pub atmo_heat_cap: Array2d<f32>,
    /// Sea temperature
    pub stemp: Array2d<f32>,
    /// Sea heat capacity [J/K]
    pub sea_heat_cap: Array2d<f32>,
    /// Tile albedo
    pub albedo: Array2d<f32>,
    /// Vapor in air
    pub vapor: Array2d<f32>,
    /// Vapor in air (used for calculation)
    pub vapor_new: Array2d<f32>,
    /// Tile humidity that calculated by adjusting rainfall by temperature
    pub humidity: Array2d<f32>,
    /// Fertility value and effect to tile from structures or other factors
    pub fertility_value_and_effect: Array2d<(f32, f32)>,
    /// Biomass difference in the cycle
    pub diff_biomass: Array2d<f32>,
    /// The number of working buildings
    pub working_buildings: fnv::FnvHashMap<BuildingKind, u32>,
    /// Hydro and geothermal energy source [GJ]
    pub energy_hydro_geothermal: Array2d<f32>,
    /// Used to sum civilization values
    pub civ_sum: CivSum,
    /// Wind and solar energy source [GJ]
    pub energy_wind_solar: f32,
    /// Civilization domain
    pub domain: Array2d<Option<(AnimalId, f32)>>,
    /// Energy efficiency
    pub energy_eff: Array2d<f32>,
    /// Settlement congestion rate
    pub settlement_cr: Array2d<f32>,
    /// Biomass consumption by settlements
    pub biomass_consumption: Array2d<f32>,
    /// Count the number of settlements in war
    pub war_counter: FnvHashMap<u32, u32>,
    /// War target settlements
    pub war_target_settlements: HashMap<AnimalId, (f32, Coords)>,
    /// New achievements
    pub new_achievements: FnvHashSet<Achivement>,
}

impl Sim {
    pub fn new(planet: &Planet, params: &Params) -> Self {
        let size = planet.map.size();
        let n_tiles = size.0 * size.1;
        let map_iter_idx = planet.map.iter_idx();
        let tile_area = 4.0 * PI * planet.basics.radius * planet.basics.radius
            / (size.0 as f32 * size.1 as f32);
        let planet_area_factor =
            (planet.basics.radius / params.default_start_params.basics.radius).powi(2);

        let mut atemp = Array2d::new(size.0, size.1, 0.0);
        let mut vapor = Array2d::new(size.0, size.1, 0.0);

        for p in map_iter_idx {
            atemp[p] = planet.map[p].temp;
            vapor[p] = planet.map[p].vapor;
        }

        Sim {
            rng: misc::get_rng(),
            before_start: false,
            size,
            tile_area,
            n_tiles,
            planet_area_factor,
            geothermal_power_per_tile: planet.basics.geothermal_power / n_tiles as f32,
            insolation: Array2d::new(size.0, size.1, 0.0),
            solar_constant_before: 0.0,
            atemp,
            atemp_new: Array2d::new(size.0, size.1, 0.0),
            atmo_heat_cap: Array2d::new(size.0, size.1, 0.0),
            stemp: Array2d::new(size.0, size.1, 0.0),
            sea_heat_cap: Array2d::new(size.0, size.1, 0.0),
            albedo: Array2d::new(size.0, size.1, 0.0),
            vapor,
            vapor_new: Array2d::new(size.0, size.1, 0.0),
            humidity: Array2d::new(size.0, size.1, 0.0),
            fertility_value_and_effect: Array2d::new(size.0, size.1, (0.0, 0.0)),
            diff_biomass: Array2d::new(size.0, size.1, 0.0),
            working_buildings: HashMap::default(),
            energy_hydro_geothermal: Array2d::new(size.0, size.1, 0.0),
            energy_wind_solar: 0.0,
            civ_sum: CivSum::default(),
            domain: Array2d::new(size.0, size.1, None),
            energy_eff: Array2d::new(size.0, size.1, 0.0),
            settlement_cr: Array2d::new(size.0, size.1, 0.0),
            biomass_consumption: Array2d::new(size.0, size.1, 0.0),
            war_counter: FnvHashMap::default(),
            war_target_settlements: HashMap::default(),
            new_achievements: FnvHashSet::default(),
        }
    }

    /// Return the factor to calculate tile biomass [Mt] from density.
    pub fn biomass_density_to_mass(&self) -> f32 {
        self.tile_area * 1.0e-9
    }

    pub fn convert_p_cyclic(&self, p: Coords) -> Option<Coords> {
        self.coords_converter().conv(p)
    }

    pub fn coords_converter(&self) -> CoordsConverter {
        CoordsConverter(self.size)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CoordsConverter((u32, u32));

impl CoordsConverter {
    pub fn new(planet: &Planet) -> Self {
        Self(planet.map.size())
    }

    pub fn conv(&self, p: Coords) -> Option<Coords> {
        geom::CyclicMode::X.convert_coords(self.0, p)
    }
}

#[derive(Default, Debug)]
pub struct CivSum(HashMap<AnimalId, CivSumValues>);

impl CivSum {
    pub fn iter(&self) -> impl Iterator<Item = (AnimalId, &CivSumValues)> {
        self.0.iter().map(|(key, sum_values)| (*key, sum_values))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (AnimalId, &mut CivSumValues)> {
        self.0.iter_mut().map(|(key, sum_values)| (*key, sum_values))
    }

    pub fn get_mut(&mut self, animal_id: AnimalId) -> &mut CivSumValues {
        self.0.entry(animal_id).or_default()
    }

    pub fn reset(&mut self, ids: impl Iterator<Item = AnimalId>) {
        for value in self.0.values_mut() {
            *value = CivSumValues::default();
        }
        for id in ids {
            self.0.entry(id).or_default();
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct CivSumValues {
    pub total_pop: f64,
    pub total_pop_prev: f64,
    pub total_settlement: [u32; CivilizationAge::LEN],
    pub total_energy_consumption: [f64; EnergySource::LEN],
    pub fossil_fuel_src_tiles: BTreeMap<ordered_float::NotNan<f32>, Coords>,
    pub fossil_fuel_supply: f32,
    pub gift_supply: f32,
    pub n_moving: u32,
}
