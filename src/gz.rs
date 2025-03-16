use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use flate2::read::GzDecoder;
use std::io::Read;

pub struct GzPlugin;

#[derive(Clone, PartialEq, Eq, Debug, Asset, TypePath)]
pub struct GunzipBin(pub Vec<u8>);

impl Plugin for GzPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<GunzipBin>().register_asset_loader(GzLoader);
    }
}

struct GzLoader;

impl AssetLoader for GzLoader {
    type Asset = GunzipBin;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let mut gz = GzDecoder::new(&bytes[..]);
        let mut decoded = Vec::new();
        gz.read_to_end(&mut decoded)?;

        Ok(GunzipBin(decoded))
    }

    fn extensions(&self) -> &[&str] {
        &["gz"]
    }
}
