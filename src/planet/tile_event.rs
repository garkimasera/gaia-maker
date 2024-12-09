use super::*;

pub fn advance(planet: &mut Planet, _sim: &mut Sim, _params: &Params) {
    for p in planet.map.iter_idx() {
        let Some(_event) = planet.map[p].event.as_mut() else {
            continue;
        };
    }
}

pub fn cause_tile_event(
    planet: &mut Planet,
    p: Coords,
    kind: TileEventKind,
    _sim: &mut Sim,
    _params: &Params,
) {
    let event = match kind {
        TileEventKind::Fire => TileEvent::Fire,
        TileEventKind::Plague => todo!(),
    };

    planet.map[p].event = Some(Box::new(event));
}
