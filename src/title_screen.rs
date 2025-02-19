use bevy::prelude::*;
use rand::{seq::IndexedRandom, Rng, SeedableRng};

use crate::{draw::UpdateDraw, GameState};

#[derive(Clone, Copy, Debug)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateDraw>()
            .add_systems(Startup, setup_title_screen)
            .add_systems(OnEnter(GameState::MainMenu), enter_title_screen)
            .add_systems(OnExit(GameState::MainMenu), exit_title_screen);
    }
}

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
pub struct TitleScreenLogo;

const N_STARS: usize = 2000;
const LOGO_SIZE: (f32, f32) = (316.0, 250.0);

fn setup_title_screen(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Background
    let bg_material = materials.add(ColorMaterial {
        color: Srgba {
            red: 0.05,
            green: 0.05,
            blue: 0.05,
            alpha: 1.0,
        }
        .into(),
        ..default()
    });
    let bg_mesh = meshes.add(Mesh::from(Rectangle::new(100000.0, 100000.0)));
    commands
        .spawn((
            Mesh2d(bg_mesh),
            MeshMaterial2d(bg_material),
            Transform::from_xyz(0.0, 0.0, 990.0),
        ))
        .insert(TitleScreen);

    // Logo
    let logo = asset_server.load("logo.png");
    let mesh_handle = meshes.add(Rectangle::from_size(Vec2::new(LOGO_SIZE.0, LOGO_SIZE.1)));
    commands
        .spawn((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(materials.add(ColorMaterial {
                texture: Some(logo),
                ..default()
            })),
            Transform::from_xyz(0.0, LOGO_SIZE.1 / 2.0, 992.0),
        ))
        .insert(TitleScreen)
        .insert(TitleScreenLogo);

    // Stars
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0x47616961);
    let colors: &[[u8; 3]] = &[
        [0xFE, 0xFE, 0xFE],
        [0xFE, 0xF3, 0xEA],
        [0xE4, 0xE4, 0xE0],
        [0xEE, 0xEC, 0xFC],
    ];
    let materials: Vec<_> = colors
        .iter()
        .map(|color| {
            materials.add(ColorMaterial {
                color: Srgba {
                    red: color[0] as f32 / 255.0,
                    green: color[1] as f32 / 255.0,
                    blue: color[2] as f32 / 255.0,
                    alpha: 1.0,
                }
                .into(),
                ..default()
            })
        })
        .collect();
    let mesh = meshes.add(Mesh::from(Rectangle::new(2.0, 2.0)));

    for _ in 0..N_STARS {
        let x = rng.random_range(-2000.0..2000.0);
        let y = rng.random_range(-2000.0..2000.0);
        commands
            .spawn((
                Mesh2d(mesh.clone()),
                MeshMaterial2d(materials.choose(&mut rng).cloned().unwrap()),
                Transform::from_xyz(x, y, 991.0),
            ))
            .insert(TitleScreen);
    }
}

fn enter_title_screen(
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    mut meshes: Query<&mut Visibility, With<TitleScreen>>,
) {
    let translation = &mut camera_query.get_single_mut().unwrap().1.translation;
    translation.x = 0.0;
    translation.y = 0.0;

    for mut bg in meshes.iter_mut() {
        *bg = Visibility::Visible;
    }
}

fn exit_title_screen(mut meshes: Query<&mut Visibility, With<TitleScreen>>) {
    for mut bg in meshes.iter_mut() {
        *bg = Visibility::Hidden;
    }
}
