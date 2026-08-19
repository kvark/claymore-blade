//! Claymore — one crate. Hex rules, isometric hunt scene, Blade raster plan.

pub mod board;
pub mod combat;
pub mod gpu;
pub mod hex;
pub mod iso;
pub mod prism;
pub mod rng;

pub use board::HuntBoard;
pub use combat::{damage_of, effect_scale, resolve_hit, Footprint, HitKind};
pub use gpu::{blade_camera, HuntGpuPlan};
pub use hex::Axial;
pub use iso::{hunt_camera, hunt_view_proj, pick_hex, terrain_height, world_to_iso};
pub use prism::{tile_instance, unit_hex_prism, Mesh, TileInstance, Vertex};
pub use rng::Rng;

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn wasm_start() {
        console_error_panic_hook::set_once();
    }
}
