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
        TILE_LOGS
            .write()
            .unwrap()
            .insert(name, f(target).to_string());
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
                    settlement.id, settlement.pop, settlement.tech_exp
                )
            }
            _ => "0".into(),
        },
    ));

    v
}

fn animals_debug_text_in_tile(animal: &Option<Animal>) -> String {
    let Some(animal) = animal else {
        return "Empty".into();
    };

    format!("{}(n={:.3})", animal.id, animal.n)
}

pub trait PlanetDebug {
    fn edit_biome(&mut self, p: Coords, biome: Biome);
    fn change_height(&mut self, p: Coords, value: f32, sim: &mut Sim, params: &Params);
    fn place_settlement(&mut self, p: Coords, settlement: Settlement);
    fn delete_settlement(&mut self);
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

    fn delete_settlement(&mut self) {
        for p in self.map.iter_idx() {
            if matches!(self.map[p].structure, Some(Structure::Settlement(_))) {
                self.map[p].structure = None;
            }
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
