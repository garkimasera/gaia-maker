use bevy::prelude::*;
use bevy_kira_audio::AudioControl;
use geom::Coords;

use crate::assets::SoundEffects;
use crate::audio::{AudioSE, SoundEffect};
use crate::draw::UpdateMap;
use crate::planet::*;
use crate::screen::CursorMode;
use crate::GameState;

#[derive(Clone, Copy, Debug)]
pub struct ActionPlugin;

#[derive(Clone, Copy, Debug)]
pub struct CursorAction {
    pub coords: Coords,
    pub drag: bool,
}

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorAction>()
            .add_system(cursor_action.in_set(OnUpdate(GameState::Running)));
    }
}

fn cursor_action(
    mut er: EventReader<CursorAction>,
    mut update_map: ResMut<UpdateMap>,
    cursor_mode: Res<CursorMode>,
    params: Res<Params>,
    mut planet: ResMut<Planet>,
    audio_se: Res<AudioSE>,
    sound_effects: Res<SoundEffects>,
) {
    for e in er.iter() {
        let CursorAction { coords, .. } = *e;

        match *cursor_mode {
            CursorMode::Normal => (),
            CursorMode::Demolition => {
                update_map.update();
                planet.demolition(coords);
            }
            CursorMode::EditBiome(biome) => {
                update_map.update();
                planet.edit_biome(coords, biome);
            }
            CursorMode::Build(kind) => match kind {
                StructureKind::None => (),
                _ => {
                    if planet.buildable(params.structures[&kind].as_ref(), 1) {
                        update_map.update();
                        let size = params.structures[&kind].size;
                        if planet.placeable(coords, size) {
                            planet.place(coords, size, new_structure(kind), &params);
                            audio_se.play(sound_effects.get(SoundEffect::Build));
                        }
                    }
                }
            },
        }
    }
}

fn new_structure(kind: StructureKind) -> Structure {
    match kind {
        StructureKind::OxygenGenerator => Structure::OxygenGenerator {
            state: StructureBuildingState::Working,
        },
        StructureKind::NitrogenSprayer => Structure::NitrogenSprayer {
            state: StructureBuildingState::Working,
        },
        StructureKind::CarbonDioxideSprayer => Structure::CarbonDioxideSprayer {
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
