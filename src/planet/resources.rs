use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resources {
    pub energy: f32,
    pub used_energy: f32,
    pub stock: ResourceMap,
    pub cap: ResourceMap,
    pub diff: ResourceMap,
}

pub fn empty_resource_map() -> ResourceMap {
    ResourceKind::iter().map(|kind| (kind, 0.0)).collect()
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            energy: 0.0,
            used_energy: 0.0,
            stock: empty_resource_map(),
            cap: ResourceKind::iter().map(|kind| (kind, 1.0E+06)).collect(),
            diff: empty_resource_map(),
        }
    }
}

impl Resources {
    pub fn new(start_params: &StartParams) -> Self {
        let mut res = Resources::default();

        for (kind, v) in &start_params.resources {
            *res.stock.get_mut(kind).unwrap() += v;
        }

        res
    }

    pub fn add(&mut self, kind: ResourceKind, value: f32) {
        let v = self.stock.get_mut(&kind).unwrap();
        *v = (*v + value).clamp(0.0, self.cap[&kind]);
    }

    pub fn remove_by_map(&mut self, map: &ResourceMap) {
        for (&kind, &v) in map {
            self.add(kind, -v);
        }
    }

    pub fn surplus_energy(&self) -> f32 {
        self.energy - self.used_energy
    }

    pub fn reset(&mut self, start_params: &StartParams) {
        let initial = Self::new(start_params);

        self.stock = initial.stock;
        self.cap = initial.cap;
        self.diff = initial.diff;
    }
}
