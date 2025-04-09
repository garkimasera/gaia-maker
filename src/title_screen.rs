use bevy::prelude::*;

use crate::{GameState, draw::UpdateDraw};

#[derive(Clone, Copy, Debug)]
pub struct TitleScreenPlugin;

impl Plugin for TitleScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateDraw>()
            .add_systems(Startup, setup_title_screen)
            .add_systems(OnEnter(GameState::MainMenu), enter_title_screen)
            .add_systems(OnExit(GameState::MainMenu), exit_title_screen)
            .add_systems(Update, on_resize.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
pub struct TitleScreenLogo;

#[derive(Component)]
pub struct TitleScreenBackground;

const LOGO_SIZE: (f32, f32) = (316.0, 250.0);
const BACKGROUND_SIZE: (f32, f32) = (1920.0, 896.0);

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
    let logo = asset_server.load("logo.webp");
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

    // Background
    let background = asset_server.load("title-screen.webp");
    let mesh_handle = meshes.add(Rectangle::from_size(Vec2::new(
        BACKGROUND_SIZE.0,
        BACKGROUND_SIZE.1,
    )));
    commands
        .spawn((
            Mesh2d(mesh_handle.clone()),
            MeshMaterial2d(materials.add(ColorMaterial {
                texture: Some(background),
                ..default()
            })),
            Transform::from_xyz(0.0, 0.0, 991.0),
        ))
        .insert(TitleScreen)
        .insert(TitleScreenBackground);
}

fn enter_title_screen(
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    mut meshes: Query<&mut Visibility, With<TitleScreen>>,
    mut bg_transform: Query<
        &mut Transform,
        (With<TitleScreenBackground>, Without<OrthographicProjection>),
    >,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let cpos = &mut camera_query.get_single_mut().unwrap().1.translation;
    cpos.x = 0.0;
    cpos.y = 0.0;
    crate::screen::adjust_camera_pos(&mut cpos.x, &mut cpos.y, window.width(), window.height());

    for mut bg in meshes.iter_mut() {
        *bg = Visibility::Visible;
    }

    let bg_scale = (window.width() / BACKGROUND_SIZE.0)
        .max(window.height() / BACKGROUND_SIZE.1)
        .max(1.0);
    let mut t = bg_transform.get_single_mut().unwrap();
    *t = t.with_scale(Vec3::new(bg_scale, bg_scale, 1.0));
}

fn exit_title_screen(mut meshes: Query<&mut Visibility, With<TitleScreen>>) {
    for mut bg in meshes.iter_mut() {
        *bg = Visibility::Hidden;
    }
}

fn on_resize(
    mut er: EventReader<bevy::window::WindowResized>,
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    mut bg_transform: Query<
        &mut Transform,
        (With<TitleScreenBackground>, Without<OrthographicProjection>),
    >,
) {
    let cpos = &mut camera_query.get_single_mut().unwrap().1.translation;
    if let Some(e) = er.read().last() {
        crate::screen::adjust_camera_pos(&mut cpos.x, &mut cpos.y, e.width, e.height);

        let bg_scale = (e.width / BACKGROUND_SIZE.0)
            .max(e.height / BACKGROUND_SIZE.1)
            .max(1.0);
        let mut t = bg_transform.get_single_mut().unwrap();
        *t = t.with_scale(Vec3::new(bg_scale, bg_scale, 1.0));
    }
}
