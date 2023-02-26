use bevy::prelude::*;
use std::collections::VecDeque;

use crate::{planet::Sim, ui::WindowsOpenState, GameState};

#[derive(Clone, Copy, Debug)]
pub struct MsgPlugin;

impl Plugin for MsgPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MsgHolder>()
            .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup_msgs))
            .add_system_set(SystemSet::on_update(GameState::Running).with_system(update_msgs));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, strum::AsRefStr)]
pub enum MsgKind {
    Notice,
    _Warn,
}

#[derive(Default, Debug, Resource)]
pub struct MsgHolder {
    msgs: VecDeque<(MsgKind, String)>,
}

impl MsgHolder {
    pub fn latest(&self) -> (MsgKind, String) {
        if let Some((msg_kind, s)) = self.msgs.front() {
            (*msg_kind, s.clone())
        } else {
            (MsgKind::Notice, t!("welcome_to"; app_name=crate::APP_NAME))
        }
    }
}

fn setup_msgs(_msg_holder: ResMut<MsgHolder>) {}

fn update_msgs(
    mut msg_holder: ResMut<MsgHolder>,
    mut sim: ResMut<Sim>,
    mut wos: ResMut<WindowsOpenState>,
) {
    if let Some(event) = sim.events.pop_front() {
        wos.message = true;
        msg_holder
            .msgs
            .push_front((MsgKind::Notice, t!(format!("event/{}", event.as_ref()))));
    }
}
