use bevy::prelude::*;

use crate::conf::Conf;
use crate::draw::UpdateMap;
use crate::screen::{Centering, HoverTile};
use crate::ui::WindowsOpenState;
use crate::{planet::*, GameSpeed, GameState, GameSystemSet};

#[derive(Clone, Copy, Debug)]
pub struct SimPlugin;

#[derive(Clone, Debug, Event)]
pub enum ManagePlanet {
    New(StartParams),
    Save(usize),
    Load(usize),
}

#[derive(Clone, Debug, Event)]
pub struct StartEvent(pub PlanetEvent);

#[derive(Clone, Default, Debug, Resource)]
pub struct DebugTools {
    pub sim_every_frame: bool,
}

#[derive(Clone, Copy, Default, Debug, Resource)]
pub struct SaveSlot(pub Option<usize>);

impl Resource for Planet {}
impl Resource for Params {}
impl Resource for Sim {}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ManagePlanet>()
            .add_event::<ManagePlanetError>()
            .add_event::<StartEvent>()
            .init_resource::<DebugTools>()
            .init_resource::<SaveSlot>()
            .add_systems(
                OnEnter(GameState::Running),
                start_sim.in_set(GameSystemSet::StartSim),
            )
            .add_systems(
                FixedUpdate,
                (update, start_event).run_if(in_state(GameState::Running)),
            )
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
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: Res<Params>,
    speed: Res<GameSpeed>,
    debug_tools: Res<DebugTools>,
    hover_tile: Query<&HoverTile>,
    wos: Res<WindowsOpenState>,
    conf: Res<Conf>,
    mut count_frame: Local<u64>,
    mut last_update: Local<Option<u64>>,
) {
    if wos.save || wos.load {
        return;
    }

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
    crate::planet::debug::clear_logs(
        hover_tile
            .get_single()
            .ok()
            .and_then(|hover_tile| hover_tile.0),
    );
    update_map.update();
    planet.advance(&mut sim, &params);

    if let Some(autosave_cycle_duration) = conf.autosave_cycle_duration {
        if planet.cycles % autosave_cycle_duration == 0 {
            ew_manage_planet.send(ManagePlanet::Save(0));
        }
    }
}

fn start_event(
    mut planet: ResMut<Planet>,
    mut sim: ResMut<Sim>,
    mut er_start_event: EventReader<StartEvent>,
    params: Res<Params>,
) {
    for event in er_start_event.read() {
        planet.start_event(event.0.clone(), &mut sim, &params);
    }
}

#[derive(Clone, Debug, Event)]
pub enum ManagePlanetError {
    Decode,
    Other,
}

fn manage_planet(
    mut command: Commands,
    mut er_manage_planet: EventReader<ManagePlanet>,
    mut ew_manage_planet_error: EventWriter<ManagePlanetError>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut ew_centering: EventWriter<Centering>,
    mut planet: Option<ResMut<Planet>>,
    mut save_slot: ResMut<SaveSlot>,
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
            *save_slot = SaveSlot(None);
            Some(planet)
        }
        ManagePlanet::Save(slot) => {
            if let Err(e) = crate::saveload::save_to(*slot, planet.as_ref().unwrap()) {
                log::warn!("cannot save: {:?}", e);
            }
            None
        }
        ManagePlanet::Load(slot) => match crate::saveload::load_from(*slot) {
            Ok(planet) => {
                if *slot != 0 {
                    *save_slot = SaveSlot(Some(*slot));
                }
                Some(planet)
            }
            Err(e) => {
                log::warn!("cannot load: {:?}", e);
                let e = if e.is::<rmp_serde::decode::Error>() {
                    ManagePlanetError::Decode
                } else {
                    ManagePlanetError::Other
                };
                ew_manage_planet_error.send(e);
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
