use super::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum BuildingKind {
    Structure(StructureKind),
    Orbital(OrbitalBuildingKind),
    StarSystem(StarSystemBuildingKind),
}

pub fn advance(planet: &mut Planet, params: &Params) {
    let c = CheckUpkeepProduces::new(planet, params);
    planet.res.stock = c.stock;
    planet.res.diff = c.diff;

    apply_building_effect(planet, &c.stopped_buildings, params);
}

#[derive(Default)]
struct CheckUpkeepProduces {
    stock: ResourceMap,
    diff: ResourceMap,
    stopped_buildings: FnvHashMap<BuildingKind, u32>,
}

impl CheckUpkeepProduces {
    fn new(planet: &Planet, params: &Params) -> Self {
        let mut c = CheckUpkeepProduces {
            stock: planet.res.stock.clone(),
            ..Default::default()
        };

        for (kind, b) in &planet.orbit {
            let building = &params.orbital_buildings[kind];
            c.check_building(BuildingKind::Orbital(*kind), b.enabled, building, planet);
        }

        for (kind, b) in &planet.star_system {
            let building = &params.star_system_buildings[kind];
            c.check_building(BuildingKind::StarSystem(*kind), b.enabled, building, planet);
        }

        for tile in planet.map.iter() {
            let kind = tile.structure.kind();
            if let Some(a) = params.structures.get(&kind) {
                c.check_building(BuildingKind::Structure(kind), 1, a.as_ref(), planet);
            }
        }

        c
    }

    fn check_building(
        &mut self,
        kind: BuildingKind,
        n: u32,
        building: &BuildingAttrs,
        planet: &Planet,
    ) {
        let available_by_upkeep = building
            .upkeep
            .iter()
            .map(|(resource_kind, v)| self.stock[resource_kind] / v)
            .min_by(|a, b| a.total_cmp(b));

        let available_by_produce = building
            .produces
            .iter()
            .map(|(resource_kind, v)| {
                (planet.res.cap[resource_kind] - self.stock[resource_kind]) / v
            })
            .min_by(|a, b| a.total_cmp(b));

        let n_available = match (available_by_upkeep, available_by_produce) {
            (None, None) => n,
            (Some(a), None) | (None, Some(a)) => a.clamp(0.0, n as f32) as u32,
            (Some(a), Some(b)) => {
                let a = a.min(b);
                a.clamp(0.0, n as f32) as u32
            }
        };

        *self.stopped_buildings.entry(kind).or_default() += n - n_available;

        let a = n_available as f32;
        for (resource_kind, v) in &building.upkeep {
            *self.diff.entry(*resource_kind).or_default() -= *v * a;
            *self.stock.get_mut(resource_kind).unwrap() -= *v * a;
        }

        for (resource_kind, v) in &building.produces {
            *self.diff.entry(*resource_kind).or_default() += *v * a;
            *self.stock.get_mut(resource_kind).unwrap() += *v * a;
        }
    }
}

fn apply_building_effect(
    planet: &mut Planet,
    stopped_buildings: &FnvHashMap<BuildingKind, u32>,
    params: &Params,
) {
    for (&orbital_building_kind, Building { enabled, .. }) in &planet.orbit {
        if let Some(effect) = &params.orbital_buildings[&orbital_building_kind].effect {
            let n = enabled
                - stopped_buildings
                    .get(&BuildingKind::Orbital(orbital_building_kind))
                    .copied()
                    .unwrap_or(0);

            match effect {
                BuildingEffect::SprayToAtmo { kind, mass } => {
                    *planet.atmo.mass.get_mut(kind).unwrap() += mass * n as f32;
                }
                BuildingEffect::Heater { .. } => (),
            }
        }
    }
}
