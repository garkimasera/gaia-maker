use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::prelude::*;
use rand::seq::IndexedRandom;
use serde::Deserialize;

use crate::{
    GameState,
    assets::{MusicSources, SoundEffectSources},
    conf::{Conf, ConfChange, ConfLoadSystemSet},
};

#[derive(Clone, Copy, Debug)]
pub struct GameAudioPlugin;

#[derive(Clone, Copy, Debug, Resource)]
pub struct SEChannel;

pub type AudioChannelSE = AudioChannel<SEChannel>;

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
            .add_systems(OnEnter(GameState::Running), play_planet_music)
            .add_systems(
                OnExit(GameState::AssetLoading),
                init_volume.after(ConfLoadSystemSet),
            )
            .add_systems(Update, on_conf_change.run_if(in_state(GameState::Running)));
    }
}

fn play_planet_music(list: Res<MusicList>, channel: Res<AudioChannel<MainTrack>>) {
    if let Some(item) = list.random_planet.choose(&mut rand::rng()) {
        let handle = item.handle.clone();
        let mut c = channel.play(handle);
        c.looped();
        if let Some(loop_from) = item.loop_from {
            c.loop_from(loop_from);
        }
        if let Some(loop_until) = item.loop_until {
            c.loop_from(loop_until);
        }
    }
}

pub fn set_music_list(
    mut commands: Commands,
    sources: Res<MusicSources>,
    music_list_assets: Res<Assets<MusicListAsset>>,
) {
    let mut list = MusicList::default();

    for item in music_list_assets.iter().flat_map(|list| &list.1.0) {
        let path = format!("music/{}", item.path);
        if let Some(handle) = sources.music_handles.get(&path) {
            match handle.clone().try_typed::<AudioSource>() {
                Ok(handle) => {
                    let item = MusicItem {
                        kind: item.kind.clone(),
                        handle,
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
                Err(e) => {
                    log::warn!("{}", e);
                }
            }
        } else {
            log::warn!("not found music: {}", path);
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
    handle: Handle<AudioSource>,
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
