use anyhow::{anyhow, Result};

use crate::planet::Planet;

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
    let data_dir =
        crate::conf::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read(file_name: &str) -> Result<Vec<u8>> {
    let data_dir =
        crate::conf::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    Ok(std::fs::read(save_dir_path.join(file_name))?)
}

#[cfg(target_arch = "wasm32")]
fn write(file_name: &str, data: &[u8]) -> Result<()> {
    use std::io::Write;

    let base64_encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    let mut encoder = flate2::write::GzEncoder::new(base64_encoder, flate2::Compression::best());
    encoder.write_all(data)?;

    let s = encoder.finish()?.into_inner();

    log::info!("save {} bytes to local storage", s.len());

    crate::conf::get_storage()?
        .set_item(&format!("save/{}", file_name), &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn read(file_name: &str) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let s = crate::conf::get_storage()?
        .get_item(&format!("save/{}", file_name))
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    let mut s = Cursor::new(s);
    let base64_decoder =
        base64::read::DecoderReader::new(&mut s, &base64::engine::general_purpose::STANDARD);
    let mut decoder = flate2::read::GzDecoder::new(base64_decoder);

    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;
    Ok(data)
}
