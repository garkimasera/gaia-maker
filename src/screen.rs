use crate::action::CursorAction;
use crate::assets::{UiTexture, UiTextures};
use crate::conf::Conf;
use crate::draw::UpdateMap;
use crate::ui::WindowsOpenState;
use crate::{planet::*, GameState, GameSystemSet};
use bevy::sprite::MaterialMesh2dBundle;
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::{
    math::{Rect, Vec3Swizzles},
    prelude::*,
};
use geom::Coords;

#[derive(Clone, Copy, Debug)]
pub struct ScreenPlugin;

#[derive(Clone, Copy, Debug)]
pub struct Centering(pub Vec2);

#[derive(Clone, Debug, Resource)]
pub enum CursorMode {
    Normal,
    Demolition,
    Build(StructureKind),
    EditBiome(Biome),
}

impl Default for CursorMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Centering>()
            .add_startup_system(setup)
            .add_startup_system(setup_main_menu_background)
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<InScreenTileRange>()
            .init_resource::<CursorMode>()
            .add_system(main_menu_background.in_schedule(OnEnter(GameState::MainMenu)))
            .add_system(main_menu_background_exit.in_schedule(OnExit(GameState::MainMenu)))
            .add_system(setup_cursor.in_schedule(OnEnter(GameState::Running)))
            .add_system(
                on_enter_running
                    .in_set(OnUpdate(GameState::Running))
                    .after(GameSystemSet::StartSim),
            )
            .add_systems(
                (centering, on_resize)
                    .in_set(OnUpdate(GameState::Running))
                    .before(GameSystemSet::Draw),
            )
            .add_system(
                update_hover_tile
                    .in_set(OnUpdate(GameState::Running))
                    .in_set(GameSystemSet::UpdateHoverTile),
            )
            .add_system(
                mouse_event
                    .in_set(OnUpdate(GameState::Running))
                    .after(GameSystemSet::UpdateHoverTile),
            )
            .add_system(keyboard_input.in_set(OnUpdate(GameState::Running)))
            .add_system(window_resize);
    }
}

#[derive(Clone, Debug, Resource)]
pub struct InScreenTileRange {
    pub from: Coords,
    pub to: Coords,
}

#[derive(Clone, Copy, Default, Debug, Component)]
pub struct HoverTile(pub Option<Coords>);

impl Default for InScreenTileRange {
    fn default() -> Self {
        Self {
            from: Coords(0, 0),
            to: Coords(0, 0),
        }
    }
}

pub fn setup(mut commands: Commands) {
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
}

pub fn setup_cursor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut setup: Local<bool>,
) {
    if *setup {
        return;
    }
    commands
        .spawn(SpriteBundle {
            texture: asset_server.get_handle("ui/tile-cursor.png"),
            visibility: Visibility::Hidden,
            ..default()
        })
        .insert(HoverTile(None));
    *setup = true;
}

fn on_enter_running(
    planet: Option<Res<Planet>>,
    mut ew_centering: EventWriter<Centering>,
    mut done: Local<bool>,
) {
    let Some(planet) = planet else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    let h = planet.map.size().1;
    ew_centering.send(Centering(Vec2 {
        x: 0.0,
        y: h as f32 * TILE_SIZE / 2.0,
    }));
}

fn mouse_event(
    mut ew_cursor_action: EventWriter<CursorAction>,
    mut ew_centering: EventWriter<Centering>,
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    hover_tile: Query<(&HoverTile, &Transform), Without<OrthographicProjection>>,
    mut cursor_mode: ResMut<CursorMode>,
    mut prev_tile_coords: Local<Option<Coords>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    // Clear current selected tool
    if mouse_button_input.just_pressed(MouseButton::Right) {
        *cursor_mode = CursorMode::Normal;
    }

    // Check covered by ui or not
    if !occupied_screen_space.check(window.width(), window.height(), pos) {
        if mouse_button_input.just_pressed(MouseButton::Left)
            && !matches!(*cursor_mode, CursorMode::EditBiome(_))
        {
            *cursor_mode = CursorMode::Normal;
        }
        return;
    }

    // Centering
    if mouse_button_input.just_pressed(MouseButton::Middle) {
        let transform = camera_query.get_single().unwrap().1;
        let mut translation = transform.translation.xy();

        let d = Vec2::new(pos.x - window.width() / 2.0, pos.y - window.height() / 2.0);

        translation += d;

        ew_centering.send(Centering(translation));
        return;
    }

    // Cursor action
    if !mouse_button_input.pressed(MouseButton::Left) {
        *prev_tile_coords = None;
        return;
    }

    if let Some(coords) = hover_tile.get_single().unwrap().0 .0 {
        if mouse_button_input.just_pressed(MouseButton::Left) {
            ew_cursor_action.send(CursorAction {
                coords,
                drag: false,
            });
            *prev_tile_coords = Some(coords);
            return;
        }

        if prev_tile_coords.is_some() && Some(coords) != *prev_tile_coords {
            ew_cursor_action.send(CursorAction { coords, drag: true });
            *prev_tile_coords = Some(coords);
        }
    }
}

fn centering(
    mut er_centering: EventReader<Centering>,
    mut update_map: ResMut<UpdateMap>,
    screen: Res<OccupiedScreenSpace>,
    window: Query<&Window, With<PrimaryWindow>>,
    egui_settings: ResMut<bevy_egui::EguiSettings>,
    mut in_screen_tile_range: ResMut<InScreenTileRange>,
    planet: Res<Planet>,
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
) {
    for e in er_centering.iter() {
        update_map.update();
        let transform = &mut camera_query.get_single_mut().unwrap().1.translation;
        let Ok(window) = window.get_single() else {
            return;
        };

        let center = &e.0;

        // Change camera position
        let width = TILE_SIZE * planet.map.size().0 as f32;
        let x = center.x;
        transform.x = if x < 0.0 {
            x + ((-x / width).trunc() + 1.0) * width
        } else {
            x - (x / width).trunc() * width
        };
        transform.y = center
            .y
            .clamp(-TILE_SIZE, (planet.map.size().1 + 1) as f32 * TILE_SIZE);

        let space_adjust = Vec3::new(
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor as f32,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor as f32,
            0.0,
        ) / 2.0;
        *transform -= space_adjust;

        transform.x = transform.x.round();
        transform.y = transform.y.round();

        // Update in screnn tile range
        let x0 = ((transform.x - window.width() / 2.0) / TILE_SIZE) as i32 - 1;
        let y0 = (((transform.y - window.height() / 2.0) / TILE_SIZE) as i32 - 1)
            .clamp(0, planet.map.size().1 as i32 - 1);
        let x1 = ((transform.x + window.width() / 2.0) / TILE_SIZE) as i32 + 1;
        let y1 = (((transform.y + window.height() / 2.0) / TILE_SIZE) as i32 + 1)
            .clamp(0, planet.map.size().1 as i32 - 1);
        in_screen_tile_range.from = Coords(x0, y0);
        in_screen_tile_range.to = Coords(x1, y1);
    }
}

fn update_hover_tile(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    planet: Res<Planet>,
    mut hover_tile: Query<
        (&mut HoverTile, &mut Transform, &mut Visibility),
        Without<OrthographicProjection>,
    >,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    cursor_mode: Res<CursorMode>,
    ui_textures: Res<UiTextures>,
    params: Res<Params>,
    mut color_entities: Local<Vec<Entity>>,
) {
    let mut hover_tile = hover_tile.get_single_mut().unwrap();
    let Ok(window) = window.get_single() else {
            return;
        };
    let cursor_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };

    let camera_pos = camera_query.get_single().unwrap().1.translation.xy();

    let p = cursor_pos + camera_pos - Vec2::new(window.width() / 2.0, window.height() / 2.0);

    let tile_i = if p.x >= 0.0 {
        (p.x / TILE_SIZE) as i32
    } else {
        (p.x / TILE_SIZE) as i32 - 1
    };
    let tile_j = (p.y / TILE_SIZE) as i32;

    *hover_tile.2 = if tile_j >= 0 && tile_j < planet.map.size().1 as i32 && p.y >= 0.0 {
        let planet_w = planet.map.size().0 as i32;
        let tile_i_rotated = if tile_i < 0 {
            tile_i + (-tile_i / planet_w + 1) * planet_w
        } else {
            tile_i - (tile_i / planet_w) * planet_w
        };
        hover_tile.0 .0 = Some(Coords(tile_i_rotated, tile_j));
        hover_tile.1.translation.x = tile_i as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        hover_tile.1.translation.y = tile_j as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        hover_tile.1.translation.z = 950.0;
        Visibility::Inherited
    } else {
        hover_tile.0 .0 = None;
        Visibility::Hidden
    };

    for entity in color_entities.iter() {
        commands.entity(*entity).despawn();
    }
    color_entities.clear();

    if *hover_tile.2 == Visibility::Hidden {
        return;
    }

    let size = match &*cursor_mode {
        CursorMode::Demolition => StructureSize::Small,
        CursorMode::Build(kind) => params.structures[kind].size,
        CursorMode::EditBiome(_) => StructureSize::Small,
        _ => {
            return;
        }
    };

    for p in [Coords(tile_i, tile_j)]
        .into_iter()
        .chain(size.occupied_tiles().iter().map(|p| *p + (tile_i, tile_j)))
    {
        let mut transform = Transform { ..default() };
        transform.translation.x = p.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        transform.translation.y = p.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        transform.translation.z = 920.0;

        let id = commands
            .spawn(SpriteBundle {
                texture: ui_textures.get(UiTexture::TileColored),
                visibility: Visibility::Inherited,
                transform,
                ..default()
            })
            .id();
        color_entities.push(id);
    }
}

#[derive(Clone, Default, Debug, Resource)]
pub struct OccupiedScreenSpace {
    pub occupied_top: f32,
    pub occupied_buttom: f32,
    pub occupied_left: f32,
    pub occupied_right: f32,
    pub window_rects: Vec<Rect>,
}

impl OccupiedScreenSpace {
    fn check(&self, w: f32, h: f32, p: Vec2) -> bool {
        if p.x < self.occupied_left
            || p.x > w - self.occupied_right
            || p.y < self.occupied_buttom
            || p.y > h - self.occupied_top
        {
            return false;
        }

        let x = p.x;
        let y = h - p.y;

        for rect in &self.window_rects {
            if rect.min.x <= x && x <= rect.max.x && rect.min.y <= y && y <= rect.max.y {
                return false;
            }
        }

        true
    }
}

fn on_resize(
    mut er: EventReader<WindowResized>,
    mut ew_centering: EventWriter<Centering>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    screen: Res<OccupiedScreenSpace>,
    egui_settings: ResMut<bevy_egui::EguiSettings>,
) {
    for _ in er.iter() {
        let transform = camera_query.get_single().unwrap().1;
        let mut translation = transform.translation.xy();

        let d = Vec2::new(
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor as f32,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor as f32,
        ) / 2.0;
        translation += d;
        ew_centering.send(Centering(translation));
    }
}

fn keyboard_input(
    keys: Res<Input<KeyCode>>,
    mut ew_centering: EventWriter<Centering>,
    mut wos: ResMut<WindowsOpenState>,
    camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    screen: Res<OccupiedScreenSpace>,
    egui_settings: ResMut<bevy_egui::EguiSettings>,
    conf: Res<Conf>,
) {
    // Shortcut keys
    if keys.just_pressed(KeyCode::F12)
        && (keys.pressed(KeyCode::LAlt) || keys.pressed(KeyCode::RAlt))
    {
        wos.debug_tools = !wos.debug_tools;
    }

    // Keys for moving camera
    let direction = match (
        keys.pressed(KeyCode::Up) || keys.pressed(KeyCode::W),
        keys.pressed(KeyCode::Left) || keys.pressed(KeyCode::A),
        keys.pressed(KeyCode::Down) || keys.pressed(KeyCode::S),
        keys.pressed(KeyCode::Right) || keys.pressed(KeyCode::D),
    ) {
        (true, false, false, false) => Some((0.0, 1.0)),
        (true, true, false, false) => Some((-1.0, 1.0)),
        (false, true, false, false) => Some((-1.0, 0.0)),
        (false, true, true, false) => Some((-1.0, -1.0)),
        (false, false, true, false) => Some((0.0, -1.0)),
        (false, false, true, true) => Some((1.0, -1.0)),
        (false, false, false, true) => Some((1.0, 0.0)),
        (true, false, false, true) => Some((1.0, 1.0)),
        _ => None,
    };

    if let Some((dx, dy)) = direction {
        let camera_pos = camera_query.get_single().unwrap().1.translation.xy();

        let space_adjust = Vec2::new(
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor as f32,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor as f32,
        ) / 2.0;
        let new_center = camera_pos + space_adjust + Vec2::new(dx, dy) * conf.camera_move_speed;
        ew_centering.send(Centering(new_center));
    }
}

#[derive(Component)]
struct MainMenuBackground;

fn setup_main_menu_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bg_material = materials.add(ColorMaterial {
        color: Color::GRAY,
        texture: None,
    });
    let bg_mesh = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(100000.0, 100000.0))));
    commands
        .spawn(MaterialMesh2dBundle {
            mesh: bg_mesh.into(),
            transform: Transform::from_xyz(0.0, 0.0, 998.0),
            material: bg_material,
            ..default()
        })
        .insert(MainMenuBackground);
}

fn main_menu_background(
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    mut bg_meshes: Query<&mut Visibility, With<MainMenuBackground>>,
) {
    let translation = &mut camera_query.get_single_mut().unwrap().1.translation;
    translation.x = 0.0;
    translation.y = 0.0;

    for mut bg in bg_meshes.iter_mut() {
        *bg = Visibility::Visible;
    }
}

fn main_menu_background_exit(mut bg_meshes: Query<&mut Visibility, With<MainMenuBackground>>) {
    for mut bg in bg_meshes.iter_mut() {
        *bg = Visibility::Hidden;
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn window_resize() {}

#[cfg(target_arch = "wasm32")]
fn window_resize(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.get_single_mut() else {
            return;
        };

    let Some(w) = web_sys::window() else {
        return;
    };

    let Some(width) = w
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64())
        .map(|width| width as f32) else {
            return;
        };
    let Some(height) = w
        .inner_height()
        .ok()
        .and_then(|height| height.as_f64())
        .map(|height| height as f32) else {
            return;
        };

    if window.width() != width as f32 || window.height() != height as f32 {
        window.resolution.set(width, height);
    }
}

const DEFAULT_WINDOW_SIZE: (u32, u32) = (1280, 720);

#[cfg(not(target_arch = "wasm32"))]
pub fn preferred_window_size() -> (u32, u32) {
    DEFAULT_WINDOW_SIZE
}

#[cfg(target_arch = "wasm32")]
pub fn preferred_window_size() -> (u32, u32) {
    let Some(w) = web_sys::window() else {
        return DEFAULT_WINDOW_SIZE;
    };

    let Some(width) = w
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64()) else {
            return DEFAULT_WINDOW_SIZE;
        };
    let Some(height) = w
        .inner_height()
        .ok()
        .and_then(|height| height.as_f64()) else {
            return DEFAULT_WINDOW_SIZE;
        };
    let width = width as u32;
    let height = height as u32;

    (width, height)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn window_open() {}

#[cfg(target_arch = "wasm32")]
pub fn window_open() {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(game_screen) = document.get_element_by_id("game-screen") else {
        return;
    };
    let Some(game_screen) = game_screen.dyn_ref::<web_sys::HtmlElement>() else {
        return;
    };
    if let Err(e) = game_screen.style().set_property("display", "block") {
        log::warn!("{:?}", e);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn window_close() {}

#[cfg(target_arch = "wasm32")]
pub fn window_close() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(location) = document.location() else {
        return;
    };
    let _ = location.reload();
}
