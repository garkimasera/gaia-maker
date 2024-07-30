use geom::Coords;
use std::sync::LazyLock;
use std::{collections::BTreeMap, sync::RwLock};

static POS_FOR_LOG: LazyLock<RwLock<Option<Coords>>> = LazyLock::new(RwLock::default);
static TILE_LOGS: LazyLock<RwLock<BTreeMap<&'static str, String>>> = LazyLock::new(RwLock::default);

pub fn clear_logs(p: Option<Coords>) {
    *POS_FOR_LOG.write().unwrap() = p;
    TILE_LOGS.write().unwrap().clear();
}

pub fn tile_logs() -> impl std::ops::Deref<Target = BTreeMap<&'static str, String>> {
    TILE_LOGS.read().unwrap()
}

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
