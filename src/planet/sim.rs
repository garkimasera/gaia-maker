use super::*;
use std::collections::HashMap;
use std::f32::consts::PI;

/// Holds data for simulation
pub struct Sim {
    /// Before start simulation or not
    pub before_start: bool,
    /// The number of tiles
    pub n_tile: u32,
    /// Tile area [m^2]
    pub tile_area: f32,
    /// Tile insolation [J/m^2]
    pub insolation: Array2d<f32>,
    /// Solar constant at last insolation calculation
    pub solar_constant_before: f32,
    /// Atmosphere temprature
    pub atemp: Array2d<f32>,
    /// Atmosphere and surface temprature (used for calculation)
    pub atemp_new: Array2d<f32>,
    /// Atmosphere and surface heat capacity [J/K]
    pub atmo_heat_cap: Array2d<f32>,
    /// Sea temprature
    pub stemp: Array2d<f32>,
    /// Sea heat capacity [J/K]
    pub sea_heat_cap: Array2d<f32>,
    /// Tile albedo
    pub albedo: Array2d<f32>,
    /// Vapor in air
    pub vapor: Array2d<f32>,
    /// Vapor in air (used for calculation)
    pub vapor_new: Array2d<f32>,
    /// Tile humidity that calculated by adjusting rainfall by temprature
    pub humidity: Array2d<f32>,
    /// Fertility effect to tile from structures or other factors
    pub fertility_effect: Array2d<f32>,
    /// The number of working buildings
    pub working_buildings: HashMap<BuildingKind, u32>,
}

impl Sim {
    pub fn new(planet: &Planet) -> Self {
        let size = planet.map.size();
        let map_iter_idx = planet.map.iter_idx();
        let tile_area = 4.0 * PI * planet.basics.radius * planet.basics.radius
            / (size.0 as f32 * size.1 as f32);

        let mut atemp = Array2d::new(size.0, size.1, 0.0);
        let mut vapor = Array2d::new(size.0, size.1, 0.0);

        for p in map_iter_idx {
            atemp[p] = planet.map[p].temp;
            vapor[p] = planet.map[p].vapor;
        }

        Sim {
            before_start: false,
            n_tile: size.0 * size.1,
            tile_area,
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
            fertility_effect: Array2d::new(size.0, size.1, 0.0),
            working_buildings: HashMap::new(),
        }
    }
}
