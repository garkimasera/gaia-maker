use crate::action::CursorAction;
use crate::assets::UiAssets;
use crate::conf::Conf;
use crate::draw::UpdateMap;
use crate::ui::WindowsOpenState;
use crate::{planet::*, GameSpeed, GameState, GameSystemSet};
use bevy::window::{PrimaryWindow, WindowResized};
use bevy::{
    math::{Rect, Vec3Swizzles},
    prelude::*,
};
use geom::Coords;

#[derive(Clone, Copy, Debug)]
pub struct ScreenPlugin;

#[derive(Clone, Copy, Debug, Event)]
pub struct Centering(pub Vec2);

#[derive(Clone, Debug, Resource)]
pub enum CursorMode {
    Normal,
    Demolition,
    Build(StructureKind),
    TileEvent(TileEventKind),
    SpawnAnimal(AnimalId),
    EditBiome(Biome),
    ChangeHeight(f32),
    PlaceSettlement(AnimalId, CivilizationAge),
}

impl Default for CursorMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Centering>()
            .add_systems(Startup, setup)
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<InScreenTileRange>()
            .init_resource::<CursorMode>()
            .add_systems(
                OnEnter(GameState::Running),
                (setup_cursor, set_scale_factor_to_occupied_screen_space),
            )
            .add_systems(
                Update,
                (centering, on_resize)
                    .run_if(in_state(GameState::Running))
                    .before(GameSystemSet::Draw),
            )
            .add_systems(
                Update,
                update_hover_tile
                    .run_if(in_state(GameState::Running))
                    .in_set(GameSystemSet::UpdateHoverTile)
                    .after(crate::ui::UiWindowsSystemSet), // Because occupied_screen_space.rects is set
            )
            .add_systems(
                Update,
                mouse_event
                    .run_if(in_state(GameState::Running))
                    .after(GameSystemSet::UpdateHoverTile),
            )
            .add_systems(Update, keyboard_input.run_if(in_state(GameState::Running)))
            .add_systems(Update, crate::platform::window_resize);
    }
}

#[derive(Clone, Debug, Resource)]
pub struct InScreenTileRange {
    pub from: Coords,
    pub to: Coords,
    pub y_to_from_not_clamped: (i32, i32),
}

#[derive(Clone, Copy, Default, Debug, Component)]
pub struct HoverTile(pub Option<Coords>);

impl Default for InScreenTileRange {
    fn default() -> Self {
        Self {
            from: Coords(0, 0),
            to: Coords(0, 0),
            y_to_from_not_clamped: (0, 0),
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
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
        .spawn((
            Sprite::from_image(asset_server.get_handle("ui/tile-cursor.png").unwrap()),
            Visibility::Hidden,
        ))
        .insert(HoverTile(None));
    *setup = true;
}

fn set_scale_factor_to_occupied_screen_space(
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    conf: Res<Conf>,
) {
    occupied_screen_space.scale_factor = conf.ui.scale_factor;
}

fn mouse_event(
    mut ew_cursor_action: EventWriter<CursorAction>,
    mut ew_centering: EventWriter<Centering>,
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
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
        Vec2::new(pos.x, window.height() - pos.y)
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
                _drag: false,
            });
            *prev_tile_coords = Some(coords);
            return;
        }

        if prev_tile_coords.is_some() && Some(coords) != *prev_tile_coords {
            ew_cursor_action.send(CursorAction {
                coords,
                _drag: true,
            });
            *prev_tile_coords = Some(coords);
        }
    }
}

fn centering(
    mut er_centering: EventReader<Centering>,
    mut update_map: ResMut<UpdateMap>,
    screen: Res<OccupiedScreenSpace>,
    window: Query<(&Window, &bevy_egui::EguiContextSettings), With<PrimaryWindow>>,
    mut in_screen_tile_range: ResMut<InScreenTileRange>,
    planet: Res<Planet>,
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
) {
    let Ok((window, egui_settings)) = window.get_single() else {
        return;
    };

    for e in er_centering.read() {
        update_map.update();
        let transform = &mut camera_query.get_single_mut().unwrap().1.translation;

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
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor,
            0.0,
        ) / 2.0;
        *transform -= space_adjust;

        transform.x = transform.x.round();
        transform.y = transform.y.round();

        // Update in screen tile range
        let x0 = ((transform.x - window.width() / 2.0) / TILE_SIZE) as i32 - 1;
        let y0 = ((transform.y - window.height() / 2.0) / TILE_SIZE) as i32 - 1;
        let x1 = ((transform.x + window.width() / 2.0) / TILE_SIZE) as i32 + 1;
        let y1 = ((transform.y + window.height() / 2.0) / TILE_SIZE) as i32 + 1;
        in_screen_tile_range.y_to_from_not_clamped = (y0, y1);
        let y0 = y0.clamp(0, planet.map.size().1 as i32 - 1);
        let y1 = y1.clamp(0, planet.map.size().1 as i32 - 1);
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
    occupied_screen_space: Res<OccupiedScreenSpace>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    cursor_mode: Res<CursorMode>,
    ui_assets: Res<UiAssets>,
    mut color_entities: Local<Vec<Entity>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let cursor_pos = if let Some(pos) = window.cursor_position() {
        Vec2::new(pos.x, window.height() - pos.y)
    } else {
        return;
    };

    let mut hover_tile = hover_tile.get_single_mut().unwrap();

    // Check covered by ui or not
    if !occupied_screen_space.check(window.width(), window.height(), cursor_pos) {
        hover_tile.0 .0 = None;
        *hover_tile.2 = Visibility::Hidden;
        return;
    }

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
    if matches!(*cursor_mode, CursorMode::Normal) {
        return;
    }

    let id = commands
        .spawn((
            Sprite::from_image(ui_assets.tile_colored.clone()),
            Visibility::Inherited,
            Transform::from_xyz(
                tile_i as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                tile_j as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                920.0,
            ),
        ))
        .id();
    color_entities.push(id);
}

#[derive(Clone, Default, Debug, Resource)]
pub struct OccupiedScreenSpace {
    pub occupied_top: f32,
    pub occupied_buttom: f32,
    pub occupied_left: f32,
    pub occupied_right: f32,
    pub window_rects: Vec<Rect>,
    pub opening_modal: bool,
    scale_factor: f32,
}

impl OccupiedScreenSpace {
    fn check(&self, w: f32, h: f32, p: Vec2) -> bool {
        if self.opening_modal {
            return false;
        }

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
            if rect.contains(Vec2::new(x, y)) {
                return false;
            }
        }

        true
    }

    pub fn push_egui_window_rect(&mut self, rect: bevy_egui::egui::Rect) {
        let rect = Rect {
            min: Vec2::new(
                rect.left() * self.scale_factor,
                rect.top() * self.scale_factor,
            ),
            max: Vec2::new(
                rect.right() * self.scale_factor,
                rect.bottom() * self.scale_factor,
            ),
        };
        self.window_rects.push(rect);
    }

    pub fn reset(&mut self) {
        self.window_rects.clear();
        self.opening_modal = false;
    }
}

fn on_resize(
    mut er: EventReader<WindowResized>,
    mut ew_centering: EventWriter<Centering>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    screen: Res<OccupiedScreenSpace>,
    egui_settings: Query<&bevy_egui::EguiContextSettings, With<bevy::window::PrimaryWindow>>,
) {
    let egui_settings = egui_settings.single();

    for _ in er.read() {
        let transform = camera_query.get_single().unwrap().1;
        let mut translation = transform.translation.xy();

        let d = Vec2::new(
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor,
        ) / 2.0;
        translation += d;
        ew_centering.send(Centering(translation));
    }
}

fn keyboard_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut ew_centering: EventWriter<Centering>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    camera_query: Query<(&OrthographicProjection, &mut Transform)>,
    screen: Res<OccupiedScreenSpace>,
    egui_settings: Query<&bevy_egui::EguiContextSettings, With<bevy::window::PrimaryWindow>>,
    conf: Res<Conf>,
    mut old_gamespeed: Local<GameSpeed>,
) {
    let egui_settings = egui_settings.single();

    // Shortcut keys

    // Pause by space
    if keys.just_pressed(KeyCode::Space) {
        match *speed {
            GameSpeed::Paused => {
                if *old_gamespeed == GameSpeed::Paused {
                    *old_gamespeed = GameSpeed::Normal;
                }
                std::mem::swap(&mut *speed, &mut *old_gamespeed);
            }
            _ => {
                *old_gamespeed = *speed;
                *speed = GameSpeed::Paused;
            }
        }
    }
    // Debug by Alt+F12
    if keys.just_pressed(KeyCode::F12)
        && (keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight))
    {
        wos.debug_tools = !wos.debug_tools;
    }

    // Keys for moving camera
    let direction = match (
        keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW),
        keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA),
        keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS),
        keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD),
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
            (screen.occupied_left - screen.occupied_right) * egui_settings.scale_factor,
            (screen.occupied_buttom - screen.occupied_top) * egui_settings.scale_factor,
        ) / 2.0;
        let new_center = camera_pos + space_adjust + Vec2::new(dx, dy) * conf.camera_move_speed;
        ew_centering.send(Centering(new_center));
    }
}
