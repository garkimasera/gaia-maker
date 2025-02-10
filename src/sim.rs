use anyhow::Context;
use bevy::prelude::*;

use crate::conf::Conf;
use crate::draw::UpdateMap;
use crate::saveload::SavedTime;
use crate::screen::{Centering, HoverTile};
use crate::ui::WindowsOpenState;
use crate::{planet::*, GameSpeed, GameState, GameSystemSet};

pub use crate::saveload::SaveState;

#[derive(Clone, Copy, Debug)]
pub struct SimPlugin;

const GLOBAL_DATA_FILE_NAME: &str = "gaia-maker_global";

#[derive(Clone, Default, Debug, Resource, serde::Serialize, serde::Deserialize)]
pub struct GlobalData {}

#[derive(Clone, Debug, Event)]
pub enum ManagePlanet {
    New(StartParams),
    Save {
        auto: bool,
        _new_name: Option<String>,
    },
    Load {
        sub_dir_name: String,
        auto: bool,
        n: u32,
    },
    Delete {
        sub_dir_name: String,
        all: bool,
        auto: bool,
        n: u32,
    },
}

#[derive(Clone, Debug, Event)]
pub struct StartEvent(pub PlanetEvent);

impl Resource for SaveState {}
impl Resource for Planet {}
impl Resource for Params {}
impl Resource for Sim {}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ManagePlanet>()
            .add_event::<ManagePlanetError>()
            .add_event::<StartEvent>()
            .init_resource::<SaveState>()
            .add_systems(
                OnExit(GameState::AssetLoading),
                (load_global_data, set_save_state),
            )
            .add_systems(OnEnter(GameState::MainMenu), reset_save_state)
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

fn load_global_data(mut command: Commands) {
    let global_data = match crate::platform::read_data_file(GLOBAL_DATA_FILE_NAME)
        .and_then(|data| toml::from_str(&data).context("deserialize global data"))
    {
        Ok(global_data) => global_data,
        Err(e) => {
            log::warn!("cannot load global data: {:?}", e);
            let global_data = GlobalData::default();
            let s = toml::to_string(&global_data).unwrap();
            if let Err(e) = crate::platform::write_data_file(GLOBAL_DATA_FILE_NAME, &s) {
                log::error!("cannot write global data: {:?}", e);
            }
            global_data
        }
    };
    command.insert_resource(global_data);
}

fn set_save_state(mut save_state: ResMut<SaveState>) {
    let mut save_sub_dirs = match crate::platform::save_sub_dirs() {
        Ok(save_sub_dirs) => save_sub_dirs,
        Err(e) => {
            log::warn!("{}", e);
            Vec::new()
        }
    };
    save_sub_dirs.sort_by_key(|(time, _)| std::cmp::Reverse(time.clone()));
    save_state.dirs = save_sub_dirs.into();
}

fn reset_save_state(mut save_state: ResMut<SaveState>) {
    save_state.change_current("", false);
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

    match (*speed, conf.max_simulation_speed) {
        (GameSpeed::Paused, _) => {
            return;
        }
        (GameSpeed::Normal, _) => {
            if last_update.is_some()
                && *count_frame - last_update.unwrap()
                    < 60 * params.sim.sim_normal_loop_duration_ms / 1000
            {
                return;
            }
        }
        (_, true) => (),
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

    if conf.autosave_enabled && planet.cycles % conf.autosave_cycle_duration == 0 {
        ew_manage_planet.send(ManagePlanet::Save {
            auto: true,
            _new_name: None,
        });
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
    mut save_state: ResMut<SaveState>,
    res_maybe_initialized: (Option<Res<Params>>, Option<Res<Conf>>),
) {
    let (Some(params), Some(conf)) = res_maybe_initialized else {
        return;
    };

    let Some(e) = er_manage_planet.read().next() else {
        return;
    };
    let new_planet = match e {
        ManagePlanet::New(start_params) => {
            let planet = Planet::new(start_params, &params);
            let sub_dir_name = sanitize_filename::sanitize_with_options(
                &start_params.basics.name,
                sanitize_filename::Options {
                    truncate: true,
                    windows: true,
                    replacement: " ",
                },
            );
            let sub_dir_name = crate::saveload::check_save_dir_name_dup(&save_state, sub_dir_name);
            save_state
                .dirs
                .push_front((SavedTime::now(), sub_dir_name.clone()));
            save_state.change_current(&sub_dir_name, true);

            if let Err(e) = crate::saveload::save_to(&planet, &mut save_state, true) {
                log::warn!("cannot save: {:?}", e);
            }
            Some(planet)
        }
        ManagePlanet::Save { auto, .. } => {
            if let Err(e) =
                crate::saveload::save_to(planet.as_ref().unwrap(), &mut save_state, *auto)
            {
                log::warn!("cannot save: {:?}", e);
            }
            crate::saveload::check_save_files_limit(&mut save_state, &conf);
            None
        }
        ManagePlanet::Load {
            sub_dir_name,
            auto,
            n,
        } => {
            save_state.change_current(sub_dir_name, false);
            match crate::saveload::load_from(&save_state, *auto, *n) {
                Ok((planet, metadata)) => {
                    save_state.save_file_metadata = metadata;
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
            }
        }
        ManagePlanet::Delete {
            sub_dir_name,
            all,
            auto,
            n,
        } => {
            if *all {
                if let Err(e) = crate::platform::delete_save_sub_dir(sub_dir_name) {
                    log::warn!("cannot delete save sub dir: {:?}", e);
                }
            } else {
                let file_name = crate::saveload::save_file_name(*auto, *n);
                log::info!("delete save file \"{}/{}\"", sub_dir_name, file_name);
                if let Err(e) = crate::platform::delete_savefile(sub_dir_name, &file_name) {
                    log::warn!("cannot delete save file: {:?}", e);
                }
                if *sub_dir_name == save_state.current {
                    if *auto {
                        save_state.auto_save_files.remove(n);
                    } else {
                        save_state.manual_save_files.remove(n);
                    }
                }
            }

            // If the sub dir has been deleted
            if save_state.auto_save_files.is_empty() && save_state.auto_save_files.is_empty() {
                save_state.dirs.retain(|(_, s)| s != sub_dir_name);
            }
            None
        }
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
