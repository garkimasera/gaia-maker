use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resources {
    pub energy: f32,
    pub used_energy: f32,
    pub material: f32,
    pub diff_material: f32,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            energy: 0.0,
            used_energy: 0.0,
            material: 0.0,
            diff_material: 0.0,
        }
    }
}

impl Resources {
    pub fn new(start_params: &StartParams) -> Self {
        Self {
            material: start_params.material,
            ..Default::default()
        }
    }

    pub fn surplus_energy(&self) -> f32 {
        self.energy - self.used_energy
    }

    pub fn reset_before_reset(&mut self) {
        self.energy = 0.0;
        self.used_energy = 0.0;
        self.diff_material = 0.0;
    }
}
