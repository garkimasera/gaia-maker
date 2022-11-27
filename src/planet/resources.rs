use super::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resources {
    pub stock: ResourceMap,
    pub cap: ResourceMap,
}

impl Default for Resources {
    fn default() -> Self {
        Resources {
            stock: ResourceKind::iter().map(|kind| (kind, 0.0)).collect(),
            cap: ResourceKind::iter().map(|kind| (kind, 1.0E+06)).collect(),
        }
    }
}

impl Resources {
    pub fn new(start_params: &StartParams) -> Self {
        let mut res = Resources::default();

        for (kind, v) in &start_params.resources {
            *res.get_stock_mut(*kind) += v;
        }

        res
    }

    pub fn get_stock_mut(&mut self, kind: ResourceKind) -> &mut f32 {
        self.stock.get_mut(&kind).unwrap()
    }

    pub fn remove_by_map(&mut self, map: &ResourceMap) {
        for (kind, v) in map {
            *self.get_stock_mut(*kind) -= v;
        }
    }
}
