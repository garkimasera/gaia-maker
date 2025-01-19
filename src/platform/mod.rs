use anyhow::{anyhow, Result};

const CONF_FILE_NAME: &str = "conf.ron";

#[cfg(not(target_arch = "wasm32"))]
static DATA_DIR: std::sync::LazyLock<Option<std::path::PathBuf>> =
    std::sync::LazyLock::new(find_data_dir);

#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir() -> Option<&'static std::path::Path> {
    DATA_DIR.as_ref().map(|path| path.as_ref())
}

#[cfg(all(not(target_arch = "wasm32"), feature = "asset_tar"))]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    data_dir()
        .map(|data_dir| data_dir.join("addons"))
        .into_iter()
        .collect()
}

#[cfg(all(target_arch = "wasm32", feature = "asset_tar"))]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    Vec::new()
}

#[cfg(not(target_arch = "wasm32"))]
fn find_data_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|path| path.join(env!("CARGO_PKG_NAME")))
}

#[cfg(not(target_arch = "wasm32"))]
fn conf_file() -> Option<std::path::PathBuf> {
    crate::platform::data_dir().map(|data_dir| data_dir.join(CONF_FILE_NAME))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn conf_load() -> Result<crate::conf::Conf> {
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let conf = ron::from_str(&std::fs::read_to_string(conf_file_path)?)?;
    Ok(conf)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn conf_save(conf: &crate::conf::Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    std::fs::write(conf_file_path, s)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn conf_load() -> Result<crate::conf::Conf> {
    let s = get_web_storage()?
        .get_item(CONF_FILE_NAME)
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    let conf = ron::from_str(&s)?;
    Ok(conf)
}

#[cfg(target_arch = "wasm32")]
pub fn conf_save(conf: &crate::conf::Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    get_web_storage()?
        .set_item(CONF_FILE_NAME, &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn savefile_write(file_name: &str, data: &[u8]) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn savefile_read(file_name: &str) -> Result<Vec<u8>> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    Ok(std::fs::read(save_dir_path.join(file_name))?)
}

#[cfg(target_arch = "wasm32")]
pub fn savefile_write(file_name: &str, data: &[u8]) -> Result<()> {
    use std::io::Write;

    let base64_encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    let mut encoder = flate2::write::GzEncoder::new(base64_encoder, flate2::Compression::best());
    encoder.write_all(data)?;

    let s = encoder.finish()?.into_inner();

    log::info!("save {} bytes to local storage", s.len());

    get_web_storage()?
        .set_item(&format!("save/{}", file_name), &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn savefile_read(file_name: &str) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let s = get_web_storage()?
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

#[cfg(target_arch = "wasm32")]
fn get_web_storage() -> Result<web_sys::Storage> {
    let window = web_sys::window().ok_or_else(|| anyhow!("cannot get Window"))?;
    let storage = window
        .local_storage()
        .map_err(|e| anyhow!("cannot get local_storage {:?}", e))?
        .ok_or_else(|| anyhow!("cannot get local_storage"))?;
    Ok(storage)
}
