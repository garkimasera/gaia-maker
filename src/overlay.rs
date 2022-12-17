use bevy::prelude::*;
use geom::Coords;
use strum::{AsRefStr, EnumIter};

use crate::planet::Planet;

#[derive(Clone, Copy, Debug)]
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(prepare_color_materials);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsRefStr, EnumIter, Resource)]
#[strum(serialize_all = "kebab-case")]
pub enum OverlayLayerKind {
    #[default]
    None,
    AirTemprature,
}

pub const N_POINTS: usize = 64;

#[derive(Resource)]
pub struct ColorMaterials {
    pub white_yellow_red: Vec<Handle<ColorMaterial>>,
}

impl ColorMaterials {
    pub fn get(
        &self,
        planet: &Planet,
        p: Coords,
        _kind: OverlayLayerKind,
    ) -> Handle<ColorMaterial> {
        let temp = planet.map[p].temp;

        let i = if temp < 263.15 {
            0
        } else {
            (((temp - 263.15) / (50.0 / N_POINTS as f32)) as usize).clamp(0, N_POINTS - 1)
        };

        self.white_yellow_red[i].clone()
    }
}

fn prepare_color_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let white_yellow_red = (0..N_POINTS)
        .map(|i| {
            let color = if i < N_POINTS / 2 {
                Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0 - (i as f32 / (N_POINTS / 2) as f32),
                    alpha: 0.4,
                }
            } else {
                Color::Rgba {
                    red: 1.0,
                    green: 1.0 - ((i - N_POINTS / 2) as f32 / (N_POINTS / 2) as f32),
                    blue: 0.0,
                    alpha: 0.4,
                }
            };

            materials.add(ColorMaterial {
                color,
                texture: None,
            })
        })
        .collect::<Vec<_>>();

    let color_materials = ColorMaterials { white_yellow_red };
    commands.insert_resource(color_materials);
}
