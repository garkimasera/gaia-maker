use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::prelude::*;

use crate::assets::SoundEffectSources;

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
        app.add_plugins(AudioPlugin).add_audio_channel::<SEChannel>();
    }
}
