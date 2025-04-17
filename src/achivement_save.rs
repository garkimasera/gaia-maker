use anyhow::Context;
use base64::Engine;
use bevy::prelude::*;
use fnv::FnvHashSet;

use crate::{
    GameState,
    planet::{Achivement, Planet, Sim, check_achivements},
};

#[derive(Debug, Resource)]
pub struct UnlockedAchivements(FnvHashSet<Achivement>);

const ACHIVEMENT_FILE_NAME: &str = "saves/achivements.planet";

const CHECK_ACHIVEMENT_INTERVAL_CYCLES: u64 = 10;

#[derive(Debug)]
pub struct AchivementPlugin;

impl Plugin for AchivementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::AssetLoading), load_unlocked_achivement)
            .add_systems(
                FixedUpdate,
                check_periodic.run_if(in_state(GameState::Running)),
            );
    }
}

fn load_unlocked_achivement(mut command: Commands) {
    let mut achivements = FnvHashSet::default();

    match crate::platform::read_data_file(ACHIVEMENT_FILE_NAME)
        .and_then(|data| {
            base64::prelude::BASE64_STANDARD
                .decode(data)
                .context("invalid achivement data")
        })
        .and_then(|data| {
            rmp_serde::from_slice::<Vec<String>>(&data).context("deserialize achivement data")
        }) {
        Ok(achivement_data) => {
            for achivement_name in achivement_data {
                match achivement_name.parse::<Achivement>() {
                    Ok(achivement) => {
                        achivements.insert(dbg!(achivement));
                    }
                    Err(_) => {
                        log::warn!("unknown achivement in file: {}", achivement_name);
                    }
                }
            }
        }
        Err(e) => {
            log::warn!("cannot load achivement data: {:?}", e);
        }
    }
    command.insert_resource(UnlockedAchivements(achivements));
}

fn check_periodic(
    planet: Res<Planet>,
    mut unlocked_achivements: ResMut<UnlockedAchivements>,
    mut sim: ResMut<Sim>,
) {
    if planet.cycles % CHECK_ACHIVEMENT_INTERVAL_CYCLES != 0 {
        return;
    }

    check_achivements(&planet, &unlocked_achivements.0, &mut sim.new_achievements);

    let mut need_update_file = false;

    for new_achivement in sim.new_achievements.drain() {
        if unlocked_achivements.0.contains(&new_achivement) {
            continue;
        }

        log::info!("get achivement {:?}", new_achivement);
        unlocked_achivements.0.insert(new_achivement);
        need_update_file = true;
    }

    if need_update_file {
        let list: Vec<String> = unlocked_achivements
            .0
            .iter()
            .map(|achivement| AsRef::<str>::as_ref(&achivement).to_owned())
            .collect();
        let data = rmp_serde::to_vec::<Vec<String>>(&list).expect("serialize achivement data");
        let data = base64::prelude::BASE64_STANDARD.encode(data);
        if let Err(e) = crate::platform::write_data_file(ACHIVEMENT_FILE_NAME, &data) {
            log::warn!("cannot write achivement data: {}", e);
        }
    }
}
