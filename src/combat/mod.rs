//! Turn-based hex combat. Native and wasm run this same code.

mod ai;
mod resolve;
mod rules;
mod setup;
mod types;

pub use ai::{
    intended_strike, next_enemy_unit, peek_enemy_tell, run_ai, run_ai_prep, IntendedStrike,
};
pub use resolve::act;
pub use rules::{can_use, legal_moves, legal_targets, raki_can_help, zone_for};
pub use setup::{core_hex, create_battle, current_unit, effect_scale, live_cells, living};
pub use types::*;

pub(crate) use rules::move_cost;
pub(crate) use setup::{
    advance_turn, check_over, current_unit_mut, in_bounds, occupied, push_log,
    terrain_at,
};
