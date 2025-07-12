use std::collections::{HashMap, VecDeque};

use super::*;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub average_air_temp: f32,
    pub average_sea_temp: f32,
    pub average_rainfall: f32,
    pub sum_biomass: f32,
    pub sum_buried_carbon: f32,
    history: VecDeque<Record>,
    #[serde(default)]
    pub animals: HashMap<AnimalId, f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub average_air_temp: f32,
    pub average_sea_temp: f32,
    pub average_rainfall: f32,
    pub biomass: f32,
    pub buried_carbon: f32,
    pub p_o2: f32,
    pub p_n2: f32,
    pub p_co2: f32,
    pub pop: fnv::FnvHashMap<AnimalId, f32>,
}

impl Stat {
    pub fn new(params: &Params) -> Self {
        Self {
            average_air_temp: 0.0,
            average_sea_temp: 0.0,
            average_rainfall: 0.0,
            sum_biomass: 0.0,
            sum_buried_carbon: 0.0,
            animals: HashMap::default(),
            history: VecDeque::with_capacity(params.history.max_record + 1),
        }
    }

    pub fn history(&self) -> &VecDeque<Record> {
        &self.history
    }

    pub fn record(&self, cycles: u64, params: &Params) -> Option<&Record> {
        let n = cycles / params.history.interval_cycles;
        self.history.get(n as usize)
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

pub fn record_stats(planet: &mut Planet, params: &Params) {
    if planet.cycles % params.history.interval_cycles != 0 {
        return;
    }

    let mut pop = fnv::FnvHashMap::default();
    for p in planet.map.iter_idx() {
        if let Some(Structure::Settlement(settlement)) = &planet.map[p].structure {
            *pop.entry(settlement.id).or_default() += settlement.pop;
        }
    }

    let record = Record {
        average_air_temp: planet.stat.average_air_temp,
        average_sea_temp: planet.stat.average_sea_temp,
        average_rainfall: planet.stat.average_rainfall,
        biomass: planet.stat.sum_biomass,
        buried_carbon: planet.stat.sum_buried_carbon,
        p_o2: planet.atmo.partial_pressure(GasKind::Oxygen),
        p_n2: planet.atmo.partial_pressure(GasKind::Nitrogen),
        p_co2: planet.atmo.partial_pressure(GasKind::CarbonDioxide),
        pop,
    };

    planet.stat.history.push_front(record);
    if planet.stat.history.len() > params.history.max_record {
        planet.stat.history.pop_back();
    }
}

impl Record {
    pub fn pop(&self, animal_id: Option<AnimalId>) -> f32 {
        if let Some(animal_id) = animal_id {
            self.pop.get(&animal_id).copied().unwrap_or_default()
        } else {
            self.pop.values().sum()
        }
    }
}
