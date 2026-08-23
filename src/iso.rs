//! True isometric (30°) of a Y-up world.

use crate::hex::{axial_to_world, world_to_axial, Axial};
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

/// Rotate world XZ by `yaw` 90° steps (0..=3) around Y. Used for discrete combat camera.
pub fn rotate_yaw(x: f32, z: f32, yaw: u8) -> (f32, f32) {
    match yaw % 4 {
        1 => (z, -x),  // 90° CW
        2 => (-x, -z), // 180°
        3 => (-z, x),  // 270°
        _ => (x, z),
    }
}

/// Inverse of `rotate_yaw` (same as rotate by 4 - yaw).
pub fn rotate_yaw_inv(x: f32, z: f32, yaw: u8) -> (f32, f32) {
    rotate_yaw(x, z, (4 - (yaw % 4)) % 4)
}

pub fn axial_to_world_yaw(h: Axial, size: f32, yaw: u8) -> (f32, f32) {
    let (x, z) = axial_to_world(h, size);
    rotate_yaw(x, z, yaw)
}

pub fn terrain_height(kind: crate::combat::Terrain, size: f32) -> f32 {
    match kind {
        crate::combat::Terrain::Ruin => size * 0.55,
        crate::combat::Terrain::Water => size * -0.28,
        crate::combat::Terrain::Mud => size * 0.06,
        crate::combat::Terrain::Grass => size * 0.12,
    }
}

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

pub fn board_size(cols: i32, rows: i32, w: f32, h: f32) -> f32 {
    let fit = (w / (cols as f32 * 1.45 + 1.0)).min(h / (rows as f32 * 0.72 + 1.9));
    fit.clamp(24.0, 62.0)
}

pub fn camera_origin(
    cols: i32,
    rows: i32,
    size: f32,
    w: f32,
    h: f32,
    pan: [f32; 2],
    zoom: f32,
    yaw: u8,
) -> (f32, f32) {
    let mid = axial_to_world_yaw(Axial::new((cols - 1) / 2, (rows - 1) / 2), size, yaw);
    let (cx, cy) = world_to_iso(mid.0, 0.0, mid.1);
    (w / 2.0 - cx * zoom + pan[0], h * 0.40 - cy * zoom + pan[1])
}

pub fn hex_to_screen(
    hex: Axial,
    w: f32,
    h: f32,
    cols: i32,
    rows: i32,
    pan: [f32; 2],
    zoom: f32,
    height: f32,
    yaw: u8,
) -> [f32; 2] {
    let size = board_size(cols, rows, w, h);
    let (ox, oy) = camera_origin(cols, rows, size, w, h, pan, zoom, yaw);
    let (wx, wz) = axial_to_world_yaw(hex, size, yaw);
    let (sx, sy) = world_to_iso(wx, height, wz);
    [
        (ox + sx * zoom) / w,
        (oy + sy * zoom) / h,
    ]
}

pub fn pick_screen(
    mx: f32,
    my: f32,
    w: f32,
    h: f32,
    cols: i32,
    rows: i32,
    pan: [f32; 2],
    zoom: f32,
    yaw: u8,
) -> Option<Axial> {
    let size = board_size(cols, rows, w, h);
    let (ox, oy) = camera_origin(cols, rows, size, w, h, pan, zoom, yaw);
    let sx = (mx - ox) / zoom;
    let sy = (my - oy) / zoom;
    let (x, z) = iso_to_world(sx, sy, size * 0.14);
    // Undo camera yaw so axial math stays in the original lattice.
    let (x, z) = rotate_yaw_inv(x, z, yaw);
    let hex = world_to_axial(x, z, size);
    if hex.q < 0 || hex.r < 0 || hex.q >= cols || hex.r >= rows {
        None
    } else {
        Some(hex)
    }
}

/// GPU camera looking at the board (narrow FOV ≈ ortho).
pub fn blade_view_proj(center: Axial, size: f32, aspect: f32) -> Mat4 {
    let (pos, rot) = hunt_camera(center, size, size * 18.0);
    hunt_view_proj(pos, rot, aspect, size * 40.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_roundtrip() {
        let (sx, sy) = world_to_iso(3.0, 1.0, 2.0);
        let (x, z) = iso_to_world(sx, sy, 1.0);
        assert!((x - 3.0).abs() < 1e-4);
        assert!((z - 2.0).abs() < 1e-4);
    }

    #[test]
    fn hex_to_screen_is_on_canvas() {
        let p = hex_to_screen(Axial::new(5, 4), 1280.0, 800.0, 11, 8, [0.0, 0.0], 1.0, 8.0, 0);
        assert!(p[0] > 0.1 && p[0] < 0.9);
        assert!(p[1] > 0.1 && p[1] < 0.9);
    }

    #[test]
    fn yaw_roundtrip() {
        let (x, z) = (3.0f32, -1.5);
        for y in 0u8..4 {
            let (rx, rz) = rotate_yaw(x, z, y);
            let (bx, bz) = rotate_yaw_inv(rx, rz, y);
            assert!((bx - x).abs() < 1e-5);
            assert!((bz - z).abs() < 1e-5);
        }
    }
}
