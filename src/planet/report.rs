use geom::Coords;
use serde::{Deserialize, Serialize};

use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::mem::discriminant;

use super::AnimalId;

const COMMON_NOTICE_REPORT_SPAN: u64 = 1000;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Reports {
    count: Reverse<u64>,
    reports: BTreeMap<Reverse<u64>, Report>,
    persistent_warns: Vec<Report>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub cycles: u64,
    pub content: ReportContent,
}

impl Reports {
    pub fn append(&mut self, cycles: u64, content: ReportContent) {
        self.count = Reverse(self.count.0.wrapping_add(1));
        self.reports.insert(self.count, Report { cycles, content });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Report> {
        ReportIter {
            reports: self.reports.values().peekable(),
            temp_reports: self.persistent_warns.iter().peekable(),
        }
    }

    pub fn append_persitent_warn(&mut self, cycles: u64, content: ReportContent) {
        let new_report = Report { cycles, content };
        if let Some(report) = self
            .persistent_warns
            .iter_mut()
            .find(|report| report.content == new_report.content)
        {
            *report = new_report
        } else {
            self.persistent_warns.push(new_report);
        }
    }

    pub fn remove_persitent_warn(&mut self, target: &ReportContent) {
        self.persistent_warns
            .retain(|report| discriminant(&report.content) != discriminant(target));
    }

    pub fn remove_outdated(&mut self, cycles: u64) {
        self.reports.retain(|_, report| {
            if let Some(span) = report.content.span() {
                report.cycles + span > cycles
            } else {
                true
            }
        });
    }
}

struct ReportIter<'a> {
    reports: std::iter::Peekable<std::collections::btree_map::Values<'a, Reverse<u64>, Report>>,
    temp_reports: std::iter::Peekable<std::slice::Iter<'a, Report>>,
}

impl<'a> Iterator for ReportIter<'a> {
    type Item = &'a Report;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.reports.peek(), self.temp_reports.peek()) {
            (Some(report), Some(temp_report)) => {
                if report.cycles > temp_report.cycles {
                    self.reports.next()
                } else {
                    self.temp_reports.next()
                }
            }
            (Some(_), None) => self.reports.next(),
            (None, Some(_)) => self.temp_reports.next(),
            (None, None) => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ReportContent {
    WarnHighTemp,
    WarnLowTemp,
    WarnLowOxygen,
    EventCivilized { pos: Coords, animal: AnimalId },
}

impl ReportContent {
    pub fn span(&self) -> Option<u64> {
        match self {
            Self::EventCivilized { .. } => Some(COMMON_NOTICE_REPORT_SPAN),
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
