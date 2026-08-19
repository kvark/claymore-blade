//! Hit table and scale. Full action application still lives in the TS slice
//! (`src/game/sim/combat.ts`) and will move here once the wasm hunt view owns input.

use crate::hex::Axial;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HitKind {
    Miss,
    Glance,
    Blocked,
    Solid,
}

/// `1.0 + 0.25 * clamp(Pa - Pd, -4, 8)`
pub fn effect_scale(pa: i32, pd: i32) -> f32 {
    let d = (pa - pd).clamp(-4, 8);
    1.0 + 0.25 * d as f32
}

pub fn resolve_hit(roll: f32, hit: i32, dodge: i32, guarded: bool) -> HitKind {
    let margin = hit as f32 - dodge as f32 + (roll - 0.5) * 8.0;
    if margin < -3.0 {
        HitKind::Miss
    } else if margin < 0.0 {
        HitKind::Glance
    } else if guarded {
        HitKind::Blocked
    } else {
        HitKind::Solid
    }
}

pub fn damage_of(kind: HitKind, power: i32, scale: f32) -> i32 {
    let frac = match kind {
        HitKind::Miss => 0.0,
        HitKind::Glance => 0.2,
        HitKind::Blocked => 0.15,
        HitKind::Solid => 1.0,
    };
    (power as f32 * scale * frac).round() as i32
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Footprint {
    pub origin: Axial,
    pub facing: i32,
    pub cells: Vec<Axial>,
    pub core: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_table() {
        assert!((effect_scale(0, 0) - 1.0).abs() < 1e-5);
        assert!((effect_scale(4, 0) - 2.0).abs() < 1e-5);
        assert!((effect_scale(0, 4) - 0.0).abs() < 1e-5);
    }
}
