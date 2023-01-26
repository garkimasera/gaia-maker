use super::*;
use geom::MDistRangeIter;

const FERTILITY_MAX: f32 = 100.0;
const FERTILITY_MIN: f32 = 0.0;

pub fn sim_biome(planet: &mut Planet, _sim: &mut Sim, params: &Params) {
    let map_iter_idx = planet.map.iter_idx();

    for p in map_iter_idx {
        if let Some(structure_param) = params.structures.get(&planet.map[p].structure.kind()) {
            if let Some(BuildingEffect::Fertilize {
                increment,
                max,
                range,
            }) = structure_param.building.effect
            {
                for (_, p) in MDistRangeIter::new(p, range as _) {
                    let fertility = &mut planet.map[p].fertility;
                    *fertility =
                        (*fertility + increment).clamp(FERTILITY_MIN, max.min(FERTILITY_MAX));
                }
            }
        }
    }
}
