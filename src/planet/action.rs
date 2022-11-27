use super::*;

impl Planet {
    pub fn buildable(&self, building: &BuildingAttrs) -> bool {
        for (kind, v) in &building.cost {
            if *v > self.res.stock[kind] {
                return false;
            }
        }
        true
    }

    pub fn build_orbital_building(&mut self, kind: OrbitalBuildingKind, params: &Params) {
        self.res
            .remove_by_map(&params.orbital_buildings[&kind].cost);
        let building = self.orbit.get_mut(&kind).unwrap();
        building.n += 1;
        building.enabled += 1;
    }

    pub fn build_star_system_building(&mut self, kind: StarSystemBuildingKind, params: &Params) {
        self.res
            .remove_by_map(&params.star_system_buildings[&kind].cost);
        let building = self.star_system.get_mut(&kind).unwrap();
        building.n += 1;
        building.enabled += 1;
    }

    pub fn edit_biome(&mut self, coords: Coords, biome: Biome) {
        self.map[coords].biome = biome;
    }
}
