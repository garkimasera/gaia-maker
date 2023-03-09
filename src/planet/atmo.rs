use super::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

const CO2_CARBON_WEIGHT_RATIO: f32 = 44.0 / 12.0;

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

    pub fn partial_pressure(&self, kind: GasKind) -> f32 {
        self.atm * self.mass[&kind] / self.total_mass()
    }

    pub fn remove_carbon(&mut self, value: f32) -> bool {
        let value = value * CO2_CARBON_WEIGHT_RATIO;
        let co2_mass = self.mass.get_mut(&GasKind::CarbonDioxide).unwrap();
        if *co2_mass > value {
            *co2_mass -= value;
            true
        } else {
            false
        }
    }

    pub fn release_carbon(&mut self, value: f32) {
        *self.mass.get_mut(&GasKind::CarbonDioxide).unwrap() += value * CO2_CARBON_WEIGHT_RATIO;
    }
}

pub fn sim_atmosphere(planet: &mut Planet, params: &Params) {
    planet.atmo.atm = planet.atmo.total_mass() / params.sim.total_mass_per_atm;
}
