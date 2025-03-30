use std::cmp::Ordering;

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::AudioSource;
use compact_str::CompactString;
use fnv::FnvHashMap;
use serde::Deserialize;
use serde_with::{DisplayFromStr, Same, serde_as};
use strum::IntoEnumIterator;

use crate::GameState;
use crate::audio::MusicListAsset;
use crate::conf::Conf;
use crate::gz::GunzipBin;
use crate::planet::*;

#[derive(Clone, Copy, Debug)]
pub struct AssetsPlugin;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, SystemSet)]
pub struct AssetsListSystemSet;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ParamsAsset>::new(&["params.ron"]))
            .add_plugins(RonAssetPlugin::<BiomeAssetList>::new(&["biomes.ron"]))
            .add_plugins(RonAssetPlugin::<StructureAssetList>::new(&[
                "structures.ron",
            ]))
            .add_plugins(RonAssetPlugin::<StartPlanetAsset>::new(&[
                "start_planet.ron",
            ]))
            .add_plugins(RonAssetPlugin::<AnimalAsset>::new(&["animal.ron"]))
            .add_plugins(RonAssetPlugin::<MusicListAsset>::new(&["music.ron"]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                    .continue_to_state(GameState::MainMenu)
                    .load_collection::<crate::text_assets::TranslationTexts>()
                    .load_collection::<PlanetAssetCollection>()
                    .load_collection::<UiAssets>()
                    .load_collection::<BiomeTextures>()
                    .load_collection::<StructureTextures>()
                    .load_collection::<SoundEffectSources>()
                    .load_collection::<MusicSources>(),
            )
            .add_systems(
                OnExit(GameState::AssetLoading),
                (
                    create_assets_list,
                    crate::text_assets::set_text_global,
                    crate::audio::set_music_list,
                )
                    .in_set(AssetsListSystemSet),
            );
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct UiAssets {
    #[asset(path = "default.conf.toml")]
    pub default_conf: Handle<Conf>,
    #[asset(path = "fonts/Mplus2-SemiBold.otf.gz")]
    pub font: Handle<GunzipBin>,
    #[asset(path = "ui", collection(mapped, typed))]
    pub ui_imgs: HashMap<String, Handle<Image>>,
    #[asset(path = "start_planets", collection(mapped))]
    pub start_planet: HashMap<String, UntypedHandle>,
    #[asset(path = "animals", collection(mapped))]
    pub animal_imgs: HashMap<String, UntypedHandle>,
    #[asset(path = "ui/tile-colored.png")]
    pub tile_colored: Handle<Image>,
}

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct ParamsAsset(Params);

#[serde_as]
#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct BiomeAssetList(
    #[serde_as(as = "FnvHashMap<DisplayFromStr, Same>")] FnvHashMap<Biome, BiomeAttrs>,
);

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct StructureAssetList(FnvHashMap<StructureKind, StructureAttrs>);

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct StartPlanetAsset(StartPlanet);

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct AnimalAsset(AnimalAttr);

#[derive(Resource)]
pub struct TextureHandles {
    pub biome_layouts: FnvHashMap<Biome, Handle<TextureAtlasLayout>>,
    pub structure_layouts: FnvHashMap<StructureKind, Handle<TextureAtlasLayout>>,
    pub animals: HashMap<AnimalId, LoadedTexture>,
    pub tile_animations: HashMap<CompactString, LoadedTexture>,
}

#[derive(Clone, Debug)]
pub struct LoadedTexture {
    pub layout: Handle<TextureAtlasLayout>,
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Resource, AssetCollection)]
pub struct PlanetAssetCollection {
    #[asset(path = "planet.params.ron")]
    params: Handle<ParamsAsset>,
    #[asset(path = "biomes/list.biomes.ron")]
    biomes: Handle<BiomeAssetList>,
    #[asset(path = "structures/list.structures.ron")]
    structures: Handle<StructureAssetList>,
    #[asset(path = "start_planets", collection(mapped))]
    start_planet_handles: HashMap<String, UntypedHandle>,
    #[asset(path = "animals", collection(mapped))]
    animal_handles: HashMap<String, UntypedHandle>,
    #[asset(path = "tile_animations", collection(mapped, typed))]
    tile_animation_handles: HashMap<String, Handle<Image>>,
}

define_asset_list_from_enum! {
    #[asset(dir_path = "biomes")]
    #[asset(extension = "png")]
    pub struct BiomeTextures {
        pub textures: HashMap<Biome, Handle<Image>>,
    }
}

define_asset_list_from_enum! {
    #[asset(dir_path = "structures")]
    #[asset(extension = "png")]
    pub struct StructureTextures {
        pub textures: HashMap<StructureKind, Handle<Image>>,
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct SoundEffectSources {
    #[asset(path = "se", collection(mapped, typed))]
    pub sound_effects: HashMap<String, Handle<AudioSource>>,
}

#[derive(Debug, Resource, AssetCollection)]
pub struct MusicSources {
    #[asset(path = "music", collection(mapped))]
    pub music_handles: HashMap<String, UntypedHandle>,
}

fn create_assets_list(
    mut command: Commands,
    images: Res<Assets<Image>>,
    planet_asset_collection: Res<PlanetAssetCollection>,
    (params_asset, biome_asset_list, structure_asset_list, start_planet_assets, animal_assets): (
        Res<Assets<ParamsAsset>>,
        Res<Assets<BiomeAssetList>>,
        Res<Assets<StructureAssetList>>,
        Res<Assets<StartPlanetAsset>>,
        Res<Assets<AnimalAsset>>,
    ),
    mut texture_atlas_assets: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Biomes
    let biome_asset_list = biome_asset_list.get(&planet_asset_collection.biomes).unwrap();
    let mut biome_texture_rects = HashMap::new();
    for j in 0..4 {
        for i in 0..3 {
            let x = (TILE_SIZE as u32 + 2) * i;
            let y = (TILE_SIZE as u32 + 2) * j;
            let rect_size = UVec2::new(PIECE_SIZE as u32, PIECE_SIZE as u32);
            biome_texture_rects.insert(
                (i * 2, j * 2),
                URect::from_center_size(
                    UVec2::new(x + 1 + PIECE_SIZE as u32 / 2, y + 1 + PIECE_SIZE as u32 / 2),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2 + 1, j * 2),
                URect::from_center_size(
                    UVec2::new(
                        x + 1 + (PIECE_SIZE * 1.5) as u32,
                        y + 1 + PIECE_SIZE as u32 / 2,
                    ),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2, j * 2 + 1),
                URect::from_center_size(
                    UVec2::new(
                        x + 1 + PIECE_SIZE as u32 / 2,
                        y + 1 + (PIECE_SIZE * 1.5) as u32,
                    ),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2 + 1, j * 2 + 1),
                URect::from_center_size(
                    UVec2::new(
                        x + 1 + (PIECE_SIZE * 1.5) as u32,
                        y + 1 + (PIECE_SIZE * 1.5) as u32,
                    ),
                    rect_size,
                ),
            );
        }
    }
    let biomes = Biome::iter()
        .map(|biome| {
            let mut texture_atlas = TextureAtlasLayout::new_empty(UVec2::new(
                (TILE_SIZE as u32 + 2) * 3,
                (TILE_SIZE as u32 + 2) * 4,
            ));
            for j in 0..8 {
                for i in 0..6 {
                    texture_atlas.add_texture(biome_texture_rects[&(i, j)]);
                }
            }
            (biome, texture_atlas_assets.add(texture_atlas))
        })
        .collect();

    // Structures
    let structure_asset_list = structure_asset_list
        .get(&planet_asset_collection.structures)
        .unwrap();
    let structures = StructureKind::iter()
        .map(|structure| {
            let attrs = &structure_asset_list.0[&structure];
            let texture_atlas = TextureAtlasLayout::from_grid(
                UVec2::new(attrs.width, attrs.height),
                attrs.columns as u32,
                attrs.rows as u32,
                None,
                None,
            );

            (structure, texture_atlas_assets.add(texture_atlas))
        })
        .collect();

    let mut params = params_asset
        .get(&planet_asset_collection.params)
        .unwrap()
        .clone()
        .0;
    params.biomes = biome_asset_list.0.clone();
    params.structures = structure_asset_list.0.clone();

    // Start planets
    for handle in planet_asset_collection.start_planet_handles.values() {
        if let Ok(handle) = handle.clone().try_typed::<StartPlanetAsset>() {
            let start_planet = start_planet_assets.get(&handle).cloned().unwrap().0;
            params.start_planets.push(start_planet);
        }
    }
    params
        .start_planets
        .sort_by(|a, b| match a.habitability.cmp(&b.habitability) {
            Ordering::Equal => a.id.cmp(&b.id),
            o => o,
        });

    // Animal
    for (path, handle) in &planet_asset_collection.animal_handles {
        let animal_id = path
            .strip_prefix("animals/")
            .and_then(|s| s.split_once('.'))
            .expect("unexpected animal asset path")
            .0;
        let animal_id = match AnimalId::from(animal_id) {
            Ok(animal_id) => animal_id,
            Err(e) => {
                log::warn!("invalid string \"{}\" for animal id: {}", animal_id, e);
                continue;
            }
        };

        if let Ok(handle) = handle.clone().try_typed::<AnimalAsset>() {
            let animal = animal_assets.get(&handle).cloned().unwrap().0;
            params.animals.insert(animal_id, animal);
            continue;
        }
    }

    let mut animals = HashMap::default();
    for (path, handle) in &planet_asset_collection.animal_handles {
        let animal_id = path
            .strip_prefix("animals/")
            .and_then(|s| s.split_once('.'))
            .expect("unexpected animal asset path")
            .0;
        let animal_id = match AnimalId::from(animal_id) {
            Ok(animal_id) => animal_id,
            Err(e) => {
                log::warn!("invalid string \"{}\" for animal id: {}", animal_id, e);
                continue;
            }
        };

        let Ok(handle) = handle.clone().try_typed::<Image>() else {
            continue;
        };
        let attr = params.animals.get(&animal_id).expect("no animal parameters");

        let nw = if attr.civ.is_some() { 3 } else { 2 };
        let image = images.get(&handle).unwrap();
        let width = image.width() / nw;
        let height = image.height() / 2;
        let layout = texture_atlas_assets.add(TextureAtlasLayout::from_grid(
            UVec2::new(width, height),
            nw,
            2,
            None,
            None,
        ));
        animals.insert(
            animal_id,
            LoadedTexture {
                layout,
                image: handle,
                width,
                height,
            },
        );
    }

    // Tile animations
    let tile_animation_layout = texture_atlas_assets.add(TextureAtlasLayout::from_grid(
        UVec2::new(TILE_SIZE as u32, TILE_SIZE as u32),
        2,
        2,
        None,
        None,
    ));
    let tile_animations: HashMap<_, _> = planet_asset_collection
        .tile_animation_handles
        .iter()
        .map(|(path, handle)| {
            let tile_animation_id: CompactString = path
                .strip_prefix("tile_animations/")
                .and_then(|s| s.split_once('.'))
                .expect("unexpected tile animation asset path")
                .0
                .into();
            let image = images.get(handle).unwrap();
            let nw = image.width() / TILE_SIZE as u32;
            let nh = image.height() / TILE_SIZE as u32;
            let layout = if nw == 2 && nh == 2 {
                tile_animation_layout.clone()
            } else {
                texture_atlas_assets.add(TextureAtlasLayout::from_grid(
                    UVec2::new(TILE_SIZE as u32, TILE_SIZE as u32),
                    nw,
                    nh,
                    None,
                    None,
                ))
            };
            (
                tile_animation_id,
                LoadedTexture {
                    layout,
                    image: handle.clone(),
                    width: TILE_SIZE as u32,
                    height: TILE_SIZE as u32,
                },
            )
        })
        .collect();

    command.insert_resource(params);
    command.insert_resource(TextureHandles {
        biome_layouts: biomes,
        structure_layouts: structures,
        animals,
        tile_animations,
    });
}
