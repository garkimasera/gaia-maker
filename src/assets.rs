use crate::audio::SoundEffect;
use crate::conf::Conf;
use crate::gz::GunzipBin;
use crate::planet::*;
use crate::text::{Lang, TranslationText};
use crate::GameState;
use bevy::prelude::*;
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
                LoadingState::new(GameState::AssetLoading)
                    .continue_to_state(GameState::MainMenu)
                    .load_collection::<TranslationTexts>()
                    .load_collection::<UiTextures>()
                    .load_collection::<UiAssets>()
                    .load_collection::<ParamsAssetCollection>()
                    .load_collection::<BiomeTextures>()
                    .load_collection::<StructureTextures>()
                    .load_collection::<SoundEffects>(),
            )
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
    IconAction,
    IconAirTemprature,
    IconBiomass,
    IconBuild,
    IconCoordinates,
    IconFertility,
    IconGameMenu,
    IconHeight,
    IconHelp,
    IconLayers,
    IconMap,
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

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct ParamsAsset(Params);

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct BiomeAssetList(FnvHashMap<Biome, BiomeAttrs>);

#[derive(Clone, Debug, Deserialize, Asset, TypePath)]
#[serde(transparent)]
pub struct StructureAssetList(FnvHashMap<StructureKind, StructureAttrs>);

#[derive(Resource)]
pub struct TextureAtlasLayouts {
    pub biomes: FnvHashMap<Biome, Handle<TextureAtlasLayout>>,
    pub structures: FnvHashMap<StructureKind, Handle<TextureAtlasLayout>>,
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
    (params_asset, biome_asset_list, structure_asset_list): (
        Res<Assets<ParamsAsset>>,
        Res<Assets<BiomeAssetList>>,
        Res<Assets<StructureAssetList>>,
    ),
    mut texture_atlas_assets: ResMut<Assets<TextureAtlasLayout>>,
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
            let mut texture_atlas = TextureAtlasLayout::new_empty(Vec2::new(
                (TILE_SIZE + 2.0) * 3.0,
                (TILE_SIZE + 2.0) * 4.0,
            ));
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
            let attrs = &structure_asset_list.0[&structure];
            let texture_atlas = TextureAtlasLayout::from_grid(
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
    command.insert_resource(TextureAtlasLayouts { biomes, structures });
}
