use crate::action::CursorAction;
use crate::assets::{UiTexture, UiTextures};
use crate::draw::UpdateMap;
use crate::planet::*;
use crate::GameState;
use bevy::window::WindowResized;
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
    EditBiome(Biome),
    Build(StructureKind),
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
            .add_system_set(SystemSet::on_enter(GameState::Running).with_system(setup_cursor))
            .init_resource::<OccupiedScreenSpace>()
            .init_resource::<InScreenTileRange>()
            .init_resource::<CursorMode>()
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(on_enter_running.after("start_sim")),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Running)
                    .with_system(centering.before("draw"))
                    .with_system(on_resize.before("draw"))
                    .with_system(update_hover_tile.label("update_hover_tile"))
                    .with_system(mouse_event.after("update_hover_tile")),
            );
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

pub fn setup_cursor(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.get_handle("ui/tile-cursor.png"),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(HoverTile(None));
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
    windows: Res<Windows>,
    mouse_button_input: Res<Input<MouseButton>>,
    camera_query: Query<(&OrthographicProjection, &Transform)>,
    occupied_screen_space: Res<OccupiedScreenSpace>,
    hover_tile: Query<(&HoverTile, &Transform), Without<OrthographicProjection>>,
    mut cursor_mode: ResMut<CursorMode>,
    mut prev_tile_coords: Local<Option<Coords>>,
) {
    let window = windows.get_primary().unwrap();
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
    windows: Res<Windows>,
    egui_settings: ResMut<bevy_egui::EguiSettings>,
    mut in_screen_tile_range: ResMut<InScreenTileRange>,
    planet: Res<Planet>,
    mut camera_query: Query<(&OrthographicProjection, &mut Transform)>,
) {
    for e in er_centering.iter() {
        update_map.update();
        let transform = &mut camera_query.get_single_mut().unwrap().1.translation;
        let window = windows.get_primary().unwrap();

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
    windows: Res<Windows>,
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
    let window = windows.get_primary().unwrap();
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

    let is_visible = if tile_j >= 0 && tile_j < planet.map.size().1 as i32 && p.y >= 0.0 {
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
        true
    } else {
        hover_tile.0 .0 = None;
        false
    };
    *hover_tile.2 = Visibility { is_visible };

    for entity in color_entities.iter() {
        commands.entity(*entity).despawn();
    }
    color_entities.clear();

    if !is_visible {
        return;
    }

    let size = match &*cursor_mode {
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
                visibility: Visibility { is_visible: true },
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
