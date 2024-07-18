use crate::overlay::{ColorMaterials, OverlayLayerKind};
use crate::screen::InScreenTileRange;
use crate::{assets::*, GameState};
use crate::{planet::*, GameSystemSet};
use arrayvec::ArrayVec;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use geom::{Array2d, Coords, Direction, RectIter};

#[derive(Clone, Copy, Debug)]
pub struct DrawPlugin;

#[derive(Clone, Copy, Default, Debug, Resource, Event)]
pub struct UpdateMap {
    need_update: bool,
}

impl UpdateMap {
    pub fn update(&mut self) {
        self.need_update = true;
    }
}

const CORNERS: [Coords; 4] = [Coords(-1, -1), Coords(-1, 1), Coords(1, 1), Coords(1, -1)];

const CORNER_PIECE_GRID: [(usize, usize); 4] = [(0, 1), (0, 0), (1, 0), (1, 1)];

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UpdateMap>()
            .init_resource::<UpdateMap>()
            .init_resource::<LayeredTexMap>()
            .add_systems(
                Update,
                (
                    update_layered_tex_map.pipe(spawn_map_textures),
                    spawn_structure_textures,
                    spawn_overlay_meshes,
                )
                    .in_set(GameSystemSet::Draw)
                    .run_if(in_state(GameState::Running)),
            )
            .add_systems(Update, reset_update_map.after(GameSystemSet::Draw));
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
    update_map: Res<UpdateMap>,
    params: Res<Params>,
    planet: Res<Planet>,
    mut ltm: ResMut<LayeredTexMap>,
) {
    if !update_map.need_update && !planet.is_changed() {
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
    update_map: Res<UpdateMap>,
    ltm: Res<LayeredTexMap>,
    params: Res<Params>,
    biome_textures: Res<BiomeTextures>,
    texture_atlas_layouts: Res<TextureAtlasLayouts>,
    in_screen_tile_range: Res<InScreenTileRange>,
    current_layer: Res<OverlayLayerKind>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_map.need_update {
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
                        SpriteBundle {
                            texture: biome_textures.get(*tile_idx),
                            sprite: Sprite::default(),
                            transform: Transform::from_xyz(x, y, tile_asset.z / 10.0),
                            visibility: Visibility::Visible,
                            ..default()
                        },
                        TextureAtlas {
                            index,
                            layout: texture_atlas_layouts.biomes[tile_idx].clone(),
                        },
                    ))
                    .id();
                tex_entities.push(id);
            }
        }
    }
}

fn spawn_structure_textures(
    mut commands: Commands,
    update_map: Res<UpdateMap>,
    params: Res<Params>,
    structure_textures: Res<StructureTextures>,
    texture_atlas_layouts: Res<TextureAtlasLayouts>,
    in_screen_tile_range: Res<InScreenTileRange>,
    planet: Res<Planet>,
    current_layer: Res<OverlayLayerKind>,
    mut tex_entities: Local<Vec<Entity>>,
) {
    if !update_map.need_update {
        return;
    }
    for entity in tex_entities.iter() {
        commands.entity(*entity).despawn();
    }
    tex_entities.clear();

    let monochrome = !matches!(*current_layer, OverlayLayerKind::None);

    for p_screen in RectIter::new(in_screen_tile_range.from, in_screen_tile_range.to) {
        let p = coord_rotation_x(planet.map.size(), p_screen);
        let structure = &planet.map[p].structure;

        if !matches!(structure, Structure::None | Structure::Occupied { .. }) {
            let kind: StructureKind = structure.into();
            let attrs = &params.structures[&kind];

            let index = if let Structure::Settlement { settlement } = structure {
                settlement.age as usize
            } else {
                0
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
                    SpriteBundle {
                        texture: structure_textures.get(kind),
                        sprite: Sprite::default(),
                        transform: Transform::from_xyz(x, y, 300.0 - p.1 as f32 / 256.0),
                        visibility: Visibility::Inherited,
                        ..default()
                    },
                    TextureAtlas {
                        index,
                        layout: texture_atlas_layouts.structures[&kind].clone(),
                    },
                ))
                .id();
            tex_entities.push(id);
        }
    }
}

fn spawn_overlay_meshes(
    mut commands: Commands,
    update_map: Res<UpdateMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    color_materials: Res<ColorMaterials>,
    in_screen_tile_range: Res<InScreenTileRange>,
    planet: Res<Planet>,
    current_layer: Res<OverlayLayerKind>,
    mut prev_layer: Local<OverlayLayerKind>,
    mut tile_mesh: Local<Option<Handle<Mesh>>>,
    mut mesh_entities: Local<Vec<Entity>>,
) {
    if !update_map.need_update && *current_layer == *prev_layer {
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
            .spawn(MaterialMesh2dBundle {
                mesh: tile_mesh.clone().into(),
                transform: Transform::from_xyz(x, y, 800.0),
                material: color_materials.get_handle(&planet, p, *current_layer),
                ..default()
            })
            .id();
        mesh_entities.push(id);
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

fn reset_update_map(mut update_map: ResMut<UpdateMap>) {
    update_map.need_update = false;
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
