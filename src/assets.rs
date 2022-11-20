use crate::planet::*;
use crate::GameState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use fnv::FnvHashMap;
use serde::Deserialize;
use std::collections::HashMap;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

#[derive(Clone, Copy, Debug)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<BiomeAssetList>::new(&["biomes.ron"]))
            .add_plugin(RonAssetPlugin::<StructureAssetList>::new(&[
                "structures.ron",
            ]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                    .continue_to_state(GameState::Running)
                    .with_collection::<UiTextures>()
                    .with_collection::<ParamsAssetCollection>()
                    .with_collection::<BiomeTextures>()
                    .with_collection::<StructureTextures>(),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::AssetLoading).with_system(create_assets_list),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, EnumIter, EnumString, AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum UiTexture {
    IconBranch,
    IconBuild,
    IconMaterial,
    IconMessage,
    IconPower,
    TileColored,
    TileCursor,
}

define_asset_list_from_enum! {
    #[asset(dir_path = "ui")]
    #[asset(extension = "png")]
    pub struct UiTextures {
        pub textures: HashMap<UiTexture, Handle<Image>>,
    }
}

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "99d5021f-98fb-4873-b16a-bd9619b8b074"]
pub struct BiomeAssetList(FnvHashMap<Biome, BiomeAttrs>);

#[derive(Clone, Debug, Deserialize, TypeUuid)]
#[serde(transparent)]
#[uuid = "801a2daa-956d-469a-8e83-3610fbca21fd"]
pub struct StructureAssetList(FnvHashMap<StructureKind, StructureAttrs>);

pub struct TextureAtlasMaps {
    pub biomes: FnvHashMap<Biome, Handle<TextureAtlas>>,
    pub structures: FnvHashMap<StructureKind, Handle<TextureAtlas>>,
}

#[derive(Debug, AssetCollection)]
pub struct ParamsAssetCollection {
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

fn create_assets_list(
    mut command: Commands,
    params_asset_collection: Res<ParamsAssetCollection>,
    biome_textures: Res<BiomeTextures>,
    structure_textures: Res<StructureTextures>,
    (biome_asset_list, structure_asset_list): (
        Res<Assets<BiomeAssetList>>,
        Res<Assets<StructureAssetList>>,
    ),
    mut texture_atlas_assets: ResMut<Assets<TextureAtlas>>,
) {
    let biome_asset_list = biome_asset_list
        .get(&params_asset_collection.biomes)
        .unwrap();
    let biomes = Biome::iter()
        .map(|biome| {
            let image = biome_textures.get(biome);
            let texture_atlas =
                TextureAtlas::from_grid(image, Vec2::new(PIECE_SIZE, PIECE_SIZE), 6, 4);
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
            );

            (structure, texture_atlas_assets.add(texture_atlas))
        })
        .collect();

    command.insert_resource(Params {
        biomes: biome_asset_list.0.clone(),
        structures: structure_asset_list.0.clone(),
    });
    command.insert_resource(TextureAtlasMaps { biomes, structures });
}
