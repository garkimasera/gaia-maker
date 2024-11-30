use geom::Coords;
use std::sync::LazyLock;
use std::{collections::BTreeMap, sync::RwLock};

use super::{Animal, Planet, Sim, KELVIN_CELSIUS};

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

pub fn tile_debug_info(planet: &Planet, sim: &Sim, p: Coords) -> BTreeMap<&'static str, String> {
    let mut map = BTreeMap::default();

    map.insert("albedo", format!("{}", sim.albedo[p]));
    map.insert(
        "sea_temp",
        format!("{:.1}", planet.map[p].sea_temp - KELVIN_CELSIUS),
    );
    map.insert("ice", format!("{}", planet.map[p].ice));
    map.insert(
        "buried carbon",
        format!("{:.2e}", planet.map[p].buried_carbon),
    );
    map.insert(
        "animal0",
        animals_debug_text_in_tile(&planet.map[p].animal[0]),
    );
    map.insert(
        "animal1",
        animals_debug_text_in_tile(&planet.map[p].animal[1]),
    );
    map.insert(
        "animal2",
        animals_debug_text_in_tile(&planet.map[p].animal[2]),
    );

    map
}

fn animals_debug_text_in_tile(animal: &Option<Animal>) -> String {
    let Some(animal) = animal else {
        return "Empty".into();
    };

    format!("{}(n={:.3})", animal.id, animal.n)
}
