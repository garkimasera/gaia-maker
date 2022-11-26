use super::*;

impl Planet {
    pub fn advance(&mut self, params: &Params) {
        self.tick += 1;

        let mut energy_upkeep = 0.0;
        let mut energy_product = 0.0;
        let mut material_upkeep = 0.0;
        let mut material_product = 0.0;

        for tile in self.map.iter() {
            if let Some(a) = params.structures.get(&StructureKind::from(&tile.structure)) {
                energy_upkeep += *a.upkeep.get(&ResourceKind::Energy).unwrap_or(&0.0);
                material_upkeep += *a.upkeep.get(&ResourceKind::Material).unwrap_or(&0.0);
                energy_product += *a.produces.get(&ResourceKind::Energy).unwrap_or(&0.0);
                material_product += *a.produces.get(&ResourceKind::Material).unwrap_or(&0.0);
            }
        }

        self.res.energy += (energy_product - energy_upkeep) * SPEED;
        self.res.material += (material_product - material_upkeep) * SPEED;
    }
}
