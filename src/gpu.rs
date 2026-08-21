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

// PLACEHOLDER_REST
