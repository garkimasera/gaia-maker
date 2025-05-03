pub mod client;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PreferredWindowResolution {
    Size(u32, u32),
    #[allow(unused)]
    Maximized,
}

impl Default for PreferredWindowResolution {
    fn default() -> Self {
        Self::Size(1600, 900)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
