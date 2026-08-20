//! Turn-based hex combat. Native and wasm run this same code.

mod ai;
mod resolve;
mod rules;
mod setup;
mod types;

pub use ai::run_ai;
pub use resolve::act;
pub use rules::{can_use, legal_moves, legal_targets, zone_for};
pub use setup::{core_hex, create_battle, current_unit, effect_scale, live_cells, living};
pub use types::*;

pub(crate) use setup::{
    advance_turn, begin_turn, check_over, current_unit_mut, in_bounds, occupied, push_log, spawn,
    terrain_at, tick_ripples,
};
