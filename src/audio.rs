use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::assets::AudioSources;

#[derive(Clone, Copy, Debug)]
pub struct GameAudioPlugin;

#[derive(Clone, Copy, Debug, Resource)]
pub struct SEChannel;

pub type AudioChannelSE = AudioChannel<SEChannel>;

pub type ResAudioPlayer<'a> = (Res<'a, AudioSources>, Res<'a, AudioChannelSE>);

pub trait AudioPlayer {
    fn play_se(&self, s: &str);
}

impl<'a> AudioPlayer for ResAudioPlayer<'a> {
    fn play_se(&self, s: &str) {
        let path = compact_str::format_compact!("se/{}.ogg", s);
        let Some(audio_source) = self.0.sound_effects.get(path.as_str()) else {
            log::warn!("unknown sound effect {}", path);
            return;
        };
        self.1.play(audio_source.clone());
    }
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<SEChannel>();
    }
}
