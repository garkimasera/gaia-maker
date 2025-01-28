use super::misc::linear_interpolation;
use super::*;

pub fn sim_energy_source(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    sim.energy_wind_solar = linear_interpolation(
        &params.sim.table_solar_constant_wind_solar,
        planet.basics.solar_constant,
    ) * sim.tile_area;

    let geothermal_per_tile =
        planet.basics.geothermal_power * 3600.0 * 24.0 * 1.0e-6 / sim.n_tiles as f32;

    for p in planet.map.iter_idx() {
        sim.energy_hydro_geothermal[p] =
            linear_interpolation(&params.sim.table_rainfall_hydro, planet.map[p].rainfall)
                * sim.tile_area
                + geothermal_per_tile;
    }
}

pub fn process_settlement_energy(
    planet: &mut Planet,
    sim: &mut Sim,
    p: Coords,
    settlement: &Settlement,
    params: &Params,
    cr: f32,
) -> f32 {
    let age = settlement.age as usize;
    let animal_id = settlement.id;

    let demand = settlement.pop * params.sim.energy_demand_per_pop[age];
    let mut supply = [0.0; EnergySource::LEN];
    let mut consume = [0.0; EnergySource::LEN];

    // Calculate sparse energy supply
    let mut surrounding_wind_solar = 0.0;
    let mut surrounding_hydro_geothermal = 0.0;

    for p_adj in geom::CHEBYSHEV_DISTANCE_1_COORDS {
        if let Some(p_adj) = sim.convert_p_cyclic(p + *p_adj) {
            if !matches!(planet.map[p_adj].structure, Some(Structure::Settlement(_))) {
                surrounding_wind_solar += sim.energy_wind_solar;
                surrounding_hydro_geothermal += sim.energy_hydro_geothermal[p_adj];
            }
        }
    }
    supply[EnergySource::WindSolar as usize] =
        surrounding_wind_solar * (1.0 - cr) + sim.energy_wind_solar;
    supply[EnergySource::HydroGeothermal as usize] =
        surrounding_hydro_geothermal * (1.0 - cr) + sim.energy_hydro_geothermal[p];

    // Calculate energy distribution
    let priority = [
        EnergySource::Gift,
        EnergySource::HydroGeothermal,
        EnergySource::Nuclear,
        EnergySource::WindSolar,
        EnergySource::FossilFuel,
    ];
    let mut remaining = demand;
    for src in priority {
        let src = src as usize;
        debug_assert!(supply[src] >= 0.0);
        consume[src] += (demand * params.sim.energy_source_limit_by_age[age][src])
            .min(supply[src])
            .min(remaining);
        remaining -= consume[src];
    }
    consume[EnergySource::Biomass as usize] = remaining;

    // Add minimum required or waste energy consume
    for src in EnergySource::iter() {
        let src = src as usize;
        let req = demand * params.sim.energy_source_min_by_age[age][src];
        let supply = supply[src] - consume[src];
        if src == 0 || supply > req {
            consume[src] += req;
        } else {
            consume[src] += supply.max(0.0);
        }
    }

    // Record
    let sum_values = sim.civ_sum.get_mut(animal_id);
    for src in EnergySource::iter() {
        sum_values.total_energy_consumption[src as usize] += consume[src as usize] as f64;
    }

    // Consume biomass from a tile that has maximum biomass
    let impact_on_biomass: f32 = params
        .sim
        .energy_source_biomass_impact
        .iter()
        .enumerate()
        .map(|(src, a)| a * consume[src])
        .sum();
    if impact_on_biomass <= 0.0 {
        return 1.0;
    }
    let biomass_to_consume = impact_on_biomass / params.sim.biomass_energy_factor;
    let mut p_max_biomass = p;
    let mut total_biomass = planet.map[p].biomass;
    let mut max_biomass = total_biomass;
    for p_adj in geom::CHEBYSHEV_DISTANCE_1_COORDS {
        if let Some(p_adj) = sim.convert_p_cyclic(p + *p_adj) {
            if !matches!(planet.map[p_adj].structure, Some(Structure::Settlement(_))) {
                let biomass = planet.map[p_adj].biomass;
                if biomass > max_biomass {
                    max_biomass = biomass;
                    total_biomass += biomass;
                    p_max_biomass = p_adj;
                }
            }
        }
    }

    // Decrease biomass
    let total_biomass = total_biomass * sim.biomass_density_to_mass();
    let max_biomass = max_biomass * sim.biomass_density_to_mass();
    let available_biomass_ratio = if biomass_to_consume > 0.0 {
        total_biomass / biomass_to_consume
    } else {
        return 1.0;
    };

    let new_biomass = (max_biomass - biomass_to_consume).max(0.0);
    let diff_biomass = max_biomass - new_biomass;
    planet.map[p_max_biomass].biomass = new_biomass / sim.biomass_density_to_mass();
    planet.atmo.release_carbon(diff_biomass);

    let x = available_biomass_ratio * params.sim.resource_availability_factor;
    if x < 1.0 {
        x * x
    } else {
        x.min(1.0)
    }
}
