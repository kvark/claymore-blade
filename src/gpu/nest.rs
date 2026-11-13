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
        ] {
            assert!(src.contains(&format!("const {name}: f32 = {v:.1};")), "{name}");
        }
    }
}
