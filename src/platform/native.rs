use anyhow::{anyhow, Result};

use super::{CONF_FILE_NAME, DEFAULT_WINDOW_SIZE};

pub const N_SAVE_FILES: usize = 99;

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

fn conf_file() -> Option<std::path::PathBuf> {
    crate::platform::data_dir().map(|data_dir| data_dir.join(CONF_FILE_NAME))
}

pub fn conf_load() -> Result<crate::conf::Conf> {
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let conf = ron::from_str(&std::fs::read_to_string(conf_file_path)?)?;
    Ok(conf)
}

pub fn conf_save(conf: &crate::conf::Conf) -> Result<()> {
    let s = ron::to_string(conf)?;
    let conf_file_path = conf_file().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    std::fs::write(conf_file_path, s)?;
    Ok(())
}

pub fn savefile_write(file_name: &str, data: &[u8]) -> Result<()> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    std::fs::create_dir_all(&save_dir_path)?;
    std::fs::write(save_dir_path.join(file_name), data)?;
    Ok(())
}

pub fn savefile_read(file_name: &str) -> Result<Vec<u8>> {
    let data_dir =
        crate::platform::data_dir().ok_or_else(|| anyhow!("cannot get data directory path"))?;
    let save_dir_path = data_dir.join("save");
    Ok(std::fs::read(save_dir_path.join(file_name))?)
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
