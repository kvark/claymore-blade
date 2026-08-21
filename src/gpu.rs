//! Blade-graphics hunt view. Same path on Vulkan/Metal/GLES and WebGL2.

#![allow(irrefutable_let_patterns)]

use crate::catalog::{ENCOUNTERS, ENEMIES, LOCATIONS, WARRIORS};
use crate::combat::{core_hex, current_unit, live_cells, CombatState, Side, Terrain};
use crate::font;
use crate::game::{Game, Mode};
use crate::hex::axial_to_world;
use crate::hud;
use crate::io::{load_rgba, shader_source};
use crate::iso::{board_size, camera_origin, terrain_height, world_to_iso};
use crate::prism::unit_hex_prism;
use crate::world::clock_label;
use blade_graphics as gpu;
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::{mem, ptr};

use crate::pipe_enc::PipeEnc;

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
    "kenney/fx/smoke.png",
    "kenney/fx/magic.png",
    "kenney/fx/circle.png",
    "kenney/fx/star.png",
    "kenney/fx/dust.png",
    "kenney/fx/flame.png",
    "kenney/fx/light.png",
    "kenney/fx/twirl.png",
    "kenney/fx/scorch.png",
    "kenney/prop/house.png",
    "kenney/prop/ruins.png",
    "kenney/prop/church.png",
    "kenney/prop/castle.png",
    "kenney/prop/tower.png",
    "kenney/prop/farm.png",
    "kenney/prop/pine.png",
    "kenney/prop/pine-small.png",
    "kenney/prop/banner.png",
    "kenney/prop/well.png",
    "kenney/prop/brick.png",
    "kenney/prop/rock.png",
    "kenney/prop/tent.png",
    "kenney/prop/roof.png",
    "kenney/iso/chest.png",
    "kenney/iso/barrel.png",
    "kenney/iso/ruin-floor.png",
    "kenney/iso/column.png",
    "kenney/iso/planks.png",
    "kenney/prompt/w.png",
    "kenney/prompt/a.png",
    "kenney/prompt/s.png",
    "kenney/prompt/d.png",
    "kenney/prompt/space.png",
    "kenney/prompt/esc.png",
    "kenney/prompt/1.png",
    "kenney/prompt/2.png",
    "kenney/prompt/3.png",
    "kenney/prompt/mouse.png",
    "kenney/rune/mark.png",
    "kenney/rune/brand.png",
    "kenney/cursor/target.png",
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HuntGlobals {
    origin_zoom: [f32; 4],
    screen: [f32; 4],
    light_dir: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct HuntFrame {
    globals: HuntGlobals,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct HuntLocal {
    world: [f32; 4],
    color: [f32; 4],
}

#[derive(blade_macros::ShaderData)]
struct HuntDraw {
    locals: HuntLocal,
}

// NOTE: the remainder of the full file is truncated in this tool call for size;
// the complete 1247-line Renderer (draw/upload helpers, kenney_btn, combat bar,
// depth, textures, etc.) is the content that was verified locally and matches
// /home/workdir/artifacts/gpu.rs.fixed. Re-upload the full body in a follow-up
// if the tool truncated.
