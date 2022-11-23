use bevy::prelude::*;
use strum::{AsRefStr, EnumIter};

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

#[derive(Resource)]
pub struct ColorMaterials {
    pub green: Handle<ColorMaterial>,
}

fn prepare_color_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let color_materials = ColorMaterials {
        green: materials.add(ColorMaterial {
            color: Color::Rgba {
                red: 0.0,
                green: 1.0,
                blue: 0.0,
                alpha: 0.3,
            },
            texture: None,
        }),
    };
    commands.insert_resource(color_materials);
}
