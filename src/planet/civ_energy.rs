use super::misc::linear_interpolation;
use super::*;

pub fn sim_energy_source(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    sim.civ_sum.reset(planet.civs.keys().copied());
    update_civ_domain(planet, sim);

    // Update sparse energy source
    sim.energy_wind_solar = linear_interpolation(
        &params.sim.table_solar_constant_wind_solar,
        planet.basics.solar_constant,
    ) * sim.tile_area;

    let geothermal_per_tile = planet.basics.geothermal_power
        * params.sim.available_geothermal_ratio
        * (3600.0 * 24.0)
        * 1.0e-9
        / sim.n_tiles as f32;

    for p in planet.map.iter_idx() {
        let geothermal =
            if planet.height_above_sea_level(p) > params.sim.max_depth_undersea_resource {
                geothermal_per_tile
            } else {
                0.0
            };
        sim.energy_hydro_geothermal[p] =
            linear_interpolation(&params.sim.table_rainfall_hydro, planet.map[p].rainfall)
                * sim.tile_area
                + geothermal;
    }

    // Fossil fuel & gift energy
    for p in planet.map.iter_idx() {
        let Some((id, _)) = sim.domain[p] else {
            continue;
        };
        let sum_values = sim.civ_sum.get_mut(id);

        let available = planet.map[p].buried_carbon - params.sim.buried_carbon_energy_threshold;
        if available > 0.0 {
            let src_tiles = &mut sum_values.fossil_fuel_src_tiles;
            src_tiles.insert(
                ordered_float::NotNan::new(available).expect("invalid buried carbon value"),
                p,
            );
            if src_tiles.len() > params.sim.n_tiles_fossil_fuel_mine {
                src_tiles.pop_first();
            }
        }

        if matches!(planet.map[p].structure, Some(Structure::GiftTower)) {
            let attrs = params.building_attrs(StructureKind::GiftTower);
            if let Some(BuildingEffect::SupplyEnergy { value }) = attrs.effect {
                sum_values.gift_supply += value;
            }
        }
    }

    for (_, sum_values) in sim.civ_sum.iter_mut() {
        let available_fossil_fuel_mass = sum_values
            .fossil_fuel_src_tiles
            .keys()
            .map(|mass| mass.into_inner())
            .sum::<f32>();
        sum_values.fossil_fuel_supply = available_fossil_fuel_mass
            * params.sim.fossil_fuel_combustion_energy
            * params.sim.available_fossil_fuel_ratio;
    }
}

pub fn update_civ_domain(planet: &Planet, sim: &mut Sim) {
    for p in planet.map.iter_idx() {
        sim.domain[p] = None;
    }

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &planet.map[p].structure else {
            continue;
        };
        sim.domain[p] = Some((settlement.id, f32::INFINITY));
        sim.civ_sum
            .get_mut(settlement.id)
            .total_pop_for_energy_distribution += settlement.pop as f64;

        for d in geom::CHEBYSHEV_DISTANCE_1_COORDS {
            if let Some(p_adj) = sim.convert_p_cyclic(p + *d) {
                if let Some((_, weight)) = sim.domain[p_adj] {
                    if settlement.pop > weight {
                        sim.domain[p_adj] = Some((settlement.id, settlement.pop));
                    }
                } else {
                    sim.domain[p_adj] = Some((settlement.id, settlement.pop));
                }
            }
        }
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

    // Calculate fossil fuel & gift energy supply
    let sum_values = sim.civ_sum.get_mut(animal_id);
    let a = settlement.pop / sum_values.total_pop_for_energy_distribution as f32;
    supply[EnergySource::FossilFuel as usize] = sum_values.fossil_fuel_supply * a;
    supply[EnergySource::Gift as usize] = sum_values.gift_supply * a;

    // Calculate nuclear energy supply
    let a = match settlement.age {
        CivilizationAge::Atomic => {
            (params.sim.base_nuclear_ratio + settlement.tech_exp).clamp(0.0, 1.0)
        }
        CivilizationAge::EarlySpace => 1.0,
        _ => 0.0,
    };
    supply[EnergySource::Nuclear as usize] = demand * a;

    // Calculate energy distribution
    let priority = [
        EnergySource::Gift,
        EnergySource::HydroGeothermal,
        EnergySource::Nuclear,
        EnergySource::FossilFuel,
        EnergySource::WindSolar,
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
    if x < 1.0 { x * x } else { x.min(1.0) }
}

pub fn consume_buried_carbon(planet: &mut Planet, sim: &mut Sim, params: &Params) {
    for (_, sum_values) in sim.civ_sum.iter() {
        let mass = sum_values.total_energy_consumption[EnergySource::FossilFuel as usize] as f32
            / params.sim.fossil_fuel_combustion_energy;
        let available_fossil_fuel_mass = sum_values
            .fossil_fuel_src_tiles
            .keys()
            .map(|mass| mass.into_inner())
            .sum::<f32>();

        for (m, p) in &sum_values.fossil_fuel_src_tiles {
            let consume_mass = mass * (m.into_inner() / available_fossil_fuel_mass);
            planet.map[*p].buried_carbon = (planet.map[*p].buried_carbon - consume_mass).max(0.0);
        }

        planet.atmo.release_carbon(mass);
    }
}
