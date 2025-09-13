use super::*;

pub fn advance_geological_event(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    let planet_relative_geo_power =
        planet.basics.geothermal_power / params.default_start_params.basics.geothermal_power;
    let prob = ((planet_relative_geo_power.log10() as f64 + 1.0)
        * params.event.volcanic_eruption_prob)
        .clamp(0.0, 1.0);
    if sim.rng.random_bool(prob) {
        let (w, h) = planet.map.size();
        let p = Coords::new(
            sim.rng.random_range(0..w) as i32,
            sim.rng.random_range(0..h) as i32,
        );
        let event = volcanic_eruption_tile_event(planet, sim, params, false);
        planet.map[p].tile_events.insert(event);
    }

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
        let burn_ratio = (params.event.volcanic_eruption_burn_ratio * power).clamp(0.0, 1.0);
        let burned_biomass = biomass * burn_ratio;
        let biomass = biomass - burned_biomass;
        tile.biomass = biomass;
        let burned_biomass = burned_biomass * sim.biomass_density_to_mass();
        planet
            .atmo
            .release_carbon(burned_biomass * params.event.volcanic_eruption_carbon_release_ratio);
        tile.buried_carbon +=
            burned_biomass * (1.0 - params.event.volcanic_eruption_carbon_release_ratio);

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
        params.event.volcanic_eruption_carbon_dioxide * power * power,
    );
}

pub fn volcanic_eruption_tile_event(
    planet: &Planet,
    sim: &mut Sim,
    params: &Params,
    artificial: bool,
) -> TileEvent {
    let planet_relative_geo_power =
        planet.basics.geothermal_power / params.default_start_params.basics.geothermal_power;
    let power_range = if artificial {
        params.event.artificial_volcanic_eruption_power
    } else {
        params.event.volcanic_eruption_power
    };
    let power = sim.rng.random_range(power_range.0..power_range.1) * planet_relative_geo_power;
    let cycles_range = params.event.volcanic_eruption_cycles;
    let remaining_cycles = sim.rng.random_range(cycles_range.0..cycles_range.1);

    TileEvent::VolcanicEruption {
        remaining_cycles,
        power,
    }
}
