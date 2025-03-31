use std::time::Duration;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::prelude::*;
use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::{
    GameState,
    assets::SoundEffectSources,
    conf::{Conf, ConfChange, ConfLoadSystemSet},
};

const N_RETRY_CHOOSE_MUSIC: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct GameAudioPlugin;

#[derive(Clone, Copy, Debug, Resource)]
pub struct SEChannel;

pub type AudioChannelSE = AudioChannel<SEChannel>;

#[derive(Debug, Component)]
struct Stop {
    path: String,
    timer: Timer,
    fade_out: Option<f64>,
}

#[derive(SystemParam)]
pub struct SoundEffectPlayer<'w> {
    sources: Res<'w, SoundEffectSources>,
    channel_se: Res<'w, AudioChannelSE>,
}

impl SoundEffectPlayer<'_> {
    pub fn play(&self, s: &str) {
        let path = compact_str::format_compact!("se/{}.ogg", s);
        let Some(audio_source) = self.sources.sound_effects.get(path.as_str()) else {
            log::warn!("unknown sound effect {}", path);
            return;
        };
        self.channel_se.stop();
        self.channel_se.play(audio_source.clone());
    }
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<SEChannel>()
            .add_systems(OnEnter(GameState::Running), play_planet_bgm)
            .add_systems(
                OnExit(GameState::AssetLoading),
                init_volume.after(ConfLoadSystemSet),
            )
            .add_systems(Update, on_conf_change.run_if(in_state(GameState::Running)))
            .add_systems(
                Update,
                check_planet_bgm.run_if(in_state(GameState::Running)),
            );
    }
}

fn play_planet_bgm(
    mut commands: Commands,
    list: Res<MusicList>,
    channel: Res<AudioChannel<MainTrack>>,
) {
    if let Some(item) = list.random_planet.choose(&mut rand::rng()) {
        commands.spawn(play_bgm(item, &channel));
    }
}

fn check_planet_bgm(
    mut commands: Commands,
    mut stop_query: Query<(Entity, &mut Stop)>,
    time: Res<Time>,
    list: Res<MusicList>,
    channel: Res<AudioChannel<MainTrack>>,
) {
    let Ok((entity, mut stop)) = stop_query.get_single_mut() else {
        return;
    };
    stop.timer.tick(time.delta());

    if !stop.timer.finished() {
        return;
    }
    commands.entity(entity).despawn();

    if let Some(fade_out) = stop.fade_out {
        commands.spawn(Stop {
            path: stop.path.clone(),
            timer: Timer::new(Duration::from_secs_f64(fade_out), TimerMode::Once),
            fade_out: None,
        });
        channel.stop().fade_out(AudioTween::new(
            Duration::from_secs_f64(fade_out),
            AudioEasing::InPowi(3),
        ));
    } else {
        for i in 0..N_RETRY_CHOOSE_MUSIC {
            if let Some(item) = list.random_planet.choose(&mut rand::rng()) {
                if item.path == stop.path && i < N_RETRY_CHOOSE_MUSIC - 1 {
                    continue;
                }
                commands.spawn(play_bgm(item, &channel));
            }
            break;
        }
    }
}

fn play_bgm(item: &MusicItem, channel: &AudioChannel<MainTrack>) -> Stop {
    channel.stop();
    let handle = item.handle.clone();
    let mut c = channel.play(handle);
    let mut l = item.length;
    if let Some(loop_from) = item.loop_from {
        c.loop_from(loop_from);
        l -= loop_from;
    }
    if let Some(loop_until) = item.loop_until {
        c.loop_from(loop_until);
        l -= loop_until;
    }
    let length = l * item.n_loop.unwrap_or(1) as f64
        + item.loop_from.unwrap_or_default()
        + item.loop_until.unwrap_or_default();

    if let Some(fade_out) = item.fade_out {
        Stop {
            path: item.path.clone(),
            timer: Timer::new(Duration::from_secs_f64(length - fade_out), TimerMode::Once),
            fade_out: Some(fade_out),
        }
    } else {
        Stop {
            path: item.path.clone(),
            timer: Timer::new(Duration::from_secs_f64(length), TimerMode::Once),
            fade_out: None,
        }
    }
}

pub fn set_music_list(
    mut commands: Commands,
    music_list_assets: Res<Assets<MusicListAsset>>,
    asset_server: Res<AssetServer>,
) {
    let mut list = MusicList::default();

    for item in music_list_assets.iter().flat_map(|list| &list.1.0) {
        let item = MusicItem {
            kind: item.kind.clone(),
            path: item.path.clone(),
            handle: asset_server.load(format!("music/{}", item.path)),
            length: item.length,
            n_loop: item.n_loop,
            fade_out: item.fade_out,
            loop_from: item.loop_from,
            loop_until: item.loop_until,
        };
        match item.kind {
            MusicKind::MainMenu => {
                todo!()
            }
            MusicKind::RandomPlanet => {
                list.random_planet.push(item);
            }
        }
    }

    commands.insert_resource(list);
}

#[derive(Clone, Default, Debug, Resource)]
struct MusicList {
    random_planet: Vec<MusicItem>,
}

#[derive(Clone, Debug)]
struct MusicItem {
    kind: MusicKind,
    path: String,
    handle: Handle<AudioSource>,
    length: f64,
    n_loop: Option<u32>,
    fade_out: Option<f64>,
    loop_from: Option<f64>,
    loop_until: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct MusicListAsset(Vec<MusicListAssetItem>);

#[derive(Clone, Debug, Deserialize)]
struct MusicListAssetItem {
    kind: MusicKind,
    path: String,
    #[allow(unused)]
    author: String,
    length: f64,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    n_loop: Option<u32>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    fade_out: Option<f64>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    loop_from: Option<f64>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    loop_until: Option<f64>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
enum MusicKind {
    MainMenu,
    RandomPlanet,
}

fn init_volume(
    conf: Res<Conf>,
    channel_music: Res<AudioChannel<MainTrack>>,
    channel_se: Res<AudioChannelSE>,
) {
    channel_music.set_volume(conf.bgm_volume as f64 / 100.0);
    channel_se.set_volume(conf.sound_effect_volume as f64 / 100.0);
}

fn on_conf_change(
    mut er_conf_change: EventReader<ConfChange>,
    conf: Res<Conf>,
    channel_music: Res<AudioChannel<MainTrack>>,
    channel_se: Res<AudioChannelSE>,
) {
    if er_conf_change.read().last().is_some() {
        channel_music.set_volume(conf.bgm_volume as f64 / 100.0);
        channel_se.set_volume(conf.sound_effect_volume as f64 / 100.0);
    }
}
