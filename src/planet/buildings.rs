use super::{misc::linear_interpolation, *};
use fnv::FnvHashMap;

impl Planet {
    pub fn space_building(&self, kind: impl Into<SpaceBuildingKind>) -> &Building {
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
            BuildingEffect::SprayToAtmo { kind, mass } => {
                planet.atmo.add(*kind, mass * n as f32);
            }
            _ => (),
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
        let structure = &planet.map[p].structure;
        let kind = BuildingKind::Structure(structure.kind());
        let Some(building_state) = structure.building_state() else {
            continue;
        };
        if *building_state == StructureBuildingState::Disabled {
            continue;
        }

        *working_buildings.entry(kind).or_insert(0) += 1;
    }
}
