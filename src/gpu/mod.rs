//! Blade-graphics hunt view. Same path on Vulkan/Metal/GLES and WebGL2.

#![allow(irrefutable_let_patterns)]

use crate::catalog::{ENCOUNTERS, ENEMIES, LOCATIONS, WARRIORS};
use crate::combat::{core_hex, current_unit, live_cells, CombatState, Side, Terrain};
use crate::font;
use crate::game::{Game, Mode};
use crate::hex::axial_to_world;
use crate::hud;
use crate::io::{load_rgba, shader_source};
use crate::iso::{
    axial_to_world_yaw, board_size, camera_origin, rotate_yaw, terrain_height, world_to_iso,
};
use crate::prism::unit_hex_prism;
use crate::world::clock_label;
use blade_graphics as gpu;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;


const ASH: [f32; 4] = [0.72, 0.64, 0.52, 1.0];
const BLOOD: [f32; 4] = [0.72, 0.18, 0.14, 1.0];

const KENNEY: &[&str] = &[
    "kenney/ui/button.png",
    "kenney/ui/button-line.png",
    "kenney/ui/button-red.png",
    "kenney/ui/button-brown.png",
    "kenney/ui/button-grey.png",
    "kenney/ui/bar.png",
    "kenney/ui/bar-fill.png",
    "kenney/ui/panel.png",
    "kenney/ui/panel-brown.png",
    "kenney/ui/banner.png",
    "kenney/ui/hex.png",
    "kenney/ui/hex-move.png",
    "kenney/ui/hex-hit.png",
    "kenney/ui/divider.png",
    "kenney/fx/slash.png",
    "kenney/fx/slash2.png",
    "kenney/fx/spark.png",
    "kenney/fx/hit.png",
    "kenney/fx/smoke.png",
    "kenney/prompt/1.png",
    "kenney/prompt/2.png",
    "kenney/prompt/3.png",
    "kenney/prompt/space.png",
    "kenney/prompt/esc.png",
    "kenney/prompt/w.png",
    "kenney/prompt/a.png",
    "kenney/prompt/s.png",
    "kenney/prompt/d.png",
    "kenney/iso/column.png",
    "kenney/cursor/pointer.png",
];
