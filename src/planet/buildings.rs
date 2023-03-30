use super::*;
use fnv::FnvHashMap;

impl Planet {
    pub fn space_building(&mut self, kind: impl Into<SpaceBuildingKind>) -> &Building {
        self.space_buildings.get(&kind.into()).unwrap()
    }

    pub fn space_building_mut(&mut self, kind: impl Into<SpaceBuildingKind>) -> &mut Building {
        self.space_buildings.get_mut(&kind.into()).unwrap()
    }

    pub fn working_building_effect<'a>(
        &self,
        p: Coords,
        params: &'a Params,
    ) -> Option<&'a BuildingEffect> {
        if self.map[p].structure.building_state() == Some(&StructureBuildingState::Working) {
            params
                .structures
                .get(&self.map[p].structure.kind())
                .and_then(|s| s.building.effect.as_ref())
        } else {
            None
        }
    }
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let working_buildings = update_upkeep_produce(planet, sim, params);
    apply_building_effect(planet, &working_buildings, params);
}

fn update_upkeep_produce(
    planet: &mut Planet,
    sim: &mut Sim,
    params: &Params,
) -> FnvHashMap<BuildingKind, u32> {
    let mut working_buildings = FnvHashMap::default();
    let mut produce = empty_resource_map();

    for v in planet.res.diff.values_mut() {
        *v = 0.0;
    }

    for kind in SpaceBuildingKind::iter() {
        let Some(attrs) = params.building_attrs(BuildingKind::Space(kind)) else { continue };
        let max = max_workable_buildings(attrs, planet);
        let building = planet.space_building_mut(kind);
        let working = max.min(building.enabled);
        building.working = working;
        sim.working_buildings
            .insert(BuildingKind::Space(kind), working);
        add_upkeep_produce(
            attrs,
            building.enabled,
            working,
            &mut planet.res,
            &mut produce,
        );
        working_buildings.insert(BuildingKind::Space(kind), working);
    }

    for p in planet.map.iter_idx() {
        let structure = &mut planet.map[p].structure;
        let kind = BuildingKind::Structure(structure.kind());
        let Some(building_state) = structure.building_state_mut() else { continue };
        let Some(attrs) = params.building_attrs(kind) else { continue };

        if *building_state == StructureBuildingState::Disabled {
            continue;
        }

        let workable = attrs
            .upkeep
            .iter()
            .all(|(resource_kind, value)| planet.res.stock[resource_kind] > *value);
        let working = if workable { 1 } else { 0 };

        add_upkeep_produce(attrs, 1, working, &mut planet.res, &mut produce);

        if workable {
            *working_buildings.entry(kind).or_insert(0) += 1;
            *building_state = StructureBuildingState::Working;
        } else {
            *building_state = StructureBuildingState::Stopped;
        }
    }

    for (resource_kind, produce) in produce {
        planet.res.add(resource_kind, produce);
    }

    working_buildings
}

fn max_workable_buildings(attrs: &BuildingAttrs, planet: &Planet) -> u32 {
    let n = attrs
        .upkeep
        .iter()
        .map(|(resource_kind, upkeep)| (planet.res.stock[resource_kind] / upkeep) as u32)
        .min()
        .unwrap_or(u32::MAX);

    if let Some(build_max) = attrs.build_max {
        n.min(build_max)
    } else {
        n
    }
}

fn add_upkeep_produce(
    attrs: &BuildingAttrs,
    enabled: u32,
    working: u32,
    res: &mut Resources,
    produce: &mut ResourceMap,
) {
    for (resource_kind, &value) in &attrs.upkeep {
        res.add(*resource_kind, -(value * working as f32));
        *res.diff.get_mut(resource_kind).unwrap() -= value * enabled as f32;
    }

    for (resource_kind, &value) in &attrs.produce {
        *produce.get_mut(resource_kind).unwrap() += value * working as f32;
        *res.diff.get_mut(resource_kind).unwrap() += value * enabled as f32;
    }
}

fn apply_building_effect(
    planet: &mut Planet,
    working_buildings: &FnvHashMap<BuildingKind, u32>,
    params: &Params,
) {
    planet.state.solar_power_multiplier = 1.0;

    for (&kind, &n) in working_buildings {
        let Some(effect) = &params.building_attrs(kind).and_then(|attrs| attrs.effect) else { continue };

        #[allow(clippy::single_match)]
        match effect {
            BuildingEffect::MultiplySolarPower { value } => {
                planet.state.solar_power_multiplier += value * n as f32;
            }
            BuildingEffect::SprayToAtmo { kind, mass } => {
                planet.atmo.add(*kind, mass * n as f32);
            }
            _ => (),
        }
    }

    planet.state.solar_power = planet.basics.solar_constant * planet.state.solar_power_multiplier;
}
