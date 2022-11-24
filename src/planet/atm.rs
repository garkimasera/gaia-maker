use super::defs::*;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Atmosphere {
    pub mass: FnvHashMap<GasKind, f32>,
}

impl Atmosphere {
    pub fn from_params(start_params: &StartParams) -> Self {
        Atmosphere {
            mass: start_params.atmo_mass.clone(),
        }
    }
}
