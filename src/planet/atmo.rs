use super::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    pub atm: f32,
    /// Gases mass [Mt]
    pub mass: FnvHashMap<GasKind, f32>,
}

impl Atmosphere {
    pub fn new(start_params: &StartParams) -> Self {
        Atmosphere {
            atm: 0.0,
            mass: start_params.atmo_mass.clone(),
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.mass.values().sum()
    }
}

pub fn sim_atmosphere(planet: &mut Planet, params: &Params) {
    planet.atmo.atm = planet.atmo.total_mass() / params.sim.total_mass_per_atm;
}
