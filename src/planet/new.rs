use sim::CoordsConverter;

use super::*;

impl Planet {
    pub fn new(start_params: &StartParams, params: &Params) -> Planet {
        let mut map = Array2d::new(start_params.size.0, start_params.size.1, Tile::default());

        let gen_conf = map_generator::GenConf {
            w: start_params.size.0,
            h: start_params.size.1,
            max_height: start_params.difference_in_elevation,
            height_table: start_params.height_table.clone(),
            height_map: start_params.height_map.clone(),
        };
        let height_map = map_generator::generate(gen_conf);
        for (p, height) in height_map.iter_with_idx() {
            map[p].height = *height;
        }

        let mut planet = Planet {
            cycles: 0,
            basics: start_params.basics.clone(),
            state: State::default(),
            res: Resources::new(start_params),
            map,
            atmo: Atmosphere::new(start_params, params),
            water: Water::new(start_params),
            space_buildings: SpaceBuildingKind::iter()
                .map(|kind| (kind, Building::default()))
                .collect(),
            events: Events::default(),
            civs: Civs::default(),
            stat: Stat::new(params),
            reports: Reports::default(),
        };

        for (&kind, &n) in &start_params.space_buildings {
            let building = planet.space_building_mut(kind);
            building.n = n;
        }
        // Locate initial buried carbon
        if let Some(initial_buried_carbon) = &start_params.initial_buried_carbon {
            locate_initial_buried_carbon(&mut planet, initial_buried_carbon);
        }

        // Adjust water volume
        if start_params.target_sea_level.is_some() || start_params.target_sea_area.is_some() {
            let sim = Sim::new(&planet, params);
            let target_sea_level = start_params
                .target_sea_level
                .map(|target_sea_level| target_sea_level * start_params.difference_in_elevation);
            let target_diff = if target_sea_level.is_some() {
                50.0
            } else {
                0.02
            };
            let max_water_volume = planet.water.water_volume;
            let water_volume = misc::bisection(
                |water_volume| {
                    planet.water.water_volume = water_volume;
                    super::water::update_sea_level(&mut planet, &sim, params);
                    if let Some(target_sea_level) = target_sea_level {
                        planet.water.sea_level - target_sea_level
                    } else {
                        let n_sea_tile =
                            planet.map.iter().filter(|tile| tile.biome.is_sea()).count();
                        let size = planet.map.size();
                        let ratio = n_sea_tile as f32 / (size.0 * size.1) as f32;
                        ratio - start_params.target_sea_area.unwrap()
                    }
                },
                0.0,
                max_water_volume * 10.0,
                20,
                target_diff,
            );
            planet.water.water_volume = water_volume;
        }

        // Simulate before start
        let mut sim = Sim::new(&planet, params);
        planet.advance(&mut sim, params);
        for initial_condition in &start_params.initial_conditions {
            initial_conditions::apply_initial_condition(
                &mut planet,
                &mut sim,
                initial_condition.clone(),
                params,
            );
        }
        planet.advance(&mut sim, params);
        heat_transfer::init_temp(&mut planet, &mut sim, params);

        let water_volume = planet.water.water_volume;
        planet.water.water_volume = 0.0;
        planet.water.water_volume = water_volume;
        planet.advance(&mut sim, params);

        sim.before_start = true;
        for _ in 0..start_params.cycles_before_start {
            if start_params
                .initial_conditions
                .iter()
                .any(|ic| matches!(ic, InitialCondition::Snowball { .. }))
            {
                sim.albedo.fill(0.8);
            }
            planet.advance(&mut sim, params);
        }

        // Reset
        planet.cycles = 0;
        planet.stat.clear_history();
        planet.res.material = 0.0;
        self::stat::record_stats(&mut planet, params);

        planet
    }
}

fn locate_initial_buried_carbon(planet: &mut Planet, initial_buried_carbon: &InitialBuriedCarbon) {
    let coords_converter = CoordsConverter::new(planet);
    let size = planet.map.size();
    let InitialBuriedCarbon {
        n_spot,
        mass,
        radius,
        scattering,
    } = *initial_buried_carbon;
    let mut rng = super::misc::get_rng();
    let n_spot = rng.random_range(n_spot.0..n_spot.1);

    for _ in 0..n_spot {
        let mass = 10.0_f32.powf(rng.random_range(mass.0.log10()..mass.1.log10()));
        let radius = rng.random_range(radius.0..=radius.1);
        let center = (rng.random_range(0..size.0), rng.random_range(0..size.1));
        let shape = geom::Shape::Circle {
            center: center.into(),
            radius,
        };
        let tiles: Vec<_> = shape
            .iter()
            .into_iter()
            .filter_map(|p| coords_converter.conv(p))
            .collect();
        let m = mass / tiles.len() as f32;
        for p in tiles {
            planet.map[p].buried_carbon =
                m * rng.random_range((1.0 - scattering)..(1.0 + scattering));
        }
    }
}
