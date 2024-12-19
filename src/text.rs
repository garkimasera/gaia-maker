use crate::planet::{Msg, MsgKind};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WithUnitDisplay {
    Energy(f32),
    Material(f32),
    GenePoint(f32),
}

impl std::fmt::Display for WithUnitDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            WithUnitDisplay::Energy(value) => {
                write!(f, "{}TW", value)
            }
            WithUnitDisplay::Material(value) => {
                if value < 1.0 {
                    write!(f, "{}Mt", value)
                } else if value < 100000.0 {
                    write!(f, "{:.0}Mt", value)
                } else {
                    write!(f, "{:.0}Gt", value / 1000.0)
                }
            }
            WithUnitDisplay::GenePoint(value) => {
                write!(f, "{:.0}", value)
            }
        }
    }
}

impl Msg {
    pub fn text(&self) -> (MsgStyle, String) {
        use MsgStyle::*;
        match &self.kind {
            MsgKind::WarnHighTemp => (Warn, t!("msg/warn-high-temp")),
            MsgKind::WarnLowTemp => (Warn, t!("msg/warn-low-temp")),
            MsgKind::WarnLowOxygen => (Warn, t!("msg/warn-low-oxygen")),
            MsgKind::EventStart => (Notice, t!("event/start")),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MsgStyle {
    Notice,
    Warn,
}

impl MsgStyle {
    pub fn icon(&self) -> &str {
        match self {
            MsgStyle::Notice => "ℹ",
            MsgStyle::Warn => "⚠",
        }
    }
}

pub fn split_to_head_body(s: &str) -> (&str, Option<&str>) {
    if let Some((head, body)) = s.split_once('\n') {
        (head, Some(body))
    } else {
        (s, None)
    }
}
