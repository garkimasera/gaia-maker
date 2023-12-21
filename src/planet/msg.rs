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
}

impl MsgHolder {
    pub fn append(&mut self, cycles: u64, kind: MsgKind) {
        self.count = Reverse(self.count.0.wrapping_add(1));
        self.msgs.insert(self.count, Msg { cycles, kind });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Msg> {
        self.msgs.values()
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

    pub fn iter_temp(&self) -> impl Iterator<Item = &Msg> {
        self.temp_msgs.iter()
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum MsgKind {
    Welcome,
    WarnHighTemp,
    WarnLowTemp,
}
