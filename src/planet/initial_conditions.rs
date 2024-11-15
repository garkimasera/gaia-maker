use rand::Rng;

use super::{
    misc::{get_rng, SymmetricalLinearDist},
    Biome, InitialCondition, Params, Planet, Sim,
};

pub fn apply_initial_condition(
    planet: &mut Planet,
    _sim: &mut Sim,
    initial_condition: InitialCondition,
    _params: &Params,
) {
    let mut rng = get_rng();

    match initial_condition {
        InitialCondition::Snowball { thickness } => {
            for p in planet.map.iter_idx() {
                let t = 250.0;
                let tile = &mut planet.map[p];
                if tile.biome.is_land() {
                    tile.biome = Biome::IceField;
                } else {
                    tile.biome = Biome::SeaIce;
                    tile.sea_temp = t;
                }
                tile.ice = rng.sample(SymmetricalLinearDist::from(thickness));
                tile.temp = t;
                tile.vapor = 0.0;
            }
        }
    }
}
