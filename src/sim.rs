use anyhow::Result;
use bevy::prelude::*;
use bevy::time::FixedTimestep;

use crate::draw::UpdateMap;
use crate::screen::Centering;
use crate::{planet::*, GameState};

#[derive(Clone, Copy, Debug)]
pub struct SimPlugin;

#[derive(Clone, Debug)]
pub enum ManagePlanet {
    New(u32, u32),
    Save(String),
    Load(String),
}

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ManagePlanet>()
            .add_system_set(
                SystemSet::on_enter(GameState::Running)
                    .with_system(start_sim)
                    .label("start_sim"),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_run_criteria(FixedTimestep::step(2.0))
                    .with_system(update),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(manage_planet)
                    .before("draw"),
            );
    }
}

fn start_sim(mut commands: Commands, mut update_map: ResMut<UpdateMap>, params: Res<Params>) {
    let planet = Planet::new(
        params.start.default_size.0,
        params.start.default_size.1,
        &params,
    );
    commands.insert_resource(planet);
    update_map.update();
}

fn update(mut planet: ResMut<Planet>, params: Res<Params>, mut update_map: ResMut<UpdateMap>) {
    update_map.update();
    planet.advance(&params);
}

fn manage_planet(
    mut er_manage_planet: EventReader<ManagePlanet>,
    mut planet: ResMut<Planet>,
    mut ew_centering: EventWriter<Centering>,
    params: Res<Params>,
) {
    for e in er_manage_planet.iter() {
        match e {
            ManagePlanet::New(w, h) => {
                *planet = Planet::new(*w, *h, &params);
                ew_centering.send(Centering(Vec2::new(
                    *w as f32 * TILE_SIZE / 2.0,
                    *h as f32 * TILE_SIZE / 2.0,
                )));
            }
            ManagePlanet::Save(path) => {
                if let Err(e) = save(&planet, path) {
                    log::warn!("cannot save: {:?}", e);
                }
            }
            ManagePlanet::Load(path) => match load(path) {
                Ok(new_planet) => {
                    *planet = new_planet;
                    ew_centering.send(Centering(Vec2::new(
                        planet.map.size().0 as f32 * TILE_SIZE / 2.0,
                        planet.map.size().1 as f32 * TILE_SIZE / 2.0,
                    )));
                }
                Err(e) => {
                    log::warn!("cannot load: {:?}", e);
                }
            },
        }
    }
}

fn save(planet: &Planet, path: &str) -> Result<()> {
    let w = std::fs::File::create(path)?;
    bincode::serialize_into(w, planet)?;
    Ok(())
}

fn load(path: &str) -> Result<Planet> {
    let r = std::fs::File::open(path)?;
    Ok(bincode::deserialize_from(r)?)
}
