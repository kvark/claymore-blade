//! True isometric (30°) of a Y-up world. Matches `src/game/sim/hex.ts`.

use claymore_sim::hex::{axial_to_world, world_to_axial, Axial};
use glam::{Mat4, Quat, Vec3};

pub const ISO_COS: f32 = 0.866_025_4;
pub const ISO_SIN: f32 = 0.5;

pub fn world_to_iso(x: f32, y: f32, z: f32) -> (f32, f32) {
    ((x - z) * ISO_COS, (x + z) * ISO_SIN - y)
}

pub fn iso_to_world(sx: f32, sy: f32, y: f32) -> (f32, f32) {
    let a = sx / ISO_COS;
    let b = (sy + y) / ISO_SIN;
    ((a + b) * 0.5, (b - a) * 0.5)
}

pub fn terrain_height(kind: &str, size: f32) -> f32 {
    match kind {
        "ruin" => size * 0.42,
        "water" => size * -0.22,
        "mud" => size * 0.04,
        _ => size * 0.14,
    }
}

/// Camera sitting above the board, looking toward +X+Z, small FOV ≈ ortho.
pub fn hunt_camera(center: Axial, size: f32, distance: f32) -> (Vec3, Quat) {
    let (cx, cz) = axial_to_world(center, size);
    let target = Vec3::new(cx, 0.0, cz);
    let offset = Vec3::new(-1.0, 1.15, -1.0).normalize() * distance;
    let pos = target + offset;
    let rot = Quat::from_rotation_arc(Vec3::NEG_Z, (target - pos).normalize());
    (pos, rot)
}

pub fn hunt_view_proj(pos: Vec3, rot: Quat, aspect: f32, depth: f32) -> Mat4 {
    let view = Mat4::from_rotation_translation(rot, pos).inverse();
    let proj = Mat4::perspective_rh(0.28, aspect.max(0.1), 0.5, depth);
    proj * view
}

pub fn pick_hex(sx: f32, sy: f32, size: f32) -> Axial {
    let (x, z) = iso_to_world(sx, sy, size * 0.14);
    world_to_axial(x, z, size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claymore_sim::hex::Axial;

    #[test]
    fn iso_roundtrip() {
        let (sx, sy) = world_to_iso(3.0, 1.0, 2.0);
        let (x, z) = iso_to_world(sx, sy, 1.0);
        assert!((x - 3.0).abs() < 1e-4);
        assert!((z - 2.0).abs() < 1e-4);
    }

    #[test]
    fn camera_looks_at_board() {
        let (pos, _rot) = hunt_camera(Axial::new(5, 4), 1.0, 20.0);
        assert!(pos.y > 0.0);
    }
}
