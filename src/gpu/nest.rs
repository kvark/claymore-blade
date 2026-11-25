//! Low-cost nest clearing: a continuous loam floor and outlying rocks in fog.
//! Same hunt pipeline / WebGL2 path as the board. No colliders or game rules.
use super::*;
use crate::iso::board_ground_center;
use crate::skel::identity_palette;

// Material kinds, mirrored by the `MAT_*` constants in hunt.wgsl.
pub(super) const MAT_MOSS: f32 = 1.0;
pub(super) const MAT_MUD: f32 = 2.0;
pub(super) const MAT_STONE: f32 = 3.0;
pub(super) const MAT_WATER: f32 = 4.0;
pub(super) const MAT_GROUND: f32 = 5.0;
pub(super) const MAT_ROCK: f32 = 6.0;
pub(super) const MAT_INK: f32 = 7.0;
/// C1 line art: fighter contour width (screen px) and the depth push that
/// keeps the black hull behind the body it outlines.
pub(super) const INK_HULL_PX: f32 = 1.3;
pub(super) const INK_HULL_DEPTH: f32 = 0.002;

/// Seed that marks the board's under-tile plinths: no ink on them, so the
/// tile gaps stay plain shadow instead of hatching.
pub(super) const PLINTH_SEED: f32 = -1.0;
/// C4 torchlight: pool radius in hex radii around Clare (about 4.5 hexes),
/// falling to `TORCH_FLOOR` (hunt.wgsl) at the rim, never to black.
pub(super) const TORCH_RADIUS: f32 = 8.0;

/// `HuntGlobals::torch`: the pool follows Clare (else the first living
/// party member); 0 strength when the party is gone.
pub(super) fn torch(combat: &CombatState, size: f32, yaw: u8) -> [f32; 4] {
    let alive = |u: &&crate::combat::Unit| !u.dead && u.side == Side::Player;
    let lead = combat.units.iter().filter(alive).find(|u| u.template_id == "clare");
    match lead.or_else(|| combat.units.iter().find(alive)) {
        Some(u) => {
            let (x, z) = axial_to_world_yaw(core_hex(u), size, yaw);
            [x, z, size * TORCH_RADIUS, 1.0]
        }
        None => [0.0, 0.0, 1.0, 0.0],
    }
}

/// Peak torch-radius wobble (fraction of the radius).
pub(super) const TORCH_FLICKER: f32 = 0.035;

/// Torch pool radius multiplier at `time`: two slow, incommensurate sines
/// (~0.2 Hz and ~0.5 Hz), so the light breathes without strobing. Pure
/// function of time; no effect on rules or timing.
pub(super) fn torch_flicker(time: f32) -> f32 {
    let w = 0.6 * (1.3 * time).sin() + 0.4 * (3.1 * time + 1.7).sin();
    1.0 + TORCH_FLICKER * w
}

/// The ink-hull redraw of a fighter: same pose and skin, drawn black.
pub(super) fn ink_hull(body: HuntLocal) -> HuntLocal {
    HuntLocal { rim: NO_RIM, mat: [MAT_INK, INK_HULL_PX, INK_HULL_DEPTH, 1.0], ..body }
}

/// `HuntLocal::mat` for a nest / terrain draw.
pub(super) fn material(kind: f32, size: f32, combat: &CombatState, seed: f32) -> [f32; 4] {
    // Farthest corner of the board parallelogram in tile units.
    let (cx, cz) = board_ground_center(combat.cols, combat.rows, 1.0, 0);
    let radius = (cx * cx + cz * cz).sqrt();
    [kind, seed, radius, size]
}

pub(super) fn terrain_material(terrain: Terrain, size: f32, combat: &CombatState) -> [f32; 4] {
    let kind = match terrain {
        Terrain::Grass => MAT_MOSS,
        Terrain::Mud => MAT_MUD,
        Terrain::Ruin => MAT_STONE,
        Terrain::Water => MAT_WATER,
    };
    material(kind, size, combat, 0.0)
}

/// Outlying rocks: hashed lattice cells outside a two-hex apron around the
/// board, out to `NEST_RING`. Returns (cell, shape hash 0..=5).
pub(super) fn nest_rocks(cols: i32, rows: i32) -> Vec<(crate::hex::Axial, u32)> {
    let ring = crate::iso::NEST_RING as i32;
    let mut out = Vec::new();
    for q in -ring..cols + ring {
        for r in -ring..rows + ring {
            if q >= -2 && r >= -2 && q <= cols + 1 && r <= rows + 1 {
                continue;
            }
            let hash = ((q + 11) * 71 + (r + 13) * 131).rem_euclid(37);
            if hash <= 5 {
                out.push((crate::hex::Axial::new(q, r), hash as u32));
            }
        }
    }
    out
}

impl Renderer {
    pub(super) fn draw_nest(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        combat: &CombatState,
        size: f32,
        yaw: u8,
    ) {
        let (cx, cz) = board_ground_center(combat.cols, combat.rows, size, yaw);
        let ground = material(MAT_GROUND, size, combat, 0.0);
        // Huge floor is pinned to far depth by the shader: it cannot touch
        // the board's linear depth bounds or cover a fighter at any yaw.
        self.nest_prism(
            rc,
            [cx, -size * 0.24, cz, size * 180.0],
            [0.08, 0.07, 0.06, size * 0.01],
            [1.0, 0.0, 0.0, 0.0],
            ground,
        );
        // Scatter only OUTSIDE a two-tile apron. Low, irregular silhouettes
        // nestle in the mist rather than rising in front of the playable cells.
        for (hex, hash) in nest_rocks(combat.cols, combat.rows) {
            let (x, z) = axial_to_world_yaw(hex, size, yaw);
            let a = hash as f32 * 1.17;
            let radius = size * (0.5 + hash as f32 * 0.10);
            let height = size * (0.28 + hash as f32 * 0.13);
            self.nest_prism(
                rc,
                [x, -size * 0.2, z, radius],
                [0.035, 0.034, 0.032, height],
                [a.cos(), a.sin(), 0.18, 0.0],
                material(MAT_ROCK, size, combat, hash as f32),
            );
            // A narrower, offset cap breaks the hex-prism outline into a boulder.
            let b = a + 0.7;
            self.nest_prism(
                rc,
                [
                    x + radius * 0.2,
                    -size * 0.2,
                    z - radius * 0.15,
                    radius * 0.62,
                ],
                [0.03, 0.029, 0.027, height * 1.45],
                [b.cos(), b.sin(), -0.25, 0.0],
                material(MAT_ROCK, size, combat, hash as f32 + 0.5),
            );
        }
    }

    fn nest_prism(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        world: [f32; 4],
        color: [f32; 4],
        pose: [f32; 4],
        mat: [f32; 4],
    ) {
        rc.bind(
            1,
            &HuntDraw {
                locals: HuntLocal {
                    world,
                    color,
                    pose,
                    rim: NO_RIM,
                    mat,
                    joints: identity_palette(),
                },
            },
        );
        rc.draw(0, self.prism_count, 0, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::nest_rocks;
    use crate::hex::{hex_distance, Axial};

    #[test]
    fn nest_rocks_ring_the_board_without_touching_it() {
        for e in crate::catalog::ENCOUNTERS.iter() {
            let rocks = nest_rocks(e.cols, e.rows);
            assert!(
                rocks.len() >= 8 && rocks.len() <= 60,
                "{}: {}",
                e.id,
                rocks.len()
            );
            for (rock, _) in &rocks {
                let near = (0..e.cols)
                    .flat_map(|q| (0..e.rows).map(move |r| Axial::new(q, r)))
                    .map(|c| hex_distance(c, *rock))
                    .min()
                    .unwrap();
                assert!(
                    near >= 2,
                    "{}: rock {rock:?} only {near} from the board",
                    e.id
                );
            }
        }
    }
}

#[cfg(test)]
mod shader_tests {
    use super::*;

    #[test]
    fn material_kinds_match_hunt_wgsl() {
        let src = include_str!("../../assets/hunt.wgsl");
        for (name, v) in [
            ("MAT_MOSS", MAT_MOSS),
            ("MAT_MUD", MAT_MUD),
            ("MAT_STONE", MAT_STONE),
            ("MAT_WATER", MAT_WATER),
            ("MAT_GROUND", MAT_GROUND),
            ("MAT_ROCK", MAT_ROCK),
            ("MAT_INK", MAT_INK),
        ] {
            assert!(src.contains(&format!("const {name}: f32 = {v:.1};")), "{name}");
        }
    }

    #[test]
    fn ink_hull_keeps_the_pose_and_draws_black_behind_the_body() {
        let body = HuntLocal {
            world: [1.0, 2.0, 3.0, 4.0],
            color: [0.7, 0.73, 0.78, 4.0],
            pose: [0.0, 1.0, 0.0, 0.12],
            rim: [0.85, 0.9, 1.0, 1.6],
            mat: [0.0; 4],
            joints: identity_palette(),
        };
        let hull = ink_hull(body);
        assert_eq!((hull.world, hull.color, hull.pose), (body.world, body.color, body.pose));
        assert_eq!(hull.rim, NO_RIM);
        assert_eq!(hull.mat[0], MAT_INK);
        // A thin contour (about a pixel) pushed behind, never in front of, the body.
        assert!(hull.mat[1] > 0.5 && hull.mat[1] < 2.5 && hull.mat[2] > 0.0 && hull.mat[2] < 0.01);
    }

    #[test]
    fn torch_flicker_is_a_small_slow_deterministic_wobble() {
        let mut lo = f32::MAX;
        let mut hi = f32::MIN;
        let mut prev = torch_flicker(0.0);
        for i in 0..2000 {
            let t = i as f32 / 60.0;
            let f = torch_flicker(t);
            assert_eq!(f, torch_flicker(t));
            assert!((f - 1.0).abs() <= TORCH_FLICKER + 1e-6);
            // Low frequency: under 0.4% radius change per 60 Hz frame.
            assert!((f - prev).abs() < 0.004, "t={t}");
            prev = f;
            lo = lo.min(f);
            hi = hi.max(f);
        }
        assert!(hi - lo > TORCH_FLICKER, "visible but subtle: {lo}..{hi}");
    }

    #[test]
    fn torch_follows_clare_then_the_party_and_goes_out_with_them() {
        let raw = include_str!("../../src/game/testdata/a9841f1-paburo-raki-combat.json");
        let v: serde_json::Value = serde_json::from_str(raw).unwrap();
        let mut combat: CombatState = serde_json::from_value(v["combat"].clone()).unwrap();
        let at = |c: &CombatState, id: &str| {
            let u = c.units.iter().find(|u| u.id == id).unwrap();
            axial_to_world_yaw(core_hex(u), 2.0, 1)
        };
        let (cx, cz) = at(&combat, "clare");
        assert_eq!(torch(&combat, 2.0, 1), [cx, cz, 2.0 * TORCH_RADIUS, 1.0]);
        combat.units.iter_mut().find(|u| u.id == "clare").unwrap().dead = true;
        let (rx, rz) = at(&combat, "raki");
        assert_eq!(torch(&combat, 2.0, 1), [rx, rz, 2.0 * TORCH_RADIUS, 1.0]);
        for u in combat.units.iter_mut().filter(|u| u.side == Side::Player) {
            u.dead = true;
        }
        assert_eq!(torch(&combat, 2.0, 1)[3], 0.0);
    }

    #[test]
    fn c4_light_is_shader_side_and_spares_plinths() {
        let hunt = include_str!("../../assets/hunt.wgsl");
        assert!(hunt.contains("lamp1c: vec4<f32>,\n    torch: vec4<f32>,\n};"), "Globals layout matches HuntGlobals");
        assert!(hunt.contains("fn c4_light(") && hunt.contains("if (v.mat.y < -0.5) {"));
        assert!(PLINTH_SEED < -0.5);
        // Far hexes darken, never vanish.
        let floor: f32 = hunt.split("const TORCH_FLOOR: f32 = ").nth(1).unwrap()
            .split(';').next().unwrap().parse().unwrap();
        assert!(floor >= 0.3 && floor < 1.0, "{floor}");
    }

    #[test]
    fn c1_grade_stays_in_the_3d_scene() {
        // The grade and ink live in hunt.wgsl only: the flat HUD, hex overlays,
        // tactical markers and cut-ins keep their colours.
        let hunt = include_str!("../../assets/hunt.wgsl");
        let flat = include_str!("../../assets/flat.wgsl");
        assert!(hunt.contains("fn grade(") && hunt.contains("const MAT_INK"));
        assert!(!flat.contains("grade") && !flat.contains("INK"));
    }
}
