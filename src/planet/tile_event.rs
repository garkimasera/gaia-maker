use super::misc::ConstantDist;
use super::*;

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(event) = &mut planet.map[p].event else {
            continue;
        };

        match **event {
            TileEvent::Fire => {
                let biomass = planet.map[p].biomass;
                let burned_biomass = biomass * params.event.fire_burn_ratio;
                let biomass = biomass - burned_biomass;
                planet.map[p].biomass = biomass;
                let burned_biomass = sim.biomass_density_to_mass();
                let extinction_biomass = sim.rng.sample(ConstantDist::from(
                    params.event.biomass_at_fire_extinction_range,
                ));
                if biomass <= extinction_biomass {
                    planet.map[p].event = None;
                    planet.atmo.release_carbon(burned_biomass);
                }
                planet.atmo.aerosol += params.event.fire_aerosol;
            }
            TileEvent::Plague => todo!(),
        }
    }
}

pub fn cause_tile_event(
    planet: &mut Planet,
    p: Coords,
    kind: TileEventKind,
    _sim: &mut Sim,
    _params: &Params,
) {
    let event = match kind {
        TileEventKind::Fire => TileEvent::Fire,
        TileEventKind::Plague => todo!(),
    };

    planet.map[p].event = Some(Box::new(event));
}
