use crate::overlay::{ColorMaterials, OverlayLayerKind};
use crate::screen::InScreenTileRange;
use crate::{GameState, assets::*};
use crate::{GameSystemSet, planet::*};
use arrayvec::ArrayVec;
use bevy::prelude::*;
use geom::{Array2d, Coords, Direction, RectIter};

#[derive(Clone, Copy, Debug)]
pub struct DrawPlugin;

#[derive(Clone, Copy, Default, Debug, Resource, Event)]
pub struct UpdateDraw {
    need_update: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Resource)]
pub struct DisplayOpts {
    pub animals: bool,
    pub cities: bool,
    pub city_icons: bool,
    pub structures: bool,
}

impl Default for DisplayOpts {
    fn default() -> Self {
        Self {
            animals: true,
            cities: true,
            city_icons: false,
            structures: true,
        }
    }
}

impl UpdateDraw {
    pub fn update(&mut self) {
        self.need_update = true;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum AnimatedTextureSpeed {
    Fast,
    Slow,
}

#[derive(Debug, Component)]
struct AnimatedTexture {
    speed: AnimatedTextureSpeed,
    n_frame: u8,
    start: u8,
    monochrome_shift: u8,
}

#[derive(Debug, Default, Resource)]
struct AnimationCounter {
    fast: usize,
    slow: usize,
}

const CORNERS: [Coords; 4] = [Coords(-1, -1), Coords(-1, 1), Coords(1, 1), Coords(1, -1)];

const CORNER_PIECE_GRID: [(usize, usize); 4] = [(0, 1), (0, 0), (1, 0), (1, 1)];

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateDraw>()
            .init_resource::<UpdateDraw>()
            .init_resource::<DisplayOpts>()
            .init_resource::<LayeredTexMap>()
            .init_resource::<AnimationCounter>()
            .add_systems(
                Update,
                (
                    update_layered_tex_map.pipe(spawn_map_textures),
                    spawn_structure_textures,
                    spawn_animal_textures,
                    spawn_tile_animation_textures,
                    spawn_overlay_meshes,
                )
                    .in_set(GameSystemSet::Draw)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(Update, reset_update_draw.after(GameSystemSet::Draw))
            .add_systems(
                Update,
                update_animation.run_if(bevy::time::common_conditions::on_timer(
                    std::time::Duration::from_millis(500),
                )),
            );
    }
}

#[derive(Resource)]
pub struct LayeredTexMap {
    biome: Array2d<ArrayVec<Biome, 9>>,
}

impl Default for LayeredTexMap {
    fn default() -> Self {
        LayeredTexMap {
            biome: Array2d::new(1, 1, ArrayVec::new()),
        }
    }
}

fn update_layered_tex_map(
    update_draw: Res<UpdateDraw>,
    params: Res<Params>,
    planet: Res<Planet>,
    mut ltm: ResMut<LayeredTexMap>,
) {
    if !update_draw.need_update && !planet.is_changed() {
        return;
    }

    let (w, h) = planet.map.size();
    if ltm.biome.size() != (w, h) {
        ltm.biome = Array2d::new(w, h, ArrayVec::new());
    } else {
        ltm.biome.fill(ArrayVec::new())
    }
    let tiles = &mut ltm.biome;

    for &i in params.biomes.keys() {
        for pos in RectIter::new((0, 0), (w - 1, h - 1)) {
            let biome_i = planet.map[pos].biome;
            if biome_i != i {
                continue;
            }

            let tile_z = params.biomes[&biome_i].z;
            tiles[pos].push(i);
            for d in Direction::EIGHT_DIRS {
                let p = coord_rotation_x(planet.map.size(), pos + d.as_coords());
                if tiles.in_range(p) {
                    let surround_tile_i = planet.map[p].biome;
                    let z = params.biomes[&surround_tile_i].z;
                    if z < tile_z && !tiles[pos].contains(&surround_tile_i) {
                        tiles[pos].push(surround_tile_i);
                    }
                }
            }
        }
    }
}

fn spawn_map_textures(
    mut commands: Commands,
    update_draw: Res<UpdateDraw>,
    ltm: Res<LayeredTexMap>,
    params: Res<Params>,
    biome_textures: Res<BiomeTextures>,
    texture_handles: Res<TextureHandles>,
    in_screen_tile_range: Res<InScreenTileRange>,
    current_layer: Res<OverlayLayerKind>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_draw.need_update {
        return;
    }

    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    // Spawn biome textures
    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(ltm.biome.size(), p_screen);
        for tile_idx in &ltm.biome[p] {
            for (corner, corner_piece_grid) in CORNERS.into_iter().zip(CORNER_PIECE_GRID) {
                let corner_index = corner_idx(
                    |pos| {
                        let pos = coord_rotation_x(ltm.biome.size(), pos);
                        if ltm.biome.in_range(pos) {
                            ltm.biome[pos].contains(tile_idx)
                        } else {
                            true
                        }
                    },
                    p,
                    corner,
                );

                let grid_x = (corner_index % 3) * 2 + corner_piece_grid.0;
                let grid_y =
                    (corner_index / 3) * 2 + corner_piece_grid.1 + if monochrome { 4 } else { 0 };

                let index = grid_x + grid_y * 6;

                let x = p_screen.0 as f32 * TILE_SIZE
                    + PIECE_SIZE * ((corner.0 + 1) / 2) as f32
                    + PIECE_SIZE / 2.0;
                let y = p_screen.1 as f32 * TILE_SIZE
                    + PIECE_SIZE * ((corner.1 + 1) / 2) as f32
                    + PIECE_SIZE / 2.0;

                let tile_asset = &params.biomes[tile_idx];
                let id = commands
                    .spawn((
                        Sprite::from_atlas_image(
                            biome_textures.get(*tile_idx),
                            TextureAtlas {
                                index,
                                layout: texture_handles.biome_layouts[tile_idx].clone(),
                            },
                        ),
                        Transform::from_xyz(x, y, tile_asset.z / 10.0),
                    ))
                    .id();
                tex_entities.push(id);
            }
        }
    }
}

fn spawn_structure_textures(
    mut commands: Commands,
    update_draw: Res<UpdateDraw>,
    params: Res<Params>,
    structure_textures: Res<StructureTextures>,
    texture_handles: Res<TextureHandles>,
    in_screen_tile_range: Res<InScreenTileRange>,
    planet: Res<Planet>,
    (current_layer, display_opts): (Res<OverlayLayerKind>, Res<DisplayOpts>),
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_draw.need_update {
        return;
    }
    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(planet.map.size(), p_screen);
        let Some(structure) = &planet.map[p].structure else {
            continue;
        };

        let kind: StructureKind = structure.into();
        let attrs = &params.structures[&kind];

        let (index, civ_icon) = if let Structure::Settlement(settlement) = structure {
            if !display_opts.cities {
                continue;
            }
            let civ_icon = if display_opts.city_icons {
                Some(settlement.id)
            } else {
                None
            };
            (settlement.age as usize, civ_icon)
        } else {
            if !display_opts.structures {
                continue;
            }
            (0, None)
        };
        let index = if monochrome {
            index + attrs.columns
        } else {
            index
        };

        let x = p_screen.0 as f32 * TILE_SIZE + attrs.width as f32 / 2.0;
        let y = p_screen.1 as f32 * TILE_SIZE + attrs.height as f32 / 2.0;
        let id = commands
            .spawn((
                Sprite::from_atlas_image(
                    structure_textures.get(kind),
                    TextureAtlas {
                        index,
                        layout: texture_handles.structure_layouts[&kind].clone(),
                    },
                ),
                Transform::from_xyz(x, y, 300.0 - p.1 as f32 / 256.0),
            ))
            .id();
        tex_entities.push(id);

        if let Some(id) = civ_icon {
            let t = &texture_handles.animals[&id];
            let x = x - (TILE_SIZE - t.width as f32) / 2.0;
            let y = y - (TILE_SIZE - t.height as f32) / 2.0;
            let id = commands
                .spawn((
                    Sprite {
                        image: t.image.clone(),
                        texture_atlas: Some(TextureAtlas {
                            index: 2,
                            layout: t.layout.clone(),
                        }),
                        ..default()
                    },
                    Transform::from_xyz(x, y, 410.0),
                ))
                .id();
            tex_entities.push(id);
        }
    }
}

fn spawn_animal_textures(
    mut commands: Commands,
    update_draw: Res<UpdateDraw>,
    texture_handles: Res<TextureHandles>,
    in_screen_tile_range: Res<InScreenTileRange>,
    planet: Res<Planet>,
    params: Res<Params>,
    (current_layer, display_opts): (Res<OverlayLayerKind>, Res<DisplayOpts>),
    counter: Res<AnimationCounter>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_draw.need_update {
        return;
    }
    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();
    if !display_opts.animals {
        return;
    }

    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(planet.map.size(), p_screen);
        let tile = &planet.map[p];

        if tile.structure.is_some() {
            if matches!(planet.map[p].structure, Some(Structure::Settlement(_))) {
                if display_opts.cities {
                    continue;
                }
            } else if display_opts.structures {
                continue;
            }
        }
        if tile
            .tile_events
            .list()
            .iter()
            .any(|te| matches!(te, TileEvent::Vehicle { .. } | TileEvent::Troop { .. }))
        {
            continue;
        }

        // Select the largest animal at this tile
        let Some(animal) = tile.largest_animal() else {
            continue;
        };

        let t = &texture_handles.animals[&animal.id];
        let monochrome_shift: u8 = if params.animals[&animal.id].civ.is_some() {
            3
        } else {
            2
        };
        let animated_texture = AnimatedTexture {
            speed: AnimatedTextureSpeed::Slow,
            n_frame: 2,
            start: 0,
            monochrome_shift,
        };
        let index = animated_texture.index(counter.slow, monochrome);

        let x = (p_screen.0 as f32 + 0.5) * TILE_SIZE;
        let y = (p_screen.1 as f32 + 0.5) * TILE_SIZE;
        let id = commands
            .spawn((
                Sprite {
                    image: t.image.clone(),
                    texture_atlas: Some(TextureAtlas {
                        index,
                        layout: t.layout.clone(),
                    }),
                    ..default()
                },
                Transform::from_xyz(x, y, 400.0),
                animated_texture,
            ))
            .id();
        tex_entities.push(id);
    }
}

fn spawn_tile_animation_textures(
    mut commands: Commands,
    update_draw: Res<UpdateDraw>,
    texture_handles: Res<TextureHandles>,
    in_screen_tile_range: Res<InScreenTileRange>,
    planet: Res<Planet>,
    current_layer: Res<OverlayLayerKind>,
    counter: Res<AnimationCounter>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_draw.need_update {
        return;
    }
    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(planet.map.size(), p_screen);
        let tile = &planet.map[p];

        let Some((_, tile_event)) = tile
            .tile_events
            .list()
            .iter()
            .filter_map(|e| {
                let key = tile_event_order_key(e);
                if key > 0 { Some((key, e)) } else { None }
            })
            .max_by_key(|(key, _)| *key)
        else {
            continue;
        };
        let animated_texture = tile_event_texture(tile_event, tile);
        let index = animated_texture.index(counter.fast, monochrome);
        let Some(t) = texture_handles.tile_animations.get(tile_event.kind().as_ref()) else {
            continue;
        };

        let x = (p_screen.0 as f32 + 0.5) * TILE_SIZE;
        let y = (p_screen.1 as f32 + 0.5) * TILE_SIZE;
        let id = commands
            .spawn((
                Sprite {
                    image: t.image.clone(),
                    texture_atlas: Some(TextureAtlas {
                        index,
                        layout: t.layout.clone(),
                    }),
                    ..default()
                },
                Transform::from_xyz(x, y, 500.0),
                animated_texture,
            ))
            .id();
        tex_entities.push(id);
    }
}

fn spawn_overlay_meshes(
    mut commands: Commands,
    update_draw: Res<UpdateDraw>,
    mut meshes: ResMut<Assets<Mesh>>,
    color_materials: Res<ColorMaterials>,
    in_screen_tile_range: Res<InScreenTileRange>,
    (planet, params): (Res<Planet>, Res<Params>),
    current_layer: Res<OverlayLayerKind>,
    mut prev_layer: Local<OverlayLayerKind>,
    mut tile_mesh: Local<Option<Handle<Mesh>>>,
    mut mesh_entities: Local<Vec<Entity>>,
) {
    if !update_draw.need_update && *current_layer == *prev_layer {
        return;
    }

    for entity in mesh_entities.iter() {
        commands.entity(*entity).despawn();
    }
    mesh_entities.clear();

    *prev_layer = *current_layer;
    if *current_layer == OverlayLayerKind::None {
        return;
    }

    let tile_mesh = if let Some(tile_mesh) = tile_mesh.clone() {
        tile_mesh
    } else {
        *tile_mesh = Some(meshes.add(Mesh::from(Rectangle::new(TILE_SIZE, TILE_SIZE))));
        tile_mesh.clone().unwrap()
    };

    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(planet.map.size(), p_screen);

        let x = p_screen.0 as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = p_screen.1 as f32 * TILE_SIZE + TILE_SIZE / 2.0;

        let id = commands
            .spawn((
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(color_materials.get_handle(&planet, p, *current_layer, &params)),
                Transform::from_xyz(x, y, 800.0),
            ))
            .id();
        mesh_entities.push(id);
    }
}

fn update_animation(
    mut counter: ResMut<AnimationCounter>,
    current_layer: Res<OverlayLayerKind>,
    mut query: Query<(&AnimatedTexture, &mut Sprite)>,
) {
    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    counter.fast = counter.fast.wrapping_add(1);
    if counter.fast % 2 == 0 {
        counter.slow = counter.slow.wrapping_add(1);
    }

    for (animated_texture, mut sprite) in &mut query {
        let counter = match animated_texture.speed {
            AnimatedTextureSpeed::Fast => counter.fast,
            AnimatedTextureSpeed::Slow => counter.slow,
        };
        sprite.texture_atlas.as_mut().unwrap().index = animated_texture.index(counter, monochrome);
    }
}

fn corner_idx<F: Fn(Coords) -> bool>(f: F, pos: Coords, corner: Coords) -> usize {
    let a = f(pos + (corner.0, 0));
    let b = f(pos + (0, corner.1));
    let c = f(pos + corner);

    match (a, b, c) {
        (true, true, true) => 0,
        (true, false, _) => 1,
        (false, true, _) => 2,
        (false, false, _) => 3,
        (true, true, false) => 4,
    }
}

fn reset_update_draw(mut update_draw: ResMut<UpdateDraw>) {
    update_draw.need_update = false;
}

fn coord_rotation_x(size: (u32, u32), p: Coords) -> Coords {
    let w = size.0 as i32;
    let new_x = if p.0 < 0 {
        p.0 + (-p.0 / w + 1) * w
    } else if p.0 >= w {
        p.0 - (p.0 / w) * w
    } else {
        p.0
    };
    Coords(new_x, p.1)
}

fn tile_event_order_key(tile_event: &TileEvent) -> u32 {
    match tile_event {
        TileEvent::Fire => 90,
        TileEvent::BlackDust { .. } => 10,
        TileEvent::AerosolInjection { .. } => 20,
        TileEvent::Plague { cured, .. } => {
            if *cured {
                0
            } else {
                31
            }
        }
        TileEvent::Vehicle { .. } => 40,
        TileEvent::Decadence { cured } => {
            if *cured {
                0
            } else {
                30
            }
        }
        TileEvent::War { .. } => 32,
        TileEvent::NuclearExplosion { .. } => 90,
        TileEvent::Troop { .. } => 41,
    }
}

fn tile_event_texture(tile_event: &TileEvent, tile: &Tile) -> AnimatedTexture {
    match tile_event {
        TileEvent::Vehicle {
            kind,
            age,
            direction,
            ..
        } => {
            let start = match kind {
                VehicleKind::Ship => {
                    if *age <= CivilizationAge::Iron {
                        0
                    } else {
                        4
                    }
                }
                VehicleKind::AirPlane => 8,
            };
            let start = if direction.0 < 0 { start } else { start + 2 };
            AnimatedTexture {
                speed: AnimatedTextureSpeed::Fast,
                n_frame: 2,
                monochrome_shift: 12,
                start,
            }
        }
        TileEvent::Troop { age, .. } => {
            let mut start = 0;
            if tile.biome.is_sea() {
                start += 2;
            }
            if *age >= CivilizationAge::Industrial {
                start += 4;
            }
            AnimatedTexture {
                speed: AnimatedTextureSpeed::Fast,
                n_frame: 2,
                monochrome_shift: 4,
                start,
            }
        }
        TileEvent::NuclearExplosion { .. } => AnimatedTexture {
            speed: AnimatedTextureSpeed::Fast,
            n_frame: 3,
            monochrome_shift: 3,
            start: 0,
        },
        _ => AnimatedTexture {
            speed: AnimatedTextureSpeed::Fast,
            n_frame: 2,
            monochrome_shift: 2,
            start: 0,
        },
    }
}

impl AnimatedTexture {
    fn index(&self, counter: usize, monochrome: bool) -> usize {
        let index = self.start as usize + counter % self.n_frame as usize;
        if monochrome {
            index + self.monochrome_shift as usize
        } else {
            index
        }
    }
}
