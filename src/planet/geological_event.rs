use super::*;

pub fn advance_geological_event(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for p in planet.map.iter_idx() {
        if let Some(TileEvent::VolcanicEruption {
            remaining_cycles,
            power,
        }) = planet.map[p]
            .tile_events
            .get_mut(TileEventKind::VolcanicEruption)
        {
            *remaining_cycles -= 1;

            if *remaining_cycles == 0 {
                planet.map[p]
                    .tile_events
                    .remove(TileEventKind::VolcanicEruption);
            } else {
                let power = *power;
                process_each_volcanic_eruption_event(planet, sim, params, p, power);
            }
        }
    }
}

fn process_each_volcanic_eruption_event(
    planet: &mut Planet,
    sim: &mut Sim,
    params: &Params,
    p_center: Coords,
    power: f32,
) {
    for &d in [Coords(0, 0)].iter().chain(geom::CHEBYSHEV_DISTANCE_1_COORDS) {
        let Some(p) = sim.convert_p_cyclic(p_center + d) else {
            continue;
        };

        let height_above_sea_level = planet.height_above_sea_level(p);

        let tile = &mut planet.map[p];
        if matches!(tile.structure, Some(Structure::Settlement(_))) {
            tile.structure = None;
        }
        tile.animal = [None; AnimalSize::LEN];
        if tile.biome.is_land() && tile.biome != Biome::Desert {
            tile.biome = Biome::Rock;
        }

        let biomass = tile.biomass;
        let burned_biomass = biomass * params.event.volcanic_eruption_burn_ratio;
        let biomass = biomass - burned_biomass;
        tile.biomass = biomass;
        let burned_biomass = sim.biomass_density_to_mass();
        planet.atmo.release_carbon(burned_biomass);

        let uplift_range = params.event.volcanic_eruption_uplift;
        let uplift = sim.rng.random_range(uplift_range.0..uplift_range.1) * power;
        let uplift = if p == p_center { 2.0 * uplift } else { uplift };
        let uplift = if height_above_sea_level < 10000.0 {
            uplift
        } else {
            0.01 * uplift
        };
        tile.height += uplift;
    }

    planet.atmo.aerosol += params.event.volcanic_eruption_aerosol * power;
    planet.atmo.add(
        GasKind::CarbonDioxide,
        params.event.volcanic_eruption_carbon_dioxide * power,
    );
}
