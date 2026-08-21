//! Claymore — one crate. Hex hunt, Blade graphics, native and web.

#![allow(irrefutable_let_patterns)]

pub mod app;
pub mod audio;
pub mod catalog;
pub mod combat;
pub mod dialog;
pub mod font;
pub mod fx;
pub mod game;
pub mod gpu;
pub mod hex;
pub mod hud;
pub mod io;
pub mod iso;
pub mod pipe_enc;
pub mod prism;
pub mod rng;
pub mod world;

pub use app::run;
pub use hex::Axial;

#[cfg(target_arch = "wasm32")]
mod web_start {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn start() {
        crate::run();
    }
}
