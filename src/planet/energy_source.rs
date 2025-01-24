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
