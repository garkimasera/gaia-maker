use bevy::prelude::*;

use crate::draw::UpdateMap;
use crate::screen::{Centering, HoverTile};
use crate::{planet::*, GameSpeed, GameState, GameSystemSet};

#[derive(Clone, Copy, Debug)]
pub struct SimPlugin;

#[derive(Clone, Debug, Event)]
pub enum ManagePlanet {
    New(StartParams),
    Save(String),
    Load(String),
}

#[derive(Clone, Default, Debug, Resource)]
pub struct DebugTools {
    pub sim_every_frame: bool,
}

impl Resource for Planet {}
impl Resource for Params {}
impl Resource for Sim {}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ManagePlanet>()
            .add_event::<ManagePlanetError>()
            .init_resource::<DebugTools>()
            .add_systems(
                OnEnter(GameState::Running),
                start_sim.in_set(GameSystemSet::StartSim),
            )
            .add_systems(Update, update.run_if(in_state(GameState::Running)))
            .add_systems(Update, manage_planet.before(GameSystemSet::Draw));
    }
}

fn start_sim(mut update_map: ResMut<UpdateMap>) {
    update_map.update();
}

fn update(
    mut planet: ResMut<Planet>,
    mut update_map: ResMut<UpdateMap>,
    mut sim: ResMut<Sim>,
    params: Res<Params>,
    speed: Res<GameSpeed>,
    debug_tools: Res<DebugTools>,
    hover_tile: Query<&HoverTile>,
    mut count_frame: Local<u64>,
    mut last_update: Local<Option<u64>>,
) {
    *count_frame += 1;

    match (*speed, debug_tools.sim_every_frame) {
        (GameSpeed::Paused, _) => {
            return;
        }
        (_, true) => (),
        (GameSpeed::Normal, _) => {
            if last_update.is_some()
                && *count_frame - last_update.unwrap()
                    < 60 * params.sim.sim_normal_loop_duration_ms / 1000
            {
                return;
            }
        }
        (GameSpeed::Fast, _) => {
            if last_update.is_some()
                && *count_frame - last_update.unwrap()
                    < 60 * params.sim.sim_fast_loop_duration_ms / 1000
            {
                return;
            }
        }
    }
    *last_update = Some(*count_frame);
    crate::planet::debug_log::clear_logs(
        hover_tile
            .get_single()
            .ok()
            .and_then(|hover_tile| hover_tile.0),
    );
    update_map.update();
    planet.advance(&mut sim, &params);
}

#[derive(Debug, Event)]
pub struct ManagePlanetError(pub String);

fn manage_planet(
    mut command: Commands,
    mut er_manage_planet: EventReader<ManagePlanet>,
    mut ew_manage_planet_eror: EventWriter<ManagePlanetError>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut ew_centering: EventWriter<Centering>,
    mut planet: Option<ResMut<Planet>>,
    params: Option<Res<Params>>,
) {
    let Some(params) = params else {
        return;
    };

    let Some(e) = er_manage_planet.read().next() else {
        return;
    };
    let new_planet = match e {
        ManagePlanet::New(start_params) => {
            let planet = Planet::new(start_params, &params);
            Some(planet)
        }
        ManagePlanet::Save(path) => {
            if let Err(e) = crate::saveload::save_to(path, planet.as_ref().unwrap()) {
                log::warn!("cannot save: {:?}", e);
            }
            None
        }
        ManagePlanet::Load(path) => match crate::saveload::load_from(path) {
            Ok(planet) => Some(planet),
            Err(e) => {
                ew_manage_planet_eror.send(ManagePlanetError(e.to_string()));
                log::warn!("cannot load: {:?}", e);
                None
            }
        },
    };

    if let Some(new_planet) = new_planet {
        ew_centering.send(Centering(Vec2::new(
            new_planet.map.size().0 as f32 * TILE_SIZE / 2.0,
            new_planet.map.size().1 as f32 * TILE_SIZE / 2.0,
        )));

        let sim = Sim::new(&new_planet);
        command.insert_resource(sim);
        if let Some(planet) = &mut planet {
            **planet = new_planet;
        } else {
            command.insert_resource(new_planet);
        }

        next_game_state.set(GameState::Running);
    }
}
