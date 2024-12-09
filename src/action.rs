use bevy::prelude::*;
use geom::Coords;

use crate::audio::AudioPlayer;
use crate::draw::UpdateMap;
use crate::planet::*;
use crate::screen::CursorMode;
use crate::{GameState, GameSystemSet};

#[derive(Clone, Copy, Debug)]
pub struct ActionPlugin;

#[derive(Clone, Copy, Debug, Event)]
pub struct CursorAction {
    pub coords: Coords,
    pub _drag: bool,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorAction>().add_systems(
            Update,
            cursor_action
                .run_if(in_state(GameState::Running))
                .before(GameSystemSet::Draw),
        );
    }
}

fn cursor_action(
    mut er: EventReader<CursorAction>,
    mut update_map: ResMut<UpdateMap>,
    cursor_mode: Res<CursorMode>,
    mut sim: ResMut<Sim>,
    params: Res<Params>,
    mut planet: ResMut<Planet>,
    audio_player: AudioPlayer,
) {
    for e in er.read() {
        let CursorAction { coords, .. } = *e;

        match *cursor_mode {
            CursorMode::Normal => (),
            CursorMode::Demolition => {
                update_map.update();
                planet.demolition(coords, &mut sim, &params);
            }
            CursorMode::Build(kind) => {
                if planet.buildable(params.structures[&kind].as_ref(), 1) {
                    update_map.update();
                    if planet.placeable(coords) {
                        planet.place(coords, new_structure(kind), &mut sim, &params);
                        audio_player.play_se("build");
                    }
                }
            }
            CursorMode::TileEvent(kind) => {
                planet.cause_tile_event(coords, kind, &mut sim, &params);
                update_map.update();
            }
            CursorMode::SpawnAnimal(ref animal_id) => {
                if planet.animal_spawnable(coords, animal_id, &params) {
                    update_map.update();
                    planet.spawn_animal(coords, animal_id, &params);
                }
            }
            CursorMode::EditBiome(biome) => {
                update_map.update();
                planet.edit_biome(coords, biome);
            }
            CursorMode::PlaceSettlement(settlement) => {
                update_map.update();
                planet.place_settlement(coords, settlement);
            }
        }
    }
}

fn new_structure(kind: StructureKind) -> Structure {
    match kind {
        StructureKind::OxygenGenerator => Structure::OxygenGenerator,
        StructureKind::Rainmaker => Structure::Rainmaker,
        StructureKind::FertilizationPlant => Structure::FertilizationPlant,
        StructureKind::Heater => Structure::Heater,
        StructureKind::CarbonCapturer => Structure::CarbonCapturer,
        _ => unreachable!(),
    }
}

pub fn cursor_mode_warn(
    planet: &Planet,
    params: &Params,
    cursor_mode: &CursorMode,
) -> Option<String> {
    match cursor_mode {
        CursorMode::Build(kind) => {
            if let Some(attr) = params.structures.get(kind).map(|a| &a.building) {
                if attr.cost > planet.res.material {
                    t!("msg/lack-of-material").into()
                } else if attr.energy < 0.0 && -attr.energy > planet.res.surplus_energy() {
                    t!("msg/lack-of-energy").into()
                } else {
                    None
                }
            } else {
                None
            }
        }
        CursorMode::SpawnAnimal(ref animal_id) => {
            let attr = &params.animals[animal_id];
            if attr.cost <= planet.res.gene_point {
                None
            } else {
                t!("msg/lack-of-gene-points").into()
            }
        }
        _ => None,
    }
}
