use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::{LazyLock, RwLock};

use geom::Coords;

use crate::planet::*;

static POS_FOR_LOG: LazyLock<RwLock<Option<Coords>>> = LazyLock::new(RwLock::default);
static TILE_LOGS: LazyLock<RwLock<BTreeMap<&'static str, String>>> = LazyLock::new(RwLock::default);

pub fn clear_logs(p: Option<Coords>) {
    *POS_FOR_LOG.write().unwrap() = p;
    TILE_LOGS.write().unwrap().clear();
}

pub fn tile_logs() -> impl std::ops::Deref<Target = BTreeMap<&'static str, String>> {
    TILE_LOGS.read().unwrap()
}

#[allow(unused)]
pub(super) fn tile_log<F: FnOnce(Coords) -> T, T: ToString>(
    target: Coords,
    name: &'static str,
    f: F,
) {
    if *POS_FOR_LOG.read().unwrap() == Some(target) {
        TILE_LOGS.write().unwrap().insert(name, f(target).to_string());
    }
}

pub fn tile_debug_info(planet: &Planet, sim: &Sim, p: Coords) -> Vec<(&'static str, String)> {
    let mut v = Vec::new();

    v.push(("height", format!("{:.1}", planet.map[p].height)));
    v.push(("humidity", format!("{:.1}", sim.humidity[p])));
    v.push(("albedo", format!("{}", sim.albedo[p])));
    v.push((
        "sea_temp",
        format!("{:.1}", planet.map[p].sea_temp - KELVIN_CELSIUS),
    ));
    v.push(("ice", format!("{}", planet.map[p].ice)));
    v.push((
        "buried carbon",
        format!("{:.2e}", planet.map[p].buried_carbon),
    ));
    v.push((
        "animal0",
        animals_debug_text_in_tile(&planet.map[p].animal[0]),
    ));
    v.push((
        "animal1",
        animals_debug_text_in_tile(&planet.map[p].animal[1]),
    ));
    v.push((
        "animal2",
        animals_debug_text_in_tile(&planet.map[p].animal[2]),
    ));
    v.push((
        "pop, tech_exp",
        match &planet.map[p].structure {
            Some(Structure::Settlement(settlement)) => {
                format!(
                    "{}: {:.2}, {:+.1}",
                    settlement.id, settlement.pop, settlement.tech_exp,
                )
            }
            _ => "-".into(),
        },
    ));
    v.push((
        "settlement state",
        match &planet.map[p].structure {
            Some(Structure::Settlement(settlement)) => {
                format!(
                    "{} {}",
                    settlement.state.as_ref(),
                    settlement.since_state_changed,
                )
            }
            _ => "-".into(),
        },
    ));
    v.push((
        "strength",
        match &planet.map[p].structure {
            Some(Structure::Settlement(settlement)) => {
                format!("{}", settlement.str)
            }
            _ => "-".into(),
        },
    ));
    v.push(("energy efficiency", format!("{:.1}", sim.energy_eff[p])));

    v
}

fn animals_debug_text_in_tile(animal: &Option<Animal>) -> String {
    let Some(animal) = animal else {
        return "-".into();
    };

    format!("{}(n={:.3})", animal.id, animal.n)
}

pub trait PlanetDebug {
    fn edit_biome(&mut self, p: Coords, biome: Biome);
    fn change_height(&mut self, p: Coords, value: f32, sim: &mut Sim, params: &Params);
    fn place_settlement(&mut self, p: Coords, settlement: Settlement);
    fn cause_decadence(&mut self, p: Coords, sim: &mut Sim, params: &Params);
    fn cause_civil_war(&mut self, p: Coords, sim: &mut Sim, params: &Params);
    fn cause_nuclear_explosion(&mut self, p: Coords, sim: &mut Sim, params: &Params);
    fn delete_civilization(&mut self);
    fn height_map_as_string(&self) -> String;
}

impl PlanetDebug for Planet {
    fn edit_biome(&mut self, p: Coords, biome: Biome) {
        self.map[p].biome = biome;
    }

    fn change_height(&mut self, p: Coords, value: f32, sim: &mut Sim, params: &Params) {
        let h = &mut self.map[p].height;
        *h = (*h + value).max(0.0);
        super::water::update_sea_level(self, sim, params);
    }

    fn place_settlement(&mut self, p: Coords, settlement: Settlement) {
        self.map[p].structure = Some(Structure::Settlement(settlement));
    }

    fn cause_decadence(&mut self, p: Coords, sim: &mut Sim, params: &Params) {
        super::decadence::cause_decadence(self, sim, params, p);
    }

    fn cause_civil_war(&mut self, p: Coords, sim: &mut Sim, params: &Params) {
        if let Some(Structure::Settlement(settlement)) = self.map[p].structure.clone() {
            super::war::start_civil_war(self, sim, params, p, settlement);
        }
    }

    fn cause_nuclear_explosion(&mut self, p: Coords, _sim: &mut Sim, params: &Params) {
        self.map[p].tile_events.insert(TileEvent::NuclearExplosion {
            remaining_cycles: params.event.nuclear_explosion_cycles,
        });
    }

    fn delete_civilization(&mut self) {
        for p in self.map.iter_idx() {
            if matches!(self.map[p].structure, Some(Structure::Settlement(_))) {
                self.map[p].structure = None;
            }
            self.map[p].tile_events.remove(TileEventKind::Vehicle);
        }
    }

    fn height_map_as_string(&self) -> String {
        let mut s = String::new();
        write!(s, "[").unwrap();
        for (i, p) in self.map.iter_idx().enumerate() {
            let separator = if i == 0 { "" } else { "," };
            let newline = if i % 64 == 0 { "\n" } else { "" };
            write!(s, "{}{}{:.2}", separator, newline, self.map[p].height).unwrap();
        }
        write!(s, "]").unwrap();
        s
    }
}
