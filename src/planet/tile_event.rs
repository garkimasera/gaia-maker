use super::misc::ConstantDist;
use super::*;

pub fn advance(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(event) = planet.map[p].event.as_ref().map(|event| event.kind()) else {
            continue;
        };

        match event {
            TileEventKind::Fire => {
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
            TileEventKind::BlackDust => {
                let rainfall = planet.map[p].rainfall;
                let TileEvent::BlackDust {
                    ref mut remaining_cycles,
                } = &mut **planet.map[p].event.as_mut().unwrap()
                else {
                    unreachable!()
                };
                let remaining_cycles_decrease =
                    (rainfall / params.event.black_dust_decrease_by_rainfall) as u32 + 1;
                if *remaining_cycles < remaining_cycles_decrease {
                    planet.map[p].event = None;
                } else {
                    *remaining_cycles -= remaining_cycles_decrease;
                }
            }
            TileEventKind::AerosolInjection => {
                let TileEvent::AerosolInjection {
                    ref mut remaining_cycles,
                } = &mut **planet.map[p].event.as_mut().unwrap()
                else {
                    unreachable!()
                };
                *remaining_cycles -= 1;
                if *remaining_cycles == 0 {
                    planet.map[p].event = None;
                }
                planet.atmo.aerosol += params.event.aerosol_injection_amount;
            }
            TileEventKind::Plague => todo!(),
        }
    }
}

pub fn cause_tile_event(
    planet: &mut Planet,
    p: Coords,
    kind: TileEventKind,
    _sim: &mut Sim,
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
        TileEventKind::Plague => todo!(),
    };

    planet.map[p].event = Some(Box::new(event));
}
