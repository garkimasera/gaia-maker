use anyhow::{anyhow, Context, Result};

use crate::saveload::SavedTime;

use super::DEFAULT_WINDOW_SIZE;

static DATA_DIR: std::sync::LazyLock<Option<std::path::PathBuf>> =
    std::sync::LazyLock::new(find_data_dir);

pub fn data_dir() -> Option<&'static std::path::Path> {
    DATA_DIR.as_ref().map(|path| path.as_ref())
}

#[cfg(feature = "asset_tar")]
pub fn addon_directory() -> Vec<std::path::PathBuf> {
    data_dir()
        .map(|data_dir| data_dir.join("addons"))
        .into_iter()
        .collect()
}

fn find_data_dir() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|path| path.join(env!("CARGO_PKG_NAME")))
}

/// Read file under the data directory
pub fn read_data_file(file_name: &str) -> Result<String> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    std::fs::read_to_string(data_dir.join(file_name)).with_context(|| format!("read {}", file_name))
}

/// Write string data to a file under the data directory
pub fn write_data_file(file_name: &str, content: &str) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    std::fs::create_dir_all(data_dir)?;
    std::fs::write(data_dir.join(file_name), content)?;
    Ok(())
}

pub fn write_savefile(dir_name: &str, file_name: &str, data: &[u8]) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("saves").join(dir_name);
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

pub fn read_savefile(dir_name: &str, file_name: &str) -> Result<Vec<u8>> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("saves").join(dir_name);
    let file_path = save_dir_path.join(file_name);
    std::fs::read(&file_path).with_context(|| format!("reading \"{}\"", file_path.display()))
}

pub fn delete_savefile(dir_name: &str, file_name: &str) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("saves").join(dir_name);
    let file_path = save_dir_path.join(file_name);
    std::fs::remove_file(&file_path)?;
    Ok(())
}

pub fn save_sub_dirs() -> Result<Vec<(SavedTime, String)>> {
    let saves_dir_path = crate::platform::data_dir()
        .ok_or_else(|| anyhow!("cannot get data directory path"))?
        .join("saves");
    let mut dirs = Vec::new();

    for entry in std::fs::read_dir(&saves_dir_path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                log::warn!("error in loading \"{}\": {}", saves_dir_path.display(), e);
                continue;
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(e) => {
                log::warn!("error in loading \"{}\": {}", saves_dir_path.display(), e);
                continue;
            }
        };
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                log::warn!("error in loading \"{}\": {}", saves_dir_path.display(), e);
                continue;
            }
        };
        if !file_type.is_dir() {
            continue;
        }
        let Some(sub_dir_name) = entry
            .path()
            .file_name()
            .map(|path| path.to_string_lossy().into_owned())
        else {
            continue;
        };
        let modified = if let Ok(modified) = metadata.modified() {
            SavedTime::from(modified)
        } else {
            SavedTime::now()
        };
        dirs.push((modified, sub_dir_name));
    }

    Ok(dirs)
}

pub fn save_sub_dir_files(dir_name: &str) -> Result<Vec<String>> {
    let dir_path = crate::platform::data_dir()
        .ok_or_else(|| anyhow!("cannot get data directory path"))?
        .join("saves")
        .join(dir_name);
    let mut files = Vec::new();

    for entry in std::fs::read_dir(&dir_path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                log::warn!("error in loading \"{}\": {}", dir_path.display(), e);
                continue;
            }
        };
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(e) => {
                log::warn!("error in loading \"{}\": {}", dir_path.display(), e);
                continue;
            }
        };
        if !file_type.is_file() {
            continue;
        }
        let Some(file_name) = entry
            .path()
            .file_name()
            .map(|path| path.to_string_lossy().into_owned())
        else {
            continue;
        };
        files.push(file_name);
    }

    Ok(files)
}

pub fn window_open() {}

pub fn window_close() {}

pub fn preferred_window_size() -> (u32, u32) {
    DEFAULT_WINDOW_SIZE
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
    let width = window.width() as u32;
    let height = window.height() as u32;

    // Adjust target size to prevent pixel blurring
    let target_width = width - width % 2;
    let target_height = height - height % 2;

    if window.width() as u32 != target_width || window.height() as u32 != target_height {
        window
            .resolution
            .set(target_width as f32, target_height as f32);
    }
}
