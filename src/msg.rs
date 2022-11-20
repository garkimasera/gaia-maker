use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Clone, Copy, PartialEq, Eq, Debug, strum::AsRefStr)]
pub enum MsgKind {
    Notice,
    _Warn,
}

static MSG_QUEUE: Lazy<Mutex<VecDeque<(MsgKind, String)>>> =
    Lazy::new(|| Mutex::new(VecDeque::default()));

pub fn push_msg<S: Into<String>>(kind: MsgKind, s: S) {
    let s = s.into();
    MSG_QUEUE.lock().unwrap().push_front((kind, s));
}

pub fn pop_msg() -> Option<(MsgKind, String)> {
    MSG_QUEUE.lock().unwrap().pop_back()
}
