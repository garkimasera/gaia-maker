use compact_str::format_compact;

use crate::planet::{Report, ReportContent};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum WithUnitDisplay {
    Power(f32),
    Material(f32),
    GenePoint(f32),
}

impl std::fmt::Display for WithUnitDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match *self {
            WithUnitDisplay::Power(value) => {
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

impl Report {
    pub fn text(&self) -> (MsgStyle, String) {
        use MsgStyle::*;
        match &self.content {
            ReportContent::WarnHighTemp => (Warn, t!("report/warn-high-temp")),
            ReportContent::WarnLowTemp => (Warn, t!("report/warn-low-temp")),
            ReportContent::WarnLowOxygen => (Warn, t!("report/warn-low-oxygen")),
            ReportContent::WarnLowCarbonDioxide => (Warn, t!("report/warn-low-carbon-dioxide")),
            ReportContent::EventCivilized { animal, .. } => {
                let animal = t!("animal", animal);
                (Notice, t!("report/civilized"; animal = animal))
            }
            ReportContent::EventCivAdvance { name, age, id, .. } => {
                let name = if let Some(name) = name {
                    name.to_owned()
                } else {
                    t!("civ", id)
                };
                let age = t!("age", age);
                (Notice, t!("report/civ-advance"; name = name, age = age))
            }
            ReportContent::EventCivExtinct { name, id } => {
                let name = if let Some(name) = name {
                    name.to_owned()
                } else {
                    t!("civ", id)
                };
                (Notice, t!("report/civ-extinct"; name = name))
            }
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

pub fn format_float_1000(value: f32, precision: usize) -> compact_str::CompactString {
    if value > 1000.0 || precision == 0 {
        format_compact!("{:.0}", value)
    } else if value > 100.0 || precision == 1 {
        format_compact!("{:.1}", value)
    } else if value > 10.0 || precision == 2 {
        format_compact!("{:.2}", value)
    } else if value > 1.0 || precision == 3 {
        format_compact!("{:.3}", value)
    } else {
        format_compact!("{}", value)
    }
}
