use anyhow::{anyhow, Result};
#[cfg(not(target_arch = "wasm32"))]
use once_cell::sync::Lazy;

use crate::planet::Planet;

#[cfg(not(target_arch = "wasm32"))]
static DATA_DIR: Lazy<Option<std::path::PathBuf>> =
    Lazy::new(|| dirs::data_dir().map(|path| path.join(env!("CARGO_PKG_NAME"))));

pub fn save_to(file_name: &str, planet: &Planet) -> Result<()> {
    let planet_data = bincode::serialize(planet)?;

    log::info!("save to {}", file_name);
    write(file_name, &planet_data)?;

    Ok(())
}

pub fn load_from(file_name: &str) -> Result<Planet> {
    log::info!("load from {}", file_name);
    let planet_data = read(file_name)?;
    Ok(bincode::deserialize(&planet_data)?)
}

#[cfg(not(target_arch = "wasm32"))]
fn write(file_name: &str, data: &[u8]) -> Result<()> {
    let data_dir = DATA_DIR
        .as_ref()
        .ok_or_else(|| anyhow!("cannot get data directory"))?;
    let save_dir_path = data_dir.join("save");
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read(file_name: &str) -> Result<Vec<u8>> {
    let data_dir = DATA_DIR
        .as_ref()
        .ok_or_else(|| anyhow!("cannot get data directory"))?;
    let save_dir_path = data_dir.join("save");
    Ok(std::fs::read(save_dir_path.join(file_name))?)
}

#[cfg(target_arch = "wasm32")]
fn write(file_name: &str, data: &[u8]) -> Result<()> {
    use std::io::Write;

    let window = web_sys::window().ok_or_else(|| anyhow!("cannot get Window"))?;
    let storage = window
        .local_storage()
        .map_err(|e| anyhow!("cannot get local_storage {:?}", e))?
        .ok_or_else(|| anyhow!("cannot get local_storage"))?;

    let mut s = String::new();
    {
        let base64_encoder = base64::write::EncoderStringWriter::from_consumer(
            &mut s,
            &base64::engine::DEFAULT_ENGINE,
        );
        let mut encoder =
            flate2::write::GzEncoder::new(base64_encoder, flate2::Compression::best());
        encoder.write_all(data)?;
    }

    storage
        .set_item(&format!("save/{}", file_name), &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read(file_name: &str) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let window = web_sys::window().ok_or_else(|| anyhow!("cannot get Window"))?;
    let storage = window
        .local_storage()
        .map_err(|e| anyhow!("cannot get local_storage {:?}", e))?
        .ok_or_else(|| anyhow!("cannot get local_storage"))?;

    let s = storage
        .get_item(&format!("save/{}", file_name))
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    let mut s = Cursor::new(s);
    let base64_decoder = base64::read::DecoderReader::from(&mut s, &base64::engine::DEFAULT_ENGINE);
    let mut decoder = flate2::read::GzDecoder::new(base64_decoder);

    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;
    Ok(data)
}
