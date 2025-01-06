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
