use arrayvec::ArrayVec;
use rayon::prelude::*;

use super::misc::linear_interpolation;
use super::*;

/// Update some prerequired calculation results in Sim for civilization simulation
pub fn update_civ_energy(planet: &Planet, sim: &mut Sim, params: &Params) {
    sim.civ_sum.reset(planet.civs.keys().copied());
    update_civ_domain(planet, sim);

    // Update settlement congestion rate
    let size = planet.map.size();
    let par_iter = sim.settlement_cr.par_iter_mut().enumerate();
    par_iter.for_each(|(i, settlement_cr)| {
        let p = Coords::from_index_size(i, size);
        *settlement_cr = super::misc::calc_congestion_rate(p, planet.map.size(), |p| {
            if matches!(planet.map[p].structure, Some(Structure::Settlement { .. })) {
                1.0
            } else {
                0.0
            }
        });
    });

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
    sim.domain.fill(None);

    for p in planet.map.iter_idx() {
        let Some(Structure::Settlement(settlement)) = &planet.map[p].structure else {
            continue;
        };
        sim.domain[p] = Some((settlement.id, f32::INFINITY));
        sim.civ_sum.get_mut(settlement.id).total_pop_prev += settlement.pop as f64;

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
    settlement: &mut Settlement,
    params: &Params,
    cr: f32,
) {
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
    supply[EnergySource::SolarWind as usize] =
        surrounding_wind_solar * (1.0 - cr) + sim.energy_wind_solar;
    supply[EnergySource::HydroGeothermal as usize] =
        surrounding_hydro_geothermal * (1.0 - cr) + sim.energy_hydro_geothermal[p];

    // Calculate fossil fuel & gift energy supply
    let sum_values = sim.civ_sum.get_mut(animal_id);
    let a = settlement.pop / sum_values.total_pop_prev as f32;
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
    let src_without_biomass = [
        EnergySource::Gift,
        EnergySource::HydroGeothermal,
        EnergySource::Nuclear,
        EnergySource::FossilFuel,
        EnergySource::SolarWind,
    ];
    let mut v: ArrayVec<(usize, f32, f32), { (EnergySource::LEN - 1) * 2 }> = ArrayVec::new();
    let mut high_eff_wind_solar = 0.0;
    for src in src_without_biomass {
        let src = src as usize;
        debug_assert!(supply[src] >= 0.0);
        let eff = params.sim.energy_efficiency[age][src];
        let high_eff = params.sim.energy_high_efficiency[age][src];
        if high_eff > 0.0 {
            let high_eff_supply = (supply[src] * params.sim.high_efficiency_limit_by_supply[src])
                .min(demand * params.sim.high_efficiency_limit_by_demand[src]);
            let normal_supply = supply[src] - high_eff_supply;
            v.push((src, high_eff, high_eff_supply));
            v.push((src, eff, normal_supply));
            if src == EnergySource::SolarWind as usize {
                high_eff_wind_solar = high_eff_supply;
            }
        } else {
            v.push((src, eff, supply[src]));
        }
    }
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut remaining = demand;
    let mut sum_eff = 0.0;
    for (src, eff, supply) in v {
        let a = (demand * params.sim.energy_source_limit_by_age[age][src] - consume[src])
            .min(supply)
            .min(remaining);
        debug_assert!(a >= 0.0);
        consume[src] += a;
        remaining -= a;
        if eff > 0.0 {
            sum_eff += a / eff;
        }
    }
    consume[EnergySource::Biomass as usize] = remaining;
    let biomass_eff_factor = linear_interpolation(
        &params.sim.biomass_energy_efficiency_density_factor_table,
        planet.map[p].biomass,
    );
    sum_eff += remaining
        / (params.sim.energy_efficiency[age][EnergySource::Biomass as usize] * biomass_eff_factor);
    sim.energy_eff[p] = demand / sum_eff;

    // Add waste energy consume
    for src in EnergySource::iter() {
        let src = src as usize;
        let req = demand * params.sim.energy_source_waste_by_age[age][src];
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

    // Calculate biomass consumption
    let impact_on_biomass: f32 = params
        .sim
        .energy_source_biomass_impact
        .iter()
        .enumerate()
        .map(|(src, a)| {
            if src == EnergySource::SolarWind as usize {
                params.sim.high_efficiency_wind_solar_biomass_impact
                    * high_eff_wind_solar.max(consume[src])
                    + a * (consume[src] - high_eff_wind_solar).max(0.0)
            } else {
                a * consume[src]
            }
        })
        .sum();
    debug_assert!(impact_on_biomass > 0.0);
    settlement.biomass_consumption = impact_on_biomass / params.sim.biomass_energy_factor;
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
