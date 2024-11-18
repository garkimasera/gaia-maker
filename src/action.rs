use bevy::prelude::*;
use geom::Coords;

use crate::audio::AudioPlayer;
use crate::draw::UpdateMap;
use crate::planet::*;
use crate::screen::CursorMode;
use crate::GameState;

#[derive(Clone, Copy, Debug)]
pub struct ActionPlugin;

#[derive(Clone, Copy, Debug, Event)]
pub struct CursorAction {
    pub coords: Coords,
    pub _drag: bool,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorAction>()
            .add_systems(Update, cursor_action.run_if(in_state(GameState::Running)));
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
            CursorMode::Build(kind) => match kind {
                StructureKind::None => (),
                _ => {
                    if planet.buildable(params.structures[&kind].as_ref(), 1) {
                        update_map.update();
                        let size = params.structures[&kind].size;
                        if planet.placeable(coords, size) {
                            planet.place(coords, size, new_structure(kind), &mut sim, &params);
                            audio_player.play_se("build");
                        }
                    }
                }
            },
            CursorMode::EditBiome(biome) => {
                update_map.update();
                planet.edit_biome(coords, biome);
            }
            CursorMode::PlaceSettlement(settlement) => {
                update_map.update();
                planet.place_settlement(coords, settlement);
            }
            CursorMode::SpawnAnimal(ref _animal_id) => {
                todo!()
            }
        }
    }
}

fn new_structure(kind: StructureKind) -> Structure {
    match kind {
        StructureKind::OxygenGenerator => Structure::OxygenGenerator {
            state: StructureBuildingState::Working,
        },
        StructureKind::Rainmaker => Structure::Rainmaker {
            state: StructureBuildingState::Working,
        },
        StructureKind::FertilizationPlant => Structure::FertilizationPlant {
            state: StructureBuildingState::Working,
        },
        StructureKind::Heater => Structure::Heater {
            state: StructureBuildingState::Working,
        },
        _ => unreachable!(),
    }
}
