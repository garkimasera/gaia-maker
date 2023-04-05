use super::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

const CO2_CARBON_WEIGHT_RATIO: f32 = 44.0 / 12.0;
const CO2_OXYGEN_WEIGHT_RATIO: f32 = 44.0 / 32.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    atm: f32,
    /// Gases mass [Mt]
    mass: FnvHashMap<GasKind, f64>,
}

impl Atmosphere {
    pub fn new(start_params: &StartParams) -> Self {
        Atmosphere {
            atm: 0.0,
            mass: start_params.atmo_mass.clone(),
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.mass.values().sum::<f64>() as f32
    }

    pub fn atm(&self) -> f32 {
        self.atm
    }

    pub fn partial_pressure(&self, kind: GasKind) -> f32 {
        self.atm * self.mass[&kind] as f32 / self.total_mass()
    }

    pub fn mass(&self, kind: GasKind) -> f32 {
        *self.mass.get(&kind).unwrap() as f32
    }

    pub fn set_mass(&mut self, kind: GasKind, value: f32) {
        *self.mass.get_mut(&kind).unwrap() = value as f64;
    }

    pub fn add(&mut self, kind: GasKind, value: impl Into<f64>) {
        let mass = self.mass.get_mut(&kind).unwrap();
        *mass = (*mass + value.into()).max(0.0);
    }

    pub fn remove_carbon(&mut self, value: f32) -> bool {
        debug_assert!(value >= 0.0);
        let co2_mass = value * CO2_CARBON_WEIGHT_RATIO;
        let co2_mass_in_atmo = self.mass.get_mut(&GasKind::CarbonDioxide).unwrap();
        if *co2_mass_in_atmo > co2_mass as f64 {
            *co2_mass_in_atmo -= co2_mass as f64;
            self.add(GasKind::Oxygen, value * CO2_OXYGEN_WEIGHT_RATIO);
            true
        } else {
            false
        }
    }

    pub fn release_carbon(&mut self, value: f32) {
        self.add(GasKind::Oxygen, -value * CO2_OXYGEN_WEIGHT_RATIO);
        self.add(GasKind::CarbonDioxide, value * CO2_CARBON_WEIGHT_RATIO);
    }

    pub fn remove_atmo(&mut self, value: impl Into<f64>) {
        let value = value.into();
        let total_mass = self.total_mass() as f64;
        for kind in GasKind::iter() {
            let mass = self.mass.get_mut(&kind).unwrap();
            let v = value * *mass / total_mass;
            *mass = (*mass - v).max(0.0);
        }
    }
}

pub fn sim_atmosphere(planet: &mut Planet, params: &Params) {
    planet.atmo.atm = planet.atmo.total_mass() / params.sim.total_mass_per_atm;
}
