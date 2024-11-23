use serde::{Deserialize, Serialize};

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::mem::discriminant;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MsgHolder {
    count: Reverse<u64>,
    msgs: BTreeMap<Reverse<u64>, Msg>,
    temp_msgs: Vec<Msg>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Msg {
    pub cycles: u64,
    pub kind: MsgKind,
    pub span: Option<u64>,
}

impl MsgHolder {
    pub fn append(&mut self, cycles: u64, kind: MsgKind, span: impl Into<Option<u64>>) {
        self.count = Reverse(self.count.0.wrapping_add(1));
        self.msgs.insert(
            self.count,
            Msg {
                cycles,
                kind,
                span: span.into(),
            },
        );
    }

    pub fn iter(&self) -> impl Iterator<Item = &Msg> {
        MsgIter {
            msgs: self.msgs.values().peekable(),
            temp_msgs: self.temp_msgs.iter().peekable(),
        }
    }

    pub fn append_temp(&mut self, new_msg: Msg) {
        if let Some(msg) = self
            .temp_msgs
            .iter_mut()
            .find(|msg| msg.kind == new_msg.kind)
        {
            *msg = new_msg
        } else {
            self.temp_msgs.push(new_msg);
        }
    }

    pub fn remove_temp(&mut self, target: &MsgKind) {
        self.temp_msgs
            .retain(|msg| discriminant(&msg.kind) != discriminant(target));
    }

    pub fn remove_outdated(&mut self, cycles: u64) {
        self.msgs.retain(|_, msg| {
            if let Some(span) = msg.span {
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
pub enum MsgKind {
    WarnHighTemp,
    WarnLowTemp,
    EventStart,
}
