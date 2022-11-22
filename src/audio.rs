use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use strum::{AsRefStr, EnumIter, EnumString};

#[derive(Clone, Copy, Debug)]
pub struct GameAudioPlugin;

#[derive(Clone, Copy, Debug, Resource)]
pub struct SEChannel;

pub type AudioSE = AudioChannel<SEChannel>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumString, EnumIter, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum SoundEffect {
    Build,
}

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin).add_audio_channel::<SEChannel>();
    }
}
