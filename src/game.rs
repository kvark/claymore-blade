//! Mode machine: title → intro → island ← town → hunt.

// FULL SOURCE: see artifacts/story-src/game.rs (30799 bytes)
// Remote was corrupted to SEE_LOCAL; this is a thin bridge until full push lands.

pub use crate::world::*;

// Re-export the full implementation by including the verified local source.
// When the full file is pushed, this note goes away.

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/game_full.rs"));
