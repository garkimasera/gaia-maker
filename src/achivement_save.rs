use std::time::Duration;

use anyhow::Context;
use base64::Engine;
use bevy::prelude::*;
use fnv::FnvHashSet;
use num_traits::FromPrimitive;

use crate::{
    GameState,
    audio::SoundEffectPlayer,
    planet::{Achivement, Params, Planet, Sim, check_achivements},
};

#[derive(Debug, Resource)]
pub struct UnlockedAchivements(pub FnvHashSet<Achivement>);

#[derive(Default, Debug, Resource)]
pub struct AchivementNotification {
    pub achivement: Option<Achivement>,
    timer: Option<Timer>,
}

const ACHIVEMENT_FILE_NAME: &str = "saves/achivements";

const CHECK_ACHIVEMENT_INTERVAL_CYCLES: u64 = 10;

const ACHIVEMENT_NOTIFICATION_DURATION: Duration = Duration::from_secs(4);

#[derive(Debug)]
pub struct AchivementPlugin;

impl Plugin for AchivementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AchivementNotification>()
            .add_systems(OnExit(GameState::AssetLoading), load_unlocked_achivement)
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
            rmp_serde::from_slice::<Vec<u16>>(&data).context("deserialize achivement data")
        }) {
        Ok(achivement_data) => {
            for achivement_number in achivement_data {
                if let Some(achivement) = Achivement::from_u16(achivement_number) {
                    achivements.insert(achivement);
                } else {
                    log::warn!("unknown achivement in file: {}", achivement_number);
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
    params: Res<Params>,
    mut unlocked_achivements: ResMut<UnlockedAchivements>,
    mut achivement_notification: ResMut<AchivementNotification>,
    mut sim: ResMut<Sim>,
    se_player: SoundEffectPlayer,
    time: Res<Time<Real>>,
) {
    if planet.cycles % CHECK_ACHIVEMENT_INTERVAL_CYCLES != 0 {
        return;
    }

    check_achivements(
        &planet,
        &unlocked_achivements.0,
        &mut sim.new_achievements,
        &params,
    );

    let mut unlocked = false;

    if let Some(timer) = &mut achivement_notification.timer {
        timer.tick(time.delta());
        if timer.finished() {
            *achivement_notification = AchivementNotification::default();
        }
    }

    for new_achivement in sim.new_achievements.drain() {
        if unlocked_achivements.0.contains(&new_achivement) {
            continue;
        }

        log::info!("get achivement {:?}", new_achivement);
        unlocked_achivements.0.insert(new_achivement);
        unlocked = true;
        achivement_notification.achivement = Some(new_achivement);
        achivement_notification.timer = Some(Timer::new(
            ACHIVEMENT_NOTIFICATION_DURATION,
            TimerMode::Once,
        ));
    }

    if unlocked {
        se_player.play("achivement");

        let list: Vec<u16> = unlocked_achivements
            .0
            .iter()
            .map(|achivement| *achivement as u16)
            .collect();
        let data = rmp_serde::to_vec(&list).expect("serialize achivement data");
        let data = base64::prelude::BASE64_STANDARD.encode(data);
        if let Err(e) = crate::platform::write_data_file(ACHIVEMENT_FILE_NAME, &data) {
            log::warn!("cannot write achivement data: {}", e);
        }
    }
}
