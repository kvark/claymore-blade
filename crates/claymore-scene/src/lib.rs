//! Hunt presentation math. Blade-render instances the prism; the web canvas
//! projects the same vertices.

pub mod iso;
pub mod prism;

pub use iso::{hunt_camera, hunt_view_proj, pick_hex, terrain_height, world_to_iso};
pub use prism::{tile_instance, unit_hex_prism, Mesh, TileInstance, Vertex};
