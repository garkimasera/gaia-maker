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
    for p in planet.map.iter_idx() {
        let tile = &mut planet.map[p];
        let tile_events = &mut tile.tile_events;
        if tile_events.0.is_empty() {
            continue;
        }

        if let Some(TileEvent::Fire) = tile_events.get_mut(TileEventKind::Fire) {
            let biomass = tile.biomass;
            let burned_biomass = biomass * params.event.fire_burn_ratio;
            let biomass = biomass - burned_biomass;
            tile.biomass = biomass;
            let burned_biomass = sim.biomass_density_to_mass();
            let extinction_biomass = sim.rng.sample(ConstantDist::from(
                params.event.biomass_at_fire_extinction_range,
            ));
            if biomass <= extinction_biomass {
                tile_events.remove(TileEventKind::Fire);
                planet.atmo.release_carbon(burned_biomass);
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
    }
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
    };

    planet.map[p].tile_events.insert(event);
}
