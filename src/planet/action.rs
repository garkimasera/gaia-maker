use super::*;

impl Planet {
    pub fn buildable(&self, building: &BuildingAttrs, n: u32) -> bool {
        if self.res.surplus_energy() < -building.energy {
            return false;
        }
        if building.cost * n as f32 > self.res.material {
            return false;
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

    pub fn place(
        &mut self,
        p: Coords,
        size: StructureSize,
        structure: Structure,
        sim: &mut Sim,
        params: &Params,
    ) {
        assert!(self.placeable(p, size));

        let kind = structure.kind();
        self.map[p].structure = structure;

        for p_rel in size.occupied_tiles().into_iter() {
            self.map[p + p_rel].structure = Structure::Occupied { by: p };
        }

        self.res.material -= params.structures[&kind].building.cost;
        self.update(sim, params);
    }

    pub fn demolition(&mut self, p: Coords, sim: &mut Sim, params: &Params) {
        self.map[p].structure = Structure::None;
        self.update(sim, params);
    }

    pub fn build_space_building(
        &mut self,
        kind: impl Into<SpaceBuildingKind>,
        sim: &mut Sim,
        params: &Params,
    ) {
        let kind = kind.into();
        let cost = &params.building_attrs(BuildingKind::Space(kind)).cost;
        self.res.material -= cost;
        let building = self.space_building_mut(kind);
        building.n += 1;

        if let BuildingControlValue::EnabledNumber(enabled) = &mut building.control {
            *enabled += 1;
        } else if building.n == 1 {
            // Set initial control value at the first build
            match params.building_attrs(kind).control {
                BuildingControl::AlwaysEnabled => (),
                BuildingControl::EnabledNumber => {
                    building.control = BuildingControlValue::EnabledNumber(1);
                }
                BuildingControl::IncreaseRate => {
                    building.control = BuildingControlValue::IncreaseRate(0);
                }
            }
        }
        self.update(sim, params);
    }

    pub fn edit_biome(&mut self, coords: Coords, biome: Biome) {
        self.map[coords].biome = biome;
    }

    pub fn place_settlement(&mut self, coords: Coords, settlement: Settlement) {
        self.map[coords].structure = Structure::Settlement { settlement };
    }
}
