#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

const DEFAULT_WINDOW_SIZE: (u32, u32) = (1280, 720);
const CONF_FILE_NAME: &str = "conf.ron";

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
