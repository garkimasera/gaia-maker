use super::*;

impl Planet {
    pub fn new(start_params: &StartParams, params: &Params) -> Planet {
        let mut map = Array2d::new(start_params.size.0, start_params.size.1, Tile::default());

        let gen_conf = map_generator::GenConf {
            w: start_params.size.0,
            h: start_params.size.1,
            max_height: start_params.difference_in_elevation,
            height_table: start_params.height_table.clone(),
        };
        let height_map = map_generator::generate(gen_conf);
        for (p, height) in height_map.iter_with_idx() {
            map[p].height = *height;
        }

        let mut msgs = MsgHolder::default();
        msgs.append(0, MsgKind::EventStart, 10000);

        let mut planet = Planet {
            cycles: 0,
            basics: start_params.basics.clone(),
            state: State::default(),
            player: Player::default(),
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
            msgs,
        };

        for (&kind, &n) in &start_params.space_buildings {
            let building = planet.space_building_mut(kind);
            building.n = n;
            if params.building_attrs(kind).control == BuildingControl::EnabledNumber {
                building.control = BuildingControlValue::EnabledNumber(n);
            }
        }

        for structure_kind in StructureKind::iter() {
            if structure_kind.buildable_by_player() {
                planet.player.buildable_structures.insert(structure_kind);
            }
        }

        // Adjust water volume
        if let Some(target_sea_level) = start_params.target_sea_level {
            let mut sim = Sim::new(&planet);
            let target_sea_level = target_sea_level * start_params.difference_in_elevation;
            let max_water_volume = planet.water.water_volume;
            let water_volume = misc::bisection(
                |water_volume| {
                    planet.water.water_volume = water_volume;
                    sim_water(&mut planet, &mut sim, params);
                    planet.water.sea_level - target_sea_level
                },
                0.0,
                max_water_volume * 10.0,
                30,
                100.0,
            );
            planet.water.water_volume = water_volume;
        }

        // Simulate before start
        let mut sim = Sim::new(&planet);
        sim.before_start = true;
        planet.advance(&mut sim, params);
        heat_transfer::init_temp(&mut planet, &mut sim, params);

        let water_volume = planet.water.water_volume;
        planet.water.water_volume = 0.0;
        for _ in 0..(start_params.cycles_before_start / 2) {
            // Advance without water to accelerate heat transfer calclation
            planet.advance(&mut sim, params);
        }
        planet.water.water_volume = water_volume;
        planet.advance(&mut sim, params);

        for initial_condition in &start_params.initial_conditions {
            initial_conditions::apply_initial_condition(
                &mut planet,
                &mut sim,
                initial_condition.clone(),
                params,
            );
        }

        for _ in 0..(start_params.cycles_before_start / 2) {
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
