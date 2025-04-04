use std::sync::LazyLock;

use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use super::misc::linear_interpolation;
use super::*;

const MOLECULAR_WEIGHT_N2: f32 = 28.0;
const MOLECULAR_WEIGHT_O2: f32 = 32.0;
const MOLECULAR_WEIGHT_CO2: f32 = 44.0;
const MOLECULAR_WEIGHT_ARGON: f32 = 40.0;

pub const CO2_CARBON_WEIGHT_RATIO: f32 = MOLECULAR_WEIGHT_CO2 / 12.0;
pub const CO2_OXYGEN_WEIGHT_RATIO: f32 = MOLECULAR_WEIGHT_CO2 / MOLECULAR_WEIGHT_O2;

static GAS_MOLECULAR_WEIGHT: LazyLock<FnvHashMap<GasKind, f32>> = LazyLock::new(|| {
    let mut map = FnvHashMap::default();
    map.insert(GasKind::Nitrogen, MOLECULAR_WEIGHT_N2);
    map.insert(GasKind::Oxygen, MOLECULAR_WEIGHT_O2);
    map.insert(GasKind::CarbonDioxide, MOLECULAR_WEIGHT_CO2);
    map.insert(GasKind::Argon, MOLECULAR_WEIGHT_ARGON);
    map
});

const AEROSOL_EQUILIBRIUM_TARGET: f32 = 1.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    atm: f32,
    /// Gases mass [Mt]
    mass: FnvHashMap<GasKind, f64>,
    /// Gass mole ratio
    pub mole_ratio: FnvHashMap<GasKind, f32>,
    /// Cloud amount [%]. The average is 50%
    pub cloud_amount: f32,
    /// Aerosol amount
    pub aerosol: f32,
}

impl Atmosphere {
    pub fn new(start_params: &StartParams, params: &Params) -> Self {
        let mass = start_params
            .atmo
            .iter()
            .map(|(gas_kind, atm)| {
                (
                    *gas_kind,
                    atm * params.sim.mol_per_atm as f64 * GAS_MOLECULAR_WEIGHT[gas_kind] as f64,
                )
            })
            .collect();
        let mole_ratio = GasKind::iter().map(|gas_kind| (gas_kind, 0.0)).collect();

        Atmosphere {
            atm: 0.0,
            mass,
            mole_ratio,
            cloud_amount: 50.0,
            aerosol: AEROSOL_EQUILIBRIUM_TARGET,
        }
    }

    pub fn total_mass(&self) -> f32 {
        self.mass.values().sum::<f64>() as f32
    }

    pub fn atm(&self) -> f32 {
        self.atm
    }

    pub fn partial_pressure(&self, kind: GasKind) -> f32 {
        self.atm * self.mole_ratio[&kind]
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

pub fn sim_atmosphere(planet: &mut Planet, _sim: &mut Sim, params: &Params) {
    let mut atmo_mole = FnvHashMap::default();
    let mut sum_mole = 0.0;
    for gas_kind in GasKind::iter() {
        let mole = planet.atmo.mass[&gas_kind] as f32 / GAS_MOLECULAR_WEIGHT[&gas_kind];
        atmo_mole.insert(gas_kind, mole);
        sum_mole += mole;
    }
    for gas_kind in GasKind::iter() {
        planet
            .atmo
            .mole_ratio
            .insert(gas_kind, atmo_mole[&gas_kind] / sum_mole);
    }
    planet.atmo.atm = sum_mole / params.sim.mol_per_atm;

    // Aerosol
    let base_supply = (1.0 - params.sim.aerosol_remaining_rate) * AEROSOL_EQUILIBRIUM_TARGET;
    planet.atmo.aerosol += base_supply;
    planet.atmo.aerosol *= params.sim.aerosol_remaining_rate;

    planet.atmo.cloud_amount =
        linear_interpolation(&params.sim.aerosol_cloud_table, planet.atmo.aerosol);
}
