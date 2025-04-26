use super::*;

impl Planet {
    pub fn buildable(&self, building: &BuildingAttrs) -> Result<(), Cost> {
        if self.res.surplus_power() < -building.power {
            return Err(Cost::Power(building.power, 0));
        }
        if building.cost > self.res.material {
            return Err(Cost::Material(building.cost));
        }
        Ok(())
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

    pub fn demolition(&mut self, p: Coords, sim: &mut Sim, params: &Params) -> bool {
        let structure = &mut self.map[p].structure;
        if structure.is_some() && !matches!(structure, Some(Structure::Settlement(_))) {
            *structure = None;
            self.update(sim, params);
            true
        } else {
            false
        }
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

        if building.n == 1 {
            // Set initial control value at the first build
            match params.building_attrs(kind).control {
                BuildingControl::AlwaysEnabled => (),
                BuildingControl::IncreaseRate => {
                    building.control = BuildingControlValue::IncreaseRate(0);
                }
            }
        }
        self.update(sim, params);
    }

    pub fn demolish_space_building(
        &mut self,
        kind: SpaceBuildingKind,
        n: u32,
        sim: &mut Sim,
        params: &Params,
    ) {
        let building = self.space_building_mut(kind);
        if building.n > n {
            building.n -= n;
        } else {
            building.n = 0;
        }

        if building.n == 0 {
            building.control = Default::default();
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

    pub fn animal_spawnable(&self, p: Coords, animal_id: AnimalId, params: &Params) -> bool {
        let attr = &params.animals[&animal_id];

        self.map[p].animal[attr.size as usize].is_none() && attr.cost <= self.res.gene_point
    }

    pub fn spawn_animal(&mut self, p: Coords, animal_id: AnimalId, params: &Params) {
        assert!(self.animal_spawnable(p, animal_id, params));

        let attr = &params.animals[&animal_id];
        self.res.gene_point -= attr.cost;
        self.map[p].animal[attr.size as usize] = Some(Animal {
            id: animal_id,
            n: 0.1,
        });
    }

    pub fn civilize_animal(&mut self, animal_id: AnimalId, params: &Params) {
        let Some(civ) = &params.animals[&animal_id].civ else {
            unreachable!()
        };
        self.res.consume(Cost::GenePoint(civ.civilize_cost));
        let event = PlanetEvent::Civilize { target: animal_id };
        self.events.start_event(event, params.event.civilize_cycles);
    }
}
