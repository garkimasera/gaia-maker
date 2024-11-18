use super::*;
use serde::{Deserialize, Serialize};

const MATERIAL_MAX: f32 = 1.0e+6;
const GENE_POINT_MAX: f32 = 1000.0;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Resources {
    pub energy: f32,
    pub used_energy: f32,
    pub material: f32,
    pub diff_material: f32,
    pub gene_point: f32,
    pub diff_gene_point: f32,
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
        self.gene_point = 0.0;
        self.diff_gene_point = 0.0;
    }

    pub fn apply_diff(&mut self) {
        self.material = (self.material + self.diff_material).min(MATERIAL_MAX);
        self.gene_point = (self.gene_point + self.diff_gene_point).min(GENE_POINT_MAX);
    }
}
