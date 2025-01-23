use super::{atmo::CO2_CARBON_WEIGHT_RATIO, misc::linear_interpolation, *};
use fnv::FnvHashMap;

impl Planet {
    pub fn space_building(&self, kind: SpaceBuildingKind) -> &Building {
        self.space_buildings.get(&kind).unwrap()
    }

    pub fn space_building_mut(&mut self, kind: SpaceBuildingKind) -> &mut Building {
        self.space_buildings.get_mut(&kind).unwrap()
    }

    pub fn working_building_effect<'a>(
        &self,
        p: Coords,
        params: &'a Params,
    ) -> Option<&'a BuildingEffect> {
        if let Some(structure) = &self.map[p].structure {
            params
                .structures
                .get(&structure.kind())
                .and_then(|s| s.building.effect.as_ref())
        } else {
            None
        }
    }
}

pub fn update(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    update_working_buildings(planet, &mut sim.working_buildings);

    for (&kind, &n) in &sim.working_buildings {
        if n == 0 {
            continue;
        }
        let attrs = params.building_attrs(kind);
        let control_value = if let BuildingKind::Space(kind) = kind {
            Some(planet.space_building(kind).control)
        } else {
            None
        };
        if attrs.energy > 0.0 {
            planet.res.energy += attrs.energy * n as f32;
        } else {
            planet.res.used_energy += -attrs.energy * n as f32;
        }

        match &attrs.effect {
            Some(BuildingEffect::ProduceMaterial { mass }) => {
                planet.res.diff_material += mass * n as f32;
            }
            Some(BuildingEffect::AdjustSolarPower) => {
                if let Some(BuildingControlValue::IncreaseRate(rate)) = control_value {
                    planet.state.solar_power_multiplier += (rate as f32) / 100.0;
                }
            }
            _ => (),
        }
    }
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for (&kind, &n) in &sim.working_buildings {
        if n == 0 {
            continue;
        }
        let Some(effect) = &params.building_attrs(kind).effect.as_ref() else {
            continue;
        };
        match effect {
            BuildingEffect::RemoveAtmo {
                mass,
                efficiency_table,
            } => {
                let efficiency = linear_interpolation(efficiency_table, planet.atmo.atm());
                planet
                    .atmo
                    .remove_atmo(*mass as f64 * n as f64 * efficiency as f64);
            }
            BuildingEffect::SprayToAtmo {
                kind,
                mass,
                limit_atm,
            } => {
                if planet.atmo.partial_pressure(GasKind::Oxygen) < *limit_atm {
                    planet.atmo.add(*kind, mass * n as f32);
                }
            }
            _ => (),
        }
    }

    for p in planet.map.iter_idx() {
        process_building_on_tile(planet, p, sim, params);
    }
}

fn process_building_on_tile(planet: &mut Planet, p: Coords, sim: &mut Sim, params: &Params) {
    let Some(effect) = planet.map[p]
        .structure
        .as_ref()
        .map(|structure| structure.kind())
        .and_then(|kind| params.building_attrs(kind).effect.as_ref())
    else {
        return;
    };

    if let BuildingEffect::CaptureCarbonDioxide { mass, limit_atm } = effect {
        // Remove co2 from atmosphere, and add buried carbon to one near tile.
        let co2_mass_to_remove =
            (planet.atmo.mass(GasKind::CarbonDioxide) / planet.atmo.total_mass()) * mass;
        let carbon_mass = co2_mass_to_remove / CO2_CARBON_WEIGHT_RATIO;
        planet.atmo.remove_carbon(carbon_mass);

        if planet.atmo.partial_pressure(GasKind::Oxygen) < *limit_atm {
            planet
                .atmo
                .add(GasKind::Oxygen, co2_mass_to_remove - carbon_mass);
        }

        loop {
            let p_target = p + (sim.rng.gen_range(-3..=3), sim.rng.gen_range(-3..=3));
            if planet.map.in_range(p_target) {
                planet.map[p_target].buried_carbon += carbon_mass;
                break;
            }
        }
    }
}

fn update_working_buildings(
    planet: &Planet,
    working_buildings: &mut FnvHashMap<BuildingKind, u32>,
) {
    working_buildings.clear();

    for kind in SpaceBuildingKind::iter() {
        let n = planet.space_building(kind).enabled();
        working_buildings.insert(BuildingKind::Space(kind), n);
    }

    for p in planet.map.iter_idx() {
        if let Some(structure) = &planet.map[p].structure {
            let kind = structure.kind();
            if kind != StructureKind::Settlement {
                *working_buildings
                    .entry(BuildingKind::Structure(kind))
                    .or_insert(0) += 1;
            }
        }
    }
}
