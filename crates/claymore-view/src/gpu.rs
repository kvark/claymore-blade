//! Blade-render raster path. Enable with `--features gpu`.
//!
//! `blade-render::Rasterizer` is the web-safe pipeline (no hardware RT).
//! `blade-graphics` on `wasm32` is WebGL2. Native uses Vulkan / Metal / GLES.
//!
//! Wiring (native or wasm, once a `gpu::Context` exists):
//!
//! ```ignore
//! let mut raster = blade_render::Rasterizer::new(
//!     encoder, &gpu, shaders, &asset_hub.shaders, &render_config,
//! );
//! raster.render(
//!     &mut pass,
//!     &camera,          // from claymore_scene::iso::hunt_camera
//!     &objects,         // instanced hex prisms + billboard units
//!     &asset_hub,
//!     environment_map,
//!     blade_render::RasterConfig {
//!         clear_color: gpu::TextureColor::OpaqueBlack,
//!         light_dir: [0.35, 0.8, 0.45].into(),
//!         light_color: [1.0, 0.95, 0.85].into(),
//!         ambient_color: [0.12, 0.11, 0.10].into(),
//!         space_sky: false,
//!     },
//! );
//! ```
//!
//! Engine shortcut: `blade_engine::config::RenderBackend::Rasterizer`.
//! Full-stack `blade-engine` still wants a native window + asset folders;
//! this crate keeps the hunt scene independent so the same board feeds
//! Rasterizer *or* the canvas preview.

use claymore_scene::iso::hunt_camera;
use claymore_scene::unit_hex_prism;
use claymore_sim::Axial;

use crate::board::HuntBoard;

pub const RASTER_BACKEND: &str = "blade-render::Rasterizer";
pub const WEB_API: &str = "WebGL2";

pub struct HuntGpuPlan {
    pub backend: &'static str,
    pub web_api: &'static str,
    pub camera_pos: [f32; 3],
    pub prism_vertices: usize,
    pub instances: usize,
}

impl HuntGpuPlan {
    pub fn from_board(board: &HuntBoard) -> Self {
        let (pos, _) = hunt_camera(board.center(), board.size, board.size * 18.0);
        let mesh = unit_hex_prism([1.0; 4]);
        Self {
            backend: RASTER_BACKEND,
            web_api: WEB_API,
            camera_pos: pos.into(),
            prism_vertices: mesh.vertices.len(),
            instances: board.tiles.len(),
        }
    }
}

/// Isometric Blade camera for a board. `fov_y` is narrow so it reads as ortho.
pub fn blade_camera(center: Axial, size: f32) -> ([f32; 3], [f32; 4], f32) {
    let (pos, rot) = hunt_camera(center, size, size * 18.0);
    (pos.into(), rot.into(), 0.28)
}
