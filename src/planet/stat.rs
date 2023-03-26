use std::collections::VecDeque;

use super::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub average_air_temp: f32,
    pub average_rainfall: f32,
    history: VecDeque<Record>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub average_air_temp: f32,
    pub average_rainfall: f32,
    pub p_o2: f32,
    pub p_n2: f32,
    pub p_co2: f32,
}

impl Stat {
    pub fn new(params: &Params) -> Self {
        Self {
            average_air_temp: 0.0,
            average_rainfall: 0.0,
            history: VecDeque::with_capacity(params.history.max_record + 1),
        }
    }

    pub fn history(&self) -> &VecDeque<Record> {
        &self.history
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

pub fn update_stats(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.history.interval_cycles != 0 {
        return;
    }

    let record = Record {
        average_air_temp: planet.stat.average_air_temp,
        average_rainfall: planet.stat.average_rainfall,
        p_o2: planet.atmo.partial_pressure(GasKind::Oxygen),
        p_n2: planet.atmo.partial_pressure(GasKind::Nitrogen),
        p_co2: planet.atmo.partial_pressure(GasKind::CarbonDioxide),
    };

    planet.stat.history.push_front(record);
    if planet.stat.history.len() > params.history.max_record {
        planet.stat.history.pop_back();
    }
}
