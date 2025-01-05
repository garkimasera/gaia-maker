use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use image::GenericImage;

const MONOCHROME_IMAGE_DIRS: &[&str] = &["animals", "biomes", "structures", "tile_animations"];

#[derive(Clone, Copy, Debug)]
pub struct ImageAssetsPlugin;

impl Plugin for ImageAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(ImageLoader);
    }
}

struct ImageLoader;

impl AssetLoader for ImageLoader {
    type Asset = Image;
    type Settings = ();
    type Error = anyhow::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let image =
            image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)?.into_rgba8();

        let path = load_context.path();
        let image = if MONOCHROME_IMAGE_DIRS
            .iter()
            .any(|dir| path.starts_with(dir))
        {
            let width = image.width();
            let height = image.height();
            let mut new_image = image::RgbaImage::new(width, height * 2);
            new_image.copy_from(&image, 0, 0)?;

            for y in 0..height {
                for x in 0..width {
                    let pixel = image.get_pixel(x, y);
                    let v = ((pixel.0[0] as u32 + pixel.0[1] as u32 + pixel.0[2] as u32) / 3) as u8;
                    new_image.put_pixel(x, height + y, image::Rgba([v, v, v, pixel.0[3]]));
                }
            }
            new_image
        } else {
            image
        };

        let width = image.width();
        let height = image.height();

        Ok(Image::new(
            bevy::render::render_resource::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            image.into_raw(),
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}
