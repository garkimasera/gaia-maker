use bevy::asset::{AddAsset, AssetLoader, BoxedFuture, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use flate2::read::GzDecoder;
use std::io::Read;

pub struct GzPlugin;

#[derive(Clone, PartialEq, Eq, Debug, TypeUuid, Reflect)]
#[uuid = "87515d55-ca3b-4de1-86f8-a31f8bf0581c"]
pub struct GunzipBin(pub Vec<u8>);

impl Plugin for GzPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<GunzipBin>().add_asset_loader(GzLoader);
    }
}

struct GzLoader;

impl AssetLoader for GzLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let mut gz = GzDecoder::new(bytes);
            let mut decoded = Vec::new();
            gz.read_to_end(&mut decoded)?;
            load_context.set_default_asset(LoadedAsset::new(GunzipBin(decoded)));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gz"]
    }
}
