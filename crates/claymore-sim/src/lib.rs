//! Portable Claymore rules. Blade presents; this crate ticks.

pub mod combat;
pub mod hex;
pub mod rng;

pub use combat::{damage_of, effect_scale, resolve_hit, Footprint, HitKind};
pub use hex::Axial;
pub use rng::Rng;
