use anyhow::{anyhow, Result};

use crate::saveload::SavedTime;

use super::DEFAULT_WINDOW_SIZE;

#[cfg(feature = "asset_tar")]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    Vec::new()
}

pub fn read_data_file(file_name: &str) -> Result<String> {
    let s = get_web_storage()?
        .get_item(file_name)
        .map_err(|e| anyhow!("getItem failed: {:?}", e))?
        .ok_or_else(|| anyhow!("getItem failed"))?;
    Ok(s)
}

pub fn write_data_file(file_name: &str, content: &str) -> Result<()> {
    get_web_storage()?
        .set_item(file_name, &content)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;
    Ok(())
}

pub fn write_savefile(dir_name: &str, file_name: &str, data: &[u8]) -> Result<()> {
    use std::io::Write;

    let base64_encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    let mut encoder = flate2::write::GzEncoder::new(base64_encoder, flate2::Compression::best());
    encoder.write_all(data)?;

    let s = encoder.finish()?.into_inner();

    log::info!("save {} bytes to local storage", s.len());

    get_web_storage()?
        .set_item(&format!("saves/{}/{}", dir_name, file_name), &s)
        .map_err(|e| anyhow!("setItem failed: {:?}", e))?;

    Ok(())
}

pub fn read_savefile(dir_name: &str, file_name: &str) -> Result<Vec<u8>> {
    use std::io::{Cursor, Read};

    let s = get_web_storage()?
        .get_item(&format!("saves/{}/{}", dir_name, file_name))
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

pub fn save_sub_dirs() -> Result<Vec<(SavedTime, String)>> {
    let web_storage = get_web_storage()?;
    let len = web_storage
        .length()
        .map_err(|e| anyhow!("length failed: {:?}", e))?;
    let mut sub_dirs = std::collections::BTreeSet::new();

    for i in 0..len {
        let key = web_storage
            .key(i)
            .map_err(|e| anyhow!("key failed: {:?}", e))?
            .unwrap();
        if let Some(s) = key.strip_prefix("saves/") {
            log::info!("{}", s);
            if let Some((sub_dir_name, _)) = s.split_once('/') {
                sub_dirs.insert(sub_dir_name.to_string());
            }
        }
    }

    // Use SavedTime::now for dir modified time because the number of sub dirs is limited to one in wasm
    Ok(sub_dirs
        .into_iter()
        .map(|sub_dir_name| (SavedTime::now(), sub_dir_name))
        .collect())
}

pub fn save_sub_dir_files(dir_name: &str) -> Result<Vec<String>> {
    let web_storage = get_web_storage()?;
    let len = web_storage
        .length()
        .map_err(|e| anyhow!("length failed: {:?}", e))?;
    let dir_path = format!("saves/{}/", dir_name);
    let mut files = Vec::new();

    for i in 0..len {
        let key = web_storage
            .key(i)
            .map_err(|e| anyhow!("key failed: {:?}", e))?
            .unwrap();
        if let Some(file_name) = key.strip_prefix(&dir_path) {
            files.push(file_name.to_owned());
        }
    }

    Ok(files)
}

fn get_web_storage() -> Result<web_sys::Storage> {
    let window = web_sys::window().ok_or_else(|| anyhow!("cannot get Window"))?;
    let storage = window
        .local_storage()
        .map_err(|e| anyhow!("cannot get local_storage {:?}", e))?
        .ok_or_else(|| anyhow!("cannot get local_storage"))?;
    Ok(storage)
}

pub fn preferred_window_size() -> (u32, u32) {
    let Some(w) = web_sys::window() else {
        return DEFAULT_WINDOW_SIZE;
    };

    let Some(width) = w.inner_width().ok().and_then(|width| width.as_f64()) else {
        return DEFAULT_WINDOW_SIZE;
    };
    let Some(height) = w.inner_height().ok().and_then(|height| height.as_f64()) else {
        return DEFAULT_WINDOW_SIZE;
    };
    let width = width as u32;
    let height = height as u32;

    (width, height)
}

pub fn window_open() {
    set_element_display("game-screen", "block");
    set_element_display("start-screen", "none");
}

pub fn window_close() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(location) = document.location() else {
        return;
    };
    let _ = location.reload();
}

fn set_element_display(id: &str, value: &str) {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Some(element) = document.get_element_by_id(id) else {
        return;
    };
    let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() else {
        return;
    };
    if let Err(e) = element.style().set_property("display", value) {
        log::warn!("{:?}", e);
    }
}

pub fn window_resize(
    mut window: bevy::prelude::Query<
        &mut bevy::window::Window,
        bevy::prelude::With<bevy::window::PrimaryWindow>,
    >,
) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    let Some(w) = web_sys::window() else {
        return;
    };

    let Some(width) = w
        .inner_width()
        .ok()
        .and_then(|width| width.as_f64())
        .map(|width| width as u32)
    else {
        return;
    };
    let Some(height) = w
        .inner_height()
        .ok()
        .and_then(|height| height.as_f64())
        .map(|height| height as u32)
    else {
        return;
    };

    // Adjust target size to prevent pixel blurring
    let target_width = width - width % 2;
    let target_height = height - height % 2;

    if window.width() as u32 != target_width || window.height() as u32 != target_height {
        window
            .resolution
            .set(target_width as f32, target_height as f32);
    }
}
