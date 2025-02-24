use geom::Coords;
use serde::{Deserialize, Serialize};

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::mem::discriminant;

use super::AnimalId;

const COMMON_NOTICE_MSG_SPAN: u64 = 1000;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgHolder {
    count: Reverse<u64>,
    msgs: BTreeMap<Reverse<u64>, Msg>,
    persistent_warns: Vec<Msg>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub cycles: u64,
    pub content: MsgContent,
}

impl MsgHolder {
    pub fn append(&mut self, cycles: u64, content: MsgContent) {
        self.count = Reverse(self.count.0.wrapping_add(1));
        self.msgs.insert(self.count, Msg { cycles, content });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Msg> {
        MsgIter {
            msgs: self.msgs.values().peekable(),
            temp_msgs: self.persistent_warns.iter().peekable(),
        }
    }

    pub fn append_persitent_warn(&mut self, cycles: u64, content: MsgContent) {
        let new_msg = Msg { cycles, content };
        if let Some(msg) = self
            .persistent_warns
            .iter_mut()
            .find(|msg| msg.content == new_msg.content)
        {
            *msg = new_msg
        } else {
            self.persistent_warns.push(new_msg);
        }
    }

    pub fn remove_persitent_warn(&mut self, target: &MsgContent) {
        self.persistent_warns
            .retain(|msg| discriminant(&msg.content) != discriminant(target));
    }

    pub fn remove_outdated(&mut self, cycles: u64) {
        self.msgs.retain(|_, msg| {
            if let Some(span) = msg.content.span() {
                msg.cycles + span > cycles
            } else {
                true
            }
        });
    }
}

struct MsgIter<'a> {
    msgs: std::iter::Peekable<std::collections::btree_map::Values<'a, Reverse<u64>, Msg>>,
    temp_msgs: std::iter::Peekable<std::slice::Iter<'a, Msg>>,
}

impl<'a> Iterator for MsgIter<'a> {
    type Item = &'a Msg;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.msgs.peek(), self.temp_msgs.peek()) {
            (Some(msg), Some(temp_msg)) => {
                if msg.cycles > temp_msg.cycles {
                    self.msgs.next()
                } else {
                    self.temp_msgs.next()
                }
            }
            (Some(_), None) => self.msgs.next(),
            (None, Some(_)) => self.temp_msgs.next(),
            (None, None) => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgContent {
    WarnHighTemp,
    WarnLowTemp,
    WarnLowOxygen,
    EventCivilized { pos: Coords, animal: AnimalId },
}

impl MsgContent {
    pub fn span(&self) -> Option<u64> {
        match self {
            Self::EventCivilized { .. } => Some(COMMON_NOTICE_MSG_SPAN),
            _ => None,
        }
    }

    pub fn pos(&self) -> Option<Coords> {
        match self {
            Self::EventCivilized { pos, .. } => Some(*pos),
            _ => None,
        }
    }
}
