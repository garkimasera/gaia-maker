use crate::audio::SoundEffect;
use crate::conf::Conf;
use crate::gz::GunzipBin;
use crate::planet::*;
use crate::text::{Lang, TranslationText};
use crate::GameState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_kira_audio::AudioSource;
use fnv::FnvHashMap;
use serde::Deserialize;
use std::collections::HashMap;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Clone, Copy, Debug)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<ParamsAsset>::new(&["params.ron"]))
            .add_plugins(RonAssetPlugin::<BiomeAssetList>::new(&["biomes.ron"]))
            .add_plugins(RonAssetPlugin::<StructureAssetList>::new(&[
                "structures.ron",
            ]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::MainMenu),
            )
            .add_collection_to_loading_state::<_, TranslationTexts>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, UiTextures>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, UiAssets>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, ParamsAssetCollection>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, BiomeTextures>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, StructureTextures>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, SoundEffects>(GameState::AssetLoading)
            .add_systems(OnExit(GameState::AssetLoading), create_assets_list);
    }
}

define_asset_list_from_enum! {
    #[asset(dir_path = "texts")]
    #[asset(extension = "text.ron")]
    pub struct TranslationTexts {
        pub texts: HashMap<Lang, Handle<TranslationText>>,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum UiTexture {
    IconAirTemprature,
    IconBiomass,
    IconBuild,
    IconCoordinates,
    IconFertility,
    IconGameMenu,
    IconHeight,
    IconHelp,
    IconLayers,
    IconMaterial,
    IconMessage,
    IconOrbit,
    IconPower,
    IconRainfall,
    IconResourceEnergy,
    IconResourceMaterial,
    IconResourceIce,
    IconResourceCarbon,
    IconResourceNitrogen,
    IconSpeedFast,
    IconSpeedFastSelected,
    IconSpeedNormal,
    IconSpeedNormalSelected,
    IconSpeedPaused,
    IconSpeedPausedSelected,
    IconStarSystem,
    IconStat,
    TileColored,
    TileCursor,
}

impl From<ResourceKind> for UiTexture {
    fn from(kind: ResourceKind) -> Self {
        match kind {
            ResourceKind::Energy => Self::IconResourceEnergy,
            ResourceKind::Material => Self::IconResourceMaterial,
            ResourceKind::Ice => Self::IconResourceIce,
            ResourceKind::Carbon => Self::IconResourceCarbon,
            ResourceKind::Nitrogen => Self::IconResourceNitrogen,
        }
    }
}

define_asset_list_from_enum! {
    #[asset(dir_path = "ui")]
    #[asset(extension = "png")]
    pub struct UiTextures {
        pub textures: HashMap<UiTexture, Handle<Image>>,
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct UiAssets {
    #[asset(path = "default.conf.ron")]
    pub default_conf: Handle<Conf>,
    #[asset(path = "fonts/Mplus2-SemiBold.otf.gz")]
    pub font: Handle<GunzipBin>,
}

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "b0aaec37-3e9e-42d0-9370-aaacbe550799"]
pub struct ParamsAsset(Params);

impl bevy::reflect::TypePath for ParamsAsset {
    fn type_path() -> &'static str {
        "gaia_maker::assets::ParamsAsset"
    }
    fn short_type_path() -> &'static str {
        "ParamsAsset"
    }
}

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "99d5021f-98fb-4873-b16a-bd9619b8b074"]
pub struct BiomeAssetList(FnvHashMap<Biome, BiomeAttrs>);

impl bevy::reflect::TypePath for BiomeAssetList {
    fn type_path() -> &'static str {
        "gaia_maker::assets::BiomeAssetList"
    }
    fn short_type_path() -> &'static str {
        "BiomeAssetList"
    }
}

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "801a2daa-956d-469a-8e83-3610fbca21fd"]
pub struct StructureAssetList(FnvHashMap<StructureKind, StructureAttrs>);

impl bevy::reflect::TypePath for StructureAssetList {
    fn type_path() -> &'static str {
        "gaia_maker::assets::StructureAssetList"
    }
    fn short_type_path() -> &'static str {
        "StructureAssetList"
    }
}

#[derive(Resource)]
pub struct TextureAtlasMaps {
    pub biomes: FnvHashMap<Biome, Handle<TextureAtlas>>,
    pub structures: FnvHashMap<StructureKind, Handle<TextureAtlas>>,
}

#[derive(Debug, Resource, AssetCollection)]
pub struct ParamsAssetCollection {
    #[asset(path = "planet.params.ron")]
    params: Handle<ParamsAsset>,
    #[asset(path = "biomes/list.biomes.ron")]
    biomes: Handle<BiomeAssetList>,
    #[asset(path = "structures/list.structures.ron")]
    structures: Handle<StructureAssetList>,
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

define_asset_list_from_enum! {
    #[asset(dir_path = "se")]
    #[asset(extension = "ogg")]
    pub struct SoundEffects {
        pub sound_effects: HashMap<SoundEffect, Handle<AudioSource>>,
    }
}

fn create_assets_list(
    mut command: Commands,
    params_asset_collection: Res<ParamsAssetCollection>,
    biome_textures: Res<BiomeTextures>,
    structure_textures: Res<StructureTextures>,
    (params_asset, biome_asset_list, structure_asset_list): (
        Res<Assets<ParamsAsset>>,
        Res<Assets<BiomeAssetList>>,
        Res<Assets<StructureAssetList>>,
    ),
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
) {
    let biome_asset_list = biome_asset_list
        .get(&params_asset_collection.biomes)
        .unwrap();
    let mut biome_texture_rects = HashMap::new();
    for j in 0..4 {
        for i in 0..3 {
            let x = (TILE_SIZE + 2.0) * i as f32;
            let y = (TILE_SIZE + 2.0) * j as f32;
            let rect_size = Vec2::new(PIECE_SIZE, PIECE_SIZE);
            biome_texture_rects.insert(
                (i * 2, j * 2),
                Rect::from_center_size(
                    Vec2::new(x + 1.0 + PIECE_SIZE * 0.5, y + 1.0 + PIECE_SIZE * 0.5),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2 + 1, j * 2),
                Rect::from_center_size(
                    Vec2::new(x + 1.0 + PIECE_SIZE * 1.5, y + 1.0 + PIECE_SIZE * 0.5),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2, j * 2 + 1),
                Rect::from_center_size(
                    Vec2::new(x + 1.0 + PIECE_SIZE * 0.5, y + 1.0 + PIECE_SIZE * 1.5),
                    rect_size,
                ),
            );
            biome_texture_rects.insert(
                (i * 2 + 1, j * 2 + 1),
                Rect::from_center_size(
                    Vec2::new(x + 1.0 + PIECE_SIZE * 1.5, y + 1.0 + PIECE_SIZE * 1.5),
                    rect_size,
                ),
            );
        }
    }

    let biomes = Biome::iter()
        .map(|biome| {
            let image = biome_textures.get(biome);
            let mut texture_atlas = TextureAtlas::new_empty(
                image,
                Vec2::new((TILE_SIZE + 2.0) * 3.0, (TILE_SIZE + 2.0) * 4.0),
            );
            for j in 0..8 {
                for i in 0..6 {
                    texture_atlas.add_texture(biome_texture_rects[&(i, j)]);
                }
            }
            (biome, texture_atlas_assets.add(texture_atlas))
        })
        .collect();

    let structure_asset_list = structure_asset_list
        .get(&params_asset_collection.structures)
        .unwrap();
    let structures = StructureKind::iter()
        .filter(|structure| !matches!(structure, StructureKind::None | StructureKind::Occupied))
        .map(|structure| {
            let image = structure_textures.get(structure);
            let attrs = &structure_asset_list.0[&structure];
            let texture_atlas = TextureAtlas::from_grid(
                image,
                Vec2::new(attrs.width as _, attrs.height as _),
                attrs.columns,
                attrs.rows,
                None,
                None,
            );

            (structure, texture_atlas_assets.add(texture_atlas))
        })
        .collect();

    let mut params = params_asset
        .get(&params_asset_collection.params)
        .unwrap()
        .clone()
        .0;
    params.biomes = biome_asset_list.0.clone();
    params.structures = structure_asset_list.0.clone();

    command.insert_resource(params);
    command.insert_resource(TextureAtlasMaps { biomes, structures });
}
