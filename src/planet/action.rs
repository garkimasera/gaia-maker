use compact_str::CompactString;

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

    pub fn placeable(&self, p: Coords) -> bool {
        if !self.map.in_range(p) {
            return false;
        }
        self.map[p].structure.is_none()
    }

    pub fn place(&mut self, p: Coords, structure: Structure, sim: &mut Sim, params: &Params) {
        assert!(self.placeable(p));

        let kind = structure.kind();
        self.map[p].structure = Some(structure);

        self.res.material -= params.structures[&kind].building.cost;
        self.update(sim, params);
    }

    pub fn demolition(&mut self, p: Coords, sim: &mut Sim, params: &Params) {
        self.map[p].structure = None;
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

    pub fn cause_tile_event(
        &mut self,
        p: Coords,
        kind: TileEventKind,
        sim: &mut Sim,
        params: &Params,
    ) {
        let cost = params.event.tile_event_costs[&kind];
        if self.res.enough_to_consume(cost) {
            self.res.consume(cost);
            super::tile_event::cause_tile_event(self, p, kind, sim, params);
            self.update(sim, params);
        }
    }

    pub fn animal_spawnable(&self, p: Coords, animal_id: &CompactString, params: &Params) -> bool {
        let attr = &params.animals[animal_id];

        self.map[p].animal[attr.size as usize].is_none() && attr.cost <= self.res.gene_point
    }

    pub fn spawn_animal(&mut self, p: Coords, animal_id: &CompactString, params: &Params) {
        assert!(self.animal_spawnable(p, animal_id, params));

        let attr = &params.animals[animal_id];
        self.res.gene_point -= attr.cost;
        self.map[p].animal[attr.size as usize] = Some(Animal {
            id: animal_id.clone(),
            n: 0.1,
        });
    }

    pub fn edit_biome(&mut self, coords: Coords, biome: Biome) {
        self.map[coords].biome = biome;
    }

    pub fn place_settlement(&mut self, coords: Coords, settlement: Settlement) {
        self.map[coords].structure = Some(Structure::Settlement { settlement });
    }
}
