//! Hunt view. CPU board is always available; Blade raster plan is `gpu`.

pub mod board;
pub mod gpu;

pub use board::HuntBoard;
pub use gpu::{blade_camera, HuntGpuPlan};

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn wasm_start() {
        console_error_panic_hook::set_once();
    }
}
