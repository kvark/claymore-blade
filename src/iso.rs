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

/// Rectangle clear of the initiative strip and the combat action panel.
fn board_viewport(w: f32, h: f32) -> [f32; 4] {
    [w * 0.04, h * 0.15, w * 0.96, h * (crate::hud::combat_bar().wait.y - 0.05)]
}

/// Bounds of cell centers in unit-sized, yaw-rotated isometric space.
fn board_center_bounds(cols: i32, rows: i32, yaw: u8) -> [f32; 4] {
    let mut bounds = [f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY];
    for q in [0, cols - 1] {
        for r in [0, rows - 1] {
            let (x, z) = axial_to_world_yaw(Axial::new(q, r), 1.0, yaw);
            let (sx, sy) = world_to_iso(x, 0.0, z);
            bounds[0] = bounds[0].min(sx);
            bounds[1] = bounds[1].min(sy);
            bounds[2] = bounds[2].max(sx);
            bounds[3] = bounds[3].max(sy);
        }
    }
    bounds
}

/// Pixel bounds before camera translation, including fighter/portrait headroom.
fn board_frame_bounds(cols: i32, rows: i32, size: f32, w: f32, h: f32, zoom: f32, yaw: u8) -> [f32; 4] {
    let bounds = board_center_bounds(cols, rows, yaw);
    let s = size * zoom;
    // An animated fighter may reach one tile outward and three tiles up.
    // Also reserve screen-space headroom for portraits, labels, and effects.
    let side = (2.0 * ISO_COS * s).max(w * 0.04);
    let head = (4.0 * s).max(0.4 * s + h * 0.12);
    [bounds[0] * s - side, bounds[1] * s - head, bounds[2] * s + side, bounds[3] * s + s]
}

/// Fit the initial view at zoom 1.0. Use the worst yaw so rotation preserves
/// tile size, and allow small tiles on narrow screens rather than clipping.
pub fn board_size(cols: i32, rows: i32, w: f32, h: f32) -> f32 {
    let viewport = board_viewport(w, h);
    let width = viewport[2] - viewport[0];
    let height = viewport[3] - viewport[1];
    let mut fit = 62.0_f32;
    for yaw in 0..4 {
        let bounds = board_center_bounds(cols, rows, yaw);
        let span_x = bounds[2] - bounds[0];
        let span_y = bounds[3] - bounds[1];
        fit = fit.min(width / (span_x + 4.0 * ISO_COS));
        fit = fit.min((width - w * 0.08) / span_x.max(1.0));
        fit = fit.min(height / (span_y + 5.0));
        fit = fit.min((height - h * 0.12) / (span_y + 1.4));
    }
    // Small inset also keeps the default UI zoom (1.05 on Continue) in frame.
    (fit * 0.94).max(0.01)
}

/// Center and scale of the hunt shader's linear depth range.
///
/// The screen axes in `world_to_iso` are perpendicular to (1, 1, 1), so
/// increasing X + Y + Z moves toward the camera and must reduce depth.
/// `extent` bounds the absolute sum of camera-relative coordinates. Leave
/// room at both clip planes instead of clamping vertices (which flattens
/// triangles onto the same depth and makes the first draw occlude the rest).
pub fn depth_origin_scale(center: [f32; 3], extent: f32) -> [f32; 4] {
    [center[0], center[1], center[2], 0.45 / extent.max(1.0)]
}

pub fn board_ground_radius(cols: i32, rows: i32, size: f32) -> f32 {
    size * (cols.max(rows) as f32 * 0.72 + 1.4)
}

/// Depth bounds in the same yaw-rotated coordinates sent to the shader.
/// Independent of pan/zoom: reframing the board must not change occlusion.
pub fn board_depth_origin_scale(cols: i32, rows: i32, size: f32, yaw: u8) -> [f32; 4] {
    let mid = Axial::new((cols - 1) / 2, (rows - 1) / 2);
    let (cx, cz) = axial_to_world_yaw(mid, size, yaw);
    let mut extent = board_ground_radius(cols, rows, size) * std::f32::consts::SQRT_2;
    for q in [0, cols - 1] {
        for r in [0, rows - 1] {
            let (x, z) = axial_to_world_yaw(Axial::new(q, r), size, yaw);
            extent = extent.max(((x - cx) + (z - cz)).abs());
        }
    }
    // Covers prism radii, raised terrain, and animated fighters extending
    // two tiles horizontally and four tiles above any cell center.
    depth_origin_scale([cx, 0.0, cz], extent + size * 8.0)
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
    let bounds = board_frame_bounds(cols, rows, size, w, h, zoom, yaw);
    let viewport = board_viewport(w, h);
    (
        (viewport[0] + viewport[2] - bounds[0] - bounds[2]) * 0.5 + pan[0],
        (viewport[1] + viewport[3] - bounds[1] - bounds[3]) * 0.5 + pan[1],
    )
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

    // CPU mirror of hunt.wgsl, checked against the shader source below.
    fn depth_at(p: [f32; 3], params: [f32; 4]) -> f32 {
        0.5 - ((p[0] - params[0]) + (p[1] - params[1]) + (p[2] - params[2])) * params[3]
    }

    fn assert_inside_depth_range(p: [f32; 3], params: [f32; 4]) {
        let depth = depth_at(p, params);
        assert!(
            depth > 0.01 && depth < 0.99,
            "vertex {p:?} hit a clip plane: {depth}, depth parameters {params:?}"
        );
    }

    #[test]
    fn depth_is_linear_along_the_isometric_view_ray() {
        let params = depth_origin_scale([1000.0, 0.0, 800.0], 2000.0);
        let far = [1100.0, 50.0, 900.0];
        let near = [1140.0, 90.0, 940.0];
        assert_eq!(
            world_to_iso(far[0], far[1], far[2]),
            world_to_iso(near[0], near[1], near[2]),
            "points on the same view ray must project to the same pixel"
        );
        assert!(depth_at(near, params) < depth_at(far, params));
        let middle = [1120.0, 70.0, 920.0];
        let interpolated = (depth_at(far, params) + depth_at(near, params)) * 0.5;
        assert!((depth_at(middle, params) - interpolated).abs() < 1e-6);
    }

    #[test]
    fn encounter_geometry_keeps_distinct_unclipped_depths_at_every_yaw() {
        use crate::combat::Terrain;

        let prism = crate::prism::unit_hex_prism([1.0; 4]);
        for encounter in crate::catalog::ENCOUNTERS {
            // Narrow/mobile through large desktop scales.
            for size in [1.0, 12.0, 24.0, 43.0, 62.0] {
                for yaw in 0..4 {
                    let params = board_depth_origin_scale(encounter.cols, encounter.rows, size, yaw);
                    let radius = board_ground_radius(encounter.cols, encounter.rows, size);
                    let ground_top = -size * 0.12 + (size * 0.05).max(0.04);
                    for vertex in &prism.vertices {
                        let [x, y, z] = vertex.position;
                        assert_inside_depth_range(
                            [
                                params[0] + x * radius,
                                -size * 0.12 + y * (size * 0.05).max(0.04),
                                params[2] + z * radius,
                            ],
                            params,
                        );
                    }
                    for q in 0..encounter.cols {
                        for r in 0..encounter.rows {
                            let (x, z) = axial_to_world_yaw(Axial::new(q, r), size, yaw);
                            for terrain in [Terrain::Water, Terrain::Mud, Terrain::Grass, Terrain::Ruin] {
                                let height = terrain_height(terrain, size).max(0.04);
                                for vertex in &prism.vertices {
                                    let [vx, vy, vz] = vertex.position;
                                    let p = [x + vx * size * 0.92, vy * height, z + vz * size * 0.92];
                                    assert_inside_depth_range(p, params);
                                    // Ground at this exact screen pixel is farther along
                                    // the same view ray, including at both board edges.
                                    let dy = p[1] - ground_top;
                                    let ground = [p[0] - dy, ground_top, p[2] - dy];
                                    assert!(depth_at(p, params) < depth_at(ground, params));
                                }
                            }
                            // Conservative animated model envelope at every cell,
                            // including off-board limbs on the outermost cells.
                            for dx in [-2.0, 2.0] {
                                for dy in [-2.0, 4.0] {
                                    for dz in [-2.0, 2.0] {
                                        assert_inside_depth_range([x + dx * size, dy * size, z + dz * size], params);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn depth_normalization_preserves_board_and_world_party_scales() {
        for yaw in 0..4 {
            let small = board_depth_origin_scale(13, 10, 24.0, yaw);
            let large = board_depth_origin_scale(13, 10, 62.0, yaw);
            let (x, z) = axial_to_world_yaw(Axial::new(12, 9), 1.0, yaw);
            assert!((depth_at([x * 24.0, 24.0, z * 24.0], small)
                - depth_at([x * 62.0, 62.0, z * 62.0], large)).abs() < 1e-6);
        }
        let scale = 1.35;
        for bob in [-1.0, 0.0, 1.0] {
            let params = depth_origin_scale([0.0, bob, 0.0], scale * 8.0);
            for x in [-2.0, 2.0] {
                for y in [-2.0, 4.0] {
                    for z in [-2.0, 2.0] {
                        assert_inside_depth_range([x * scale, y * scale + bob, z * scale], params);
                    }
                }
            }
            assert_eq!(depth_at([0.0, bob, 0.0], params), 0.5);
        }
    }

    #[test]
    fn hunt_shader_uses_the_same_unclamped_depth_projection() {
        let source = include_str!("../assets/hunt.wgsl")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(source.contains("depth_origin_scale: vec4<f32>"));
        assert!(source.contains(
            "let depth = 0.5 - dot(p - globals.depth_origin_scale.xyz, vec3<f32>(1.0)) * globals.depth_origin_scale.w;"
        ));
        assert!(source.contains("o.clip = vec4<f32>(ndc_x, ndc_y, depth, 1.0);"));
    }

    #[test]
    fn fitted_encounters_clear_the_hud_at_every_yaw_and_viewport() {
        for (w, h) in [(320.0, 568.0), (390.0, 844.0), (800.0, 600.0), (1280.0, 720.0),
            (1280.0, 800.0), (1920.0, 1080.0), (2560.0, 1440.0)] {
            let viewport = board_viewport(w, h);
            let assert_visible = |x: f32, y: f32| {
                assert!(x >= viewport[0] - 0.01 && x <= viewport[2] + 0.01,
                    "x={x} outside {viewport:?} on {w}x{h}");
                assert!(y >= viewport[1] - 0.01 && y <= viewport[3] + 0.01,
                    "y={y} outside {viewport:?} on {w}x{h}");
            };
            for encounter in crate::catalog::ENCOUNTERS {
                let size = board_size(encounter.cols, encounter.rows, w, h);
                for yaw in 0..4 {
                    // Minimum, new battle, and Continue's default zoom.
                    for zoom in [crate::game::COMBAT_ZOOM_MIN, 1.0, 1.05] {
                        let (ox, oy) = camera_origin(encounter.cols, encounter.rows,
                            size, w, h, [0.0, 0.0], zoom, yaw);
                        for q in 0..encounter.cols {
                            for r in 0..encounter.rows {
                                let (x, z) = axial_to_world_yaw(Axial::new(q, r), size, yaw);
                                // A prism and an animated fighter fit inside this
                                // envelope; project it independently of the fit math.
                                for dx in [-size, size] {
                                    for y in [0.0, size * 3.0] {
                                        for dz in [-size, size] {
                                            let (sx, sy) = world_to_iso(x + dx, y, z + dz);
                                            assert_visible(ox + sx * zoom, oy + sy * zoom);
                                        }
                                    }
                                }
                                // Screen-space overlays also retain a generous margin.
                                let (sx, sy) = world_to_iso(x, size * 0.4, z);
                                let (px, py) = (ox + sx * zoom, oy + sy * zoom);
                                assert_visible(px - w * 0.04, py - h * 0.12);
                                assert_visible(px + w * 0.04, py);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn fitted_projection_and_picking_agree_after_pan_zoom_and_rotation() {
        for (w, h) in [(390.0, 844.0), (800.0, 600.0), (1280.0, 800.0), (1920.0, 1080.0)] {
            for encounter in crate::catalog::ENCOUNTERS {
                let size = board_size(encounter.cols, encounter.rows, w, h);
                for yaw in 0..4 {
                    for zoom in [crate::game::COMBAT_ZOOM_MIN, 1.0, crate::game::COMBAT_ZOOM_MAX] {
                        for pan in [[0.0, 0.0], [83.0, -47.0]] {
                            for q in 0..encounter.cols {
                                for r in 0..encounter.rows {
                                    let hex = Axial::new(q, r);
                                    let p = hex_to_screen(hex, w, h, encounter.cols, encounter.rows,
                                        pan, zoom, size * 0.14, yaw);
                                    assert_eq!(pick_screen(p[0] * w, p[1] * h, w, h,
                                        encounter.cols, encounter.rows, pan, zoom, yaw), Some(hex));
                                }
                            }
                            let centered = camera_origin(encounter.cols, encounter.rows,
                                size, w, h, [0.0, 0.0], zoom, yaw);
                            let panned = camera_origin(encounter.cols, encounter.rows,
                                size, w, h, pan, zoom, yaw);
                            assert!((panned.0 - centered.0 - pan[0]).abs() < 1e-4);
                            assert!((panned.1 - centered.1 - pan[1]).abs() < 1e-4);
                        }
                    }
                }
            }
        }
    }

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
