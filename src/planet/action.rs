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

    pub fn placeable(&self, p: Coords, size: StructureSize) -> bool {
        if !self.map.in_range(p) {
            return false;
        }

        for p in size.occupied_tiles().into_iter() {
            if let Some(tile) = self.map.get(p) {
                if !matches!(tile.structure, Structure::None) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn place(&mut self, p: Coords, size: StructureSize, structure: Structure, params: &Params) {
        assert!(self.placeable(p, size));

        let kind = structure.kind();
        self.map[p].structure = structure;

        for p_rel in size.occupied_tiles().into_iter() {
            self.map[p + p_rel].structure = Structure::Occupied { by: p };
        }

        self.res
            .remove_by_map(&params.structures[&kind].building.cost);
    }

    pub fn demolition(&mut self, p: Coords) {
        self.map[p].structure = Structure::None;
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
