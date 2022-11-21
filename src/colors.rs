use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ColorsPlugin;

impl Plugin for ColorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(prepare_color_materials);
    }
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
