use geom::Coords;
use std::sync::LazyLock;
use std::{collections::BTreeMap, sync::RwLock};

use crate::planet::{Animal, Planet, Sim, Structure, KELVIN_CELSIUS};

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
