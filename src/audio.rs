use std::{collections::HashSet, time::Duration};

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::prelude::*;
use serde::Deserialize;

use crate::{
    GameState,
    assets::SoundEffectSources,
    conf::{Conf, ConfChange, ConfLoadSystemSet},
    planet::Planet,
};

const N_RETRY_CHOOSE_MUSIC: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct GameAudioPlugin;

#[derive(Clone, Copy, Debug, Resource)]
pub struct SEChannel;

pub type AudioChannelSE = AudioChannel<SEChannel>;

#[derive(Default, Debug, Resource)]
struct BgmState {
    instance: Option<Handle<AudioInstance>>,
    stop_timer_fadeout: Option<(Timer, f64)>,
    kind: Option<MusicKind>,
    path: String,
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
            .init_resource::<BgmState>()
            .add_systems(OnEnter(GameState::Running), stop_main_menu_bgm)
            .add_systems(
                OnExit(GameState::AssetLoading),
                init_volume.after(ConfLoadSystemSet),
            )
            .add_systems(Update, on_conf_change.run_if(in_state(GameState::Running)))
            .add_systems(
                Update,
                check_main_menu_bgm.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                Update,
                check_planet_bgm.run_if(in_state(GameState::Running)),
            );
    }
}

fn stop_main_menu_bgm(mut bgm_state: ResMut<BgmState>, channel: Res<AudioChannel<MainTrack>>) {
    channel.stop().fade_out(AudioTween::new(
        Duration::from_secs_f64(3.0),
        AudioEasing::InPowi(3),
    ));
    bgm_state.kind = None;
}

fn check_main_menu_bgm(
    mut bgm_state: ResMut<BgmState>,
    list: Res<MusicList>,
    channel: Res<AudioChannel<MainTrack>>,
) {
    if bgm_state
        .instance
        .as_ref()
        .is_none_or(|instance| matches!(channel.state(instance), PlaybackState::Stopped))
    {
        if let Some(item) = &list.main_menu {
            let handle = channel.play(item.handle.clone()).looped().handle();
            bgm_state.instance = Some(handle);
            bgm_state.path = item.path.clone();
            bgm_state.kind = Some(item.kind);
        }
    }

    if bgm_state.kind.is_some_and(|kind| kind != MusicKind::MainMenu) {
        channel.stop().fade_out(AudioTween::new(
            Duration::from_secs_f64(3.0),
            AudioEasing::InPowi(3),
        ));
        bgm_state.stop_timer_fadeout = None;
        bgm_state.kind = None;
    }
}

fn check_planet_bgm(
    mut bgm_state: ResMut<BgmState>,
    time: Res<Time<Real>>,
    list: Res<MusicList>,
    channel: Res<AudioChannel<MainTrack>>,
    planet: Res<Planet>,
) {
    if bgm_state
        .instance
        .as_ref()
        .is_none_or(|instance| matches!(channel.state(instance), PlaybackState::Stopped))
    {
        for i in 0..N_RETRY_CHOOSE_MUSIC {
            if let Some(item) = list.choose(&planet) {
                if item.path == bgm_state.path && i < N_RETRY_CHOOSE_MUSIC - 1 {
                    continue;
                }
                play_bgm(&mut bgm_state, item, &channel);
            }
            break;
        }
        return;
    }

    if let Some((timer, fade_out)) = &mut bgm_state.stop_timer_fadeout {
        timer.tick(time.delta());
        if timer.finished() {
            channel.stop().fade_out(AudioTween::new(
                Duration::from_secs_f64(*fade_out),
                AudioEasing::InPowi(3),
            ));
            bgm_state.stop_timer_fadeout = None;
        }
    }
}

fn play_bgm(bgm_state: &mut BgmState, item: &MusicItem, channel: &AudioChannel<MainTrack>) {
    channel.stop();
    let handle = item.handle.clone();
    let mut c = channel.play(handle);

    let n_loop = item.n_loop.unwrap_or(1);
    if n_loop > 1 {
        let mut l = item.length.unwrap_or_default();
        c.looped();
        if let Some(loop_from) = item.loop_from {
            c.loop_from(loop_from);
            l -= loop_from;
        }
        if let Some(loop_until) = item.loop_until {
            c.loop_from(loop_until);
            l -= item.length.unwrap_or_default() - loop_until;
        }
        let length = l * item.n_loop.unwrap_or(1) as f64
            + item.loop_from.unwrap_or_default()
            + item.loop_until.unwrap_or_default();
        let fade_out = item.fade_out.unwrap_or_default();
        let timer = Timer::new(Duration::from_secs_f64(length - fade_out), TimerMode::Once);
        bgm_state.stop_timer_fadeout = Some((timer, fade_out));
    }
    bgm_state.instance = Some(c.handle());
    bgm_state.path = item.path.clone();
    bgm_state.kind = Some(item.kind);
}

pub fn set_music_list(
    mut commands: Commands,
    music_list_assets: Res<Assets<MusicListAsset>>,
    asset_server: Res<AssetServer>,
) {
    let mut list = MusicList::default();

    for item in music_list_assets.iter().flat_map(|list| &list.1.0) {
        let item = MusicItem {
            kind: item.kind,
            path: item.path.clone(),
            handle: asset_server.load(format!("music/{}", item.path)),
            length: item.length,
            n_loop: item.n_loop,
            fade_out: item.fade_out,
            loop_from: item.loop_from,
            loop_until: item.loop_until,
        };
        if matches!(item.kind, MusicKind::MainMenu) {
            list.main_menu = Some(item);
        } else {
            list.random_planet.push(item);
        }
    }

    commands.insert_resource(list);
}

#[derive(Clone, Default, Debug, Resource)]
struct MusicList {
    main_menu: Option<MusicItem>,
    random_planet: Vec<MusicItem>,
}

#[derive(Clone, Debug)]
struct MusicItem {
    kind: MusicKind,
    path: String,
    handle: Handle<AudioSource>,
    n_loop: Option<u32>,
    length: Option<f64>,
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
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    n_loop: Option<u32>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    length: Option<f64>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    fade_out: Option<f64>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    loop_from: Option<f64>,
    #[serde(default, with = "serde_with::rust::unwrap_or_skip")]
    loop_until: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Deserialize)]
enum MusicKind {
    MainMenu,
    RandomPlanet,
    Uncivilized,
    Civilization,
    Industrial,
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

impl MusicList {
    fn choose(&self, planet: &Planet) -> Option<&MusicItem> {
        use rand::distr::Distribution;
        let kind_set = music_kind_set_by_planet_state(planet);

        let w = self.random_planet.iter().map(|item| {
            if kind_set.contains(&item.kind) {
                1.0
            } else {
                0.0
            }
        });
        let w = rand::distr::weighted::WeightedIndex::new(w).ok()?;
        self.random_planet.get(w.sample(&mut rand::rng()))
    }
}

fn music_kind_set_by_planet_state(planet: &Planet) -> HashSet<MusicKind> {
    use crate::planet::*;

    let mut kind_list = HashSet::default();
    kind_list.insert(MusicKind::RandomPlanet);
    kind_list.insert(MusicKind::Uncivilized);

    for civ in planet.civs.values() {
        if civ.total_pop > 0.0 {
            kind_list.remove(&MusicKind::Uncivilized);
        }
        if civ.total_pop > 1000.0 {
            kind_list.insert(MusicKind::Civilization);
        }
        if civ.most_advanced_age >= CivilizationAge::Industrial && civ.total_pop > 10000.0 {
            kind_list.insert(MusicKind::Industrial);
        }
    }

    kind_list
}
