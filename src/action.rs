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
                if planet.demolition(coords, &mut sim, &params) {
                    audio_player.play_se("demolish");
                }
            }
            CursorMode::Build(kind) => {
                if planet.buildable(params.structures[&kind].as_ref()) {
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
            CursorMode::SpawnAnimal(animal_id) => {
                if planet.animal_spawnable(coords, animal_id, &params) {
                    update_map.update();
                    planet.spawn_animal(coords, animal_id, &params);
                }
            }
            CursorMode::EditBiome(biome) => {
                update_map.update();
                planet.edit_biome(coords, biome);
            }
            CursorMode::PlaceSettlement(id, age) => {
                update_map.update();
                planet.place_settlement(
                    coords,
                    Settlement {
                        id,
                        age,
                        pop: params.sim.settlement_init_pop[age as usize],
                        tech_exp: 0.0,
                    },
                );
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
        StructureKind::GiftTower => Structure::GiftTower,
        _ => unreachable!(),
    }
}

pub fn cursor_mode_lack_and_cost(
    planet: &Planet,
    params: &Params,
    cursor_mode: &CursorMode,
) -> Vec<(bool, Cost)> {
    let mut cost_list = Vec::new();

    match cursor_mode {
        CursorMode::Build(kind) => {
            if let Some(attr) = params.structures.get(kind).map(|a| &a.building) {
                if attr.energy < 0.0 {
                    cost_list.push((
                        -attr.energy > planet.res.surplus_energy(),
                        Cost::Energy(-attr.energy, 0),
                    ));
                }
                if attr.cost > 0.0 {
                    cost_list.push((attr.cost > planet.res.material, Cost::Material(attr.cost)));
                }
            }
        }
        CursorMode::SpawnAnimal(animal_id) => {
            let attr = &params.animals[animal_id];
            cost_list.push((
                attr.cost > planet.res.gene_point,
                Cost::GenePoint(attr.cost),
            ));
        }
        CursorMode::TileEvent(kind) => {
            if let Some(cost) = params.event.tile_event_costs.get(kind) {
                cost_list.push((!planet.res.enough_to_consume(*cost), *cost));
            }
        }
        _ => (),
    }
    cost_list
}
