use super::misc::ConstantDist;
use super::*;

#[derive(Clone, Default, Debug, serde::Serialize, serde::Deserialize)]
pub struct TileEvents(smallvec::SmallVec<[TileEvent; 4]>);

impl TileEvents {
    pub fn insert(&mut self, tile_event: TileEvent) {
        if let Some(e) = self.get_mut(tile_event.kind()) {
            *e = tile_event;
        } else {
            self.0.push(tile_event);
        }
    }

    pub fn remove(&mut self, kind: TileEventKind) {
        self.0.retain(|e| e.kind() != kind);
    }

    pub fn contains(&self, kind: TileEventKind) -> bool {
        self.0.iter().any(|e| e.kind() == kind)
    }

    pub fn get(&self, kind: TileEventKind) -> Option<&TileEvent> {
        self.0.iter().find(|e| e.kind() == kind)
    }

    pub fn get_mut(&mut self, kind: TileEventKind) -> Option<&mut TileEvent> {
        self.0.iter_mut().find(|e| e.kind() == kind)
    }

    pub fn retain<F: FnMut(&mut TileEvent) -> bool>(&mut self, f: F) {
        self.0.retain(f);
    }

    pub fn list(&self) -> &[TileEvent] {
        &self.0
    }
}

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    sim.war_counter.clear();

    for tile in planet.map.iter_mut() {
        let tile_events = &mut tile.tile_events;
        if tile_events.0.is_empty() {
            continue;
        }

        if !matches!(tile.structure, Some(Structure::Settlement(_))) {
            tile_events.retain(|te| !te.is_settlement_event());
        }

        if let Some(TileEvent::Fire) = tile_events.get_mut(TileEventKind::Fire) {
            let biomass = tile.biomass;
            let burned_biomass = biomass * params.event.fire_burn_ratio;
            let biomass = biomass - burned_biomass;
            tile.biomass = biomass;
            let burned_biomass = sim.biomass_density_to_mass();
            planet.atmo.release_carbon(burned_biomass);
            let extinction_biomass = sim.rng.sample(ConstantDist::from(
                params.event.biomass_at_fire_extinction_range,
            ));
            if biomass <= extinction_biomass {
                tile_events.remove(TileEventKind::Fire);
            }
            planet.atmo.aerosol += params.event.fire_aerosol;

            if matches!(tile.structure, Some(Structure::Settlement(_))) {
                tile.structure = None;
            }
            tile.animal = [None; AnimalSize::LEN];
        }

        if let Some(TileEvent::BlackDust { remaining_cycles }) =
            tile_events.get_mut(TileEventKind::BlackDust)
        {
            let rainfall = tile.rainfall;
            let remaining_cycles_decrease =
                (rainfall / params.event.black_dust_decrease_by_rainfall) as u32 + 1;
            if *remaining_cycles < remaining_cycles_decrease {
                tile_events.remove(TileEventKind::BlackDust);
            } else {
                *remaining_cycles -= remaining_cycles_decrease;
            }
        }

        if let Some(TileEvent::AerosolInjection { remaining_cycles }) =
            tile_events.get_mut(TileEventKind::AerosolInjection)
        {
            *remaining_cycles -= 1;
            if *remaining_cycles == 0 {
                tile_events.remove(TileEventKind::AerosolInjection);
            }
            planet.atmo.aerosol += params.event.aerosol_injection_amount;
        }

        if let Some(TileEvent::War {
            i: id, offence_str, ..
        }) = tile_events.get_mut(TileEventKind::War)
        {
            if let Some(Structure::Settlement(settlement)) = &mut tile.structure {
                let (damage, finished) =
                    super::war::exec_combat(&mut settlement.str, offence_str, params);
                settlement.pop = (settlement.pop
                    - damage * params.event.coef_pop_decrease_by_combat_damage)
                    .max(0.0);
                *sim.war_counter.entry(*id).or_default() += 1;
                if finished {
                    settlement.change_state_after_bad_event(sim, params);
                    tile_events.remove(TileEventKind::War);
                }
            } else {
                tile_events.remove(TileEventKind::War);
            }
        }

        if let Some(TileEvent::NuclearExplosion { remaining_cycles }) =
            tile_events.get_mut(TileEventKind::NuclearExplosion)
        {
            let biomass = tile.biomass;
            let burned_biomass = biomass * params.event.nuclear_explosion_biomass_burn_ratio;
            let biomass = biomass - burned_biomass;
            tile.biomass = biomass;
            let burned_biomass = sim.biomass_density_to_mass();
            planet.atmo.release_carbon(burned_biomass);
            planet.atmo.aerosol += params.event.nuclear_explosion_aerosol;

            if matches!(tile.structure, Some(Structure::Settlement(_))) {
                tile.structure = None;
            }
            tile.animal = [None; AnimalSize::LEN];
            *remaining_cycles -= 1;
            if *remaining_cycles == 0 {
                tile_events.remove(TileEventKind::NuclearExplosion);
            }
        }
    }

    advance_vehicle(planet, sim, params);
    super::war::advance_troops(planet, sim, params);
}

pub fn cause_tile_event(
    planet: &mut Planet,
    p: Coords,
    kind: TileEventKind,
    sim: &mut Sim,
    params: &Params,
) {
    let event = match kind {
        TileEventKind::Fire => TileEvent::Fire,
        TileEventKind::BlackDust => TileEvent::BlackDust {
            remaining_cycles: params.event.black_dust_cycles,
        },
        TileEventKind::AerosolInjection => TileEvent::AerosolInjection {
            remaining_cycles: params.event.aerosol_injection_cycles,
        },
        TileEventKind::Plague => {
            if let Some(Structure::Settlement(_)) = &mut planet.map[p].structure {
                super::plague::cause_plague(planet, sim, params, p);
            }
            return;
        }
        _ => unreachable!(),
    };

    planet.map[p].tile_events.insert(event);
}

fn advance_vehicle(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let mut moved_vehicles = Vec::new();

    for p_prev in planet.map.iter_idx() {
        let Some(TileEvent::Vehicle {
            kind,
            id,
            age,
            direction,
        }) = planet.map[p_prev]
            .tile_events
            .get(TileEventKind::Vehicle)
            .copied()
        else {
            continue;
        };
        planet.map[p_prev].tile_events.remove(TileEventKind::Vehicle);
        let dy = if sim.rng.random_bool(params.event.vehicle_ns_move_prob) {
            direction.1
        } else {
            0
        };
        let Some(p) = sim.convert_p_cyclic(p_prev + (direction.0 as i32, dy as i32)) else {
            continue;
        };
        if planet.map[p].structure.is_some() {
            continue;
        }

        let animal_attr = &params.animals[&id];
        let civ_sum_values = sim.civ_sum.get_mut(id);

        if !animal_attr.habitat.match_biome(planet.map[p].biome) {
            moved_vehicles.push((
                p,
                TileEvent::Vehicle {
                    kind,
                    id,
                    age,
                    direction,
                },
            ));
            civ_sum_values.total_pop_prev += 1.0;
            civ_sum_values.n_moving += 1;
        } else {
            // Build settlement if habitable
            let cap_animal = super::animal::calc_cap_by_atmo_temp(
                planet,
                p,
                animal_attr,
                params,
                params.sim.civ_temp_bonus[age as usize],
            );
            if sim.settlement_cr[p]
                < params.sim.base_settlement_spreading_threshold
                    * (planet.map[p].fertility / 100.0)
                    * cap_animal
                    * params.event.vehicle_settlement_penalty
            {
                planet.map[p].structure = Some(Structure::Settlement(Settlement {
                    id,
                    pop: params.sim.settlement_init_pop[age as usize],
                    age,
                    ..Default::default()
                }));
            } else if kind == VehicleKind::AirPlane {
                moved_vehicles.push((
                    p,
                    TileEvent::Vehicle {
                        kind,
                        id,
                        age,
                        direction,
                    },
                ));
                civ_sum_values.total_pop_prev += 1.0;
                civ_sum_values.n_moving += 1;
            }
        }
    }

    for (p, vehicle) in moved_vehicles {
        planet.map[p].tile_events.insert(vehicle);
    }
}

impl TileEvent {
    pub fn is_settlement_event(&self) -> bool {
        matches!(
            self,
            Self::War { .. } | Self::Decadence { .. } | Self::Plague { .. }
        )
    }
}
