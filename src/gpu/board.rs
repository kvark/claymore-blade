use super::*;
use crate::skel::{archive_joint_palette, identity_palette, joint_palette, pose_fighter, PoseInput};

impl Renderer {
    pub(super) fn draw_board(
        &self,
        pass: &mut gpu::RenderCommandEncoder,
        combat: &CombatState,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let size = board_size(combat.cols, combat.rows, w, h);
        let pan = Self::pan(game, w, h);
        let zoom = game.view_camera().1;
        let yaw = game.ui.yaw;
        let (ox, oy) = camera_origin(combat.cols, combat.rows, size, w, h, pan, zoom, yaw);
        let (lx, lz) = rotate_yaw(0.55, -0.15, yaw);
        let actor = current_unit(combat);
        let (lamp0, lamp0c, lamp1, lamp1c) = combat_lamps(combat, actor, size, yaw, game);
        let mut rc = pass.with(&self.hunt);
        rc.bind(
            0,
            &HuntFrame {
                globals: HuntGlobals {
                    origin_zoom: [ox, oy, zoom, 0.0],
                    screen: [w, h, 0.0, 0.0],
                    depth_origin_scale: board_depth_origin_scale(combat.cols, combat.rows, size, yaw),
                    light_dir: [lx, 0.72, lz, 0.0],
                    lamp0,
                    lamp0c,
                    lamp1,
                    lamp1c,
                },
            },
        );
        rc.bind_vertex(0, self.prism.into());
        // Dark ground under the prism grid so gaps / lavapipe void do not read as pure black.
        // One plinth per terrain cell, placed exactly like the tiles, so the ground
        // follows the board's footprint at every yaw instead of a separately sized plate.
        let [px, py, pz] = BOARD_PRISM_POSE;
        for (hex, _) in &combat.terrain {
            rc.bind(
                1,
                &HuntDraw {
                    locals: HuntLocal {
                        world: board_ground_world(*hex, size, yaw),
                        color: [0.07, 0.06, 0.05, board_ground_height(size)],
                        pose: [px, py, pz, 0.04],
                        rim: NO_RIM,
                        joints: identity_palette(),
                    },
                },
            );
            rc.draw(0, self.prism_count, 0, 1);
        }
        let preview = game.preview_zone();
        let threat = game.threat_zone();
        let skill_on = game.ui.selected_skill.is_some();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let mut color = terrain_tint(*terrain);
            if preview.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = if skill_on {
                    [0.42, 0.14, 0.12]
                } else {
                    [0.28, 0.34, 0.18]
                };
            }
            // Enemy claw tell — blood/danger, not gold (overrides move tint).
            if threat.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = [0.50, 0.10, 0.12];
            }
            if game.ui.hover == Some(*hex) {
                color = [0.52, 0.48, 0.40]; // dust/ash, not gold
            }
            rc.bind(
                1,
                &HuntDraw {
                    locals: HuntLocal {
                        world: board_tile_world(*hex, size, yaw),
                        color: [color[0], color[1], color[2], height.max(0.04)],
                        // Unlit albedo lift so terrain hue survives Lavapipe + actor lamps.
                        pose: [px, py, pz, 0.14],
                        rim: NO_RIM,
                        joints: identity_palette(),
                    },
                },
            );
            rc.draw(0, self.prism_count, 0, 1);
        }
        let t = game.fx.time;
        let hurt = game.fx.hitstop > 0.0 || game.fx.flash > 0.0;
        for u in &combat.units {
            if u.dead {
                continue;
            }
            let cell = core_hex(u);
            let (x, z) = axial_to_world_yaw(cell, size, yaw);
            let acting = actor.map(|a| a.id == u.id).unwrap_or(false);
            let (clip, clip_u) = game.fx.clip_of(&u.id);
            let unit_size = if u.template_id == "raki" {
                size * RAKI_SIZE
            } else {
                size
            };
            let glow = ((u.trans as f32) / 100.0).clamp(0.0, 1.0) * 0.55
                + if acting { 0.25 } else { 0.0 };
            // Prefer archive GLB: Clare/fighters → vika, yoma/enemies → valefor.
            let archive = archive_mesh_for(self, &u.template_id, u.side);
            if let Some((kind, buf, count)) = archive {
                let face = (u.facing.rem_euclid(6) as f32) * std::f32::consts::FRAC_PI_3
                    + (yaw as f32) * std::f32::consts::FRAC_PI_2;
                // Models face +Z in Blender export; nudge so they look along hex facing.
                let face = face + std::f32::consts::PI;
                // Vika is a slim humanoid (XZ ~0.72); scale up so body reads at hex size.
                // Valefor (yoma) was 0.9 and read tiny at default zoom.
                let scale = unit_size * archive_scale_mul(kind) * unit_tier_scale(&u.template_id, u.side);
                let body = archive_body(kind);
                // Sit on hex top so the mesh is not buried in the prism.
                let ground = combat
                    .terrain
                    .iter()
                    .find(|(h, _)| *h == cell)
                    .map(|(_, tr)| terrain_height(*tr, size).max(0.0))
                    .unwrap_or(0.0);
                let [cr, cg, cb] = archive_tint(kind, u.color);
                let mesh_glow = glow + body.glow;
                // Drive archive skin with the same fight clips as procedural bones
                // (idle bob / Cut swing / hurt flinch) via hunt.wgsl's 8-joint palette.
                let pose_in = PoseInput {
                    x,
                    z,
                    size: unit_size,
                    facing: u.facing,
                    cam_yaw: yaw,
                    time: t
                        + (u.id.bytes().fold(0u32, |a, b| a.wrapping_add(b as u32)) as f32) * 0.07,
                    color: u.color,
                    side: u.side,
                    acting,
                    hurt: hurt && acting,
                    trans: u.trans,
                    clip,
                    clip_u,
                };
                rc.bind_vertex(0, buf.into());
                rc.bind(
                    1,
                    &HuntDraw {
                        locals: HuntLocal {
                            world: [x, ground, z, scale * body.width],
                            color: [cr, cg, cb, scale],
                            pose: [face.cos(), face.sin(), 0.0, mesh_glow],
                            rim: body.rim,
                            joints: archive_joint_palette(&pose_in),
                        },
                    },
                );
                rc.draw(0, count, 0, 1);
            } else {
                let bones = pose_fighter(&PoseInput {
                    x,
                    z,
                    size: unit_size,
                    facing: u.facing,
                    cam_yaw: yaw,
                    time: t
                        + (u.id.bytes().fold(0u32, |a, b| a.wrapping_add(b as u32)) as f32) * 0.07,
                    color: u.color,
                    side: u.side,
                    acting,
                    hurt: hurt && acting,
                    trans: u.trans,
                    clip,
                    clip_u,
                });
                let glow = bones[1].glow;
                rc.bind_vertex(0, self.fighter.into());
                rc.bind(
                    1,
                    &HuntDraw {
                        locals: HuntLocal {
                            world: [0.0, 0.0, 0.0, 1.0],
                            color: [u.color[0], u.color[1], u.color[2], 1.0],
                            pose: [1.0, 0.0, 0.0, glow],
                            rim: NO_RIM,
                            joints: joint_palette(&bones),
                        },
                    },
                );
                rc.draw(0, self.fighter_count, 0, 1);
            }
            rc.bind_vertex(0, self.prism.into());
        }
    }
}

/// Raki stands at about this fraction of a Claymore (a boy next to Clare).
pub(super) const RAKI_SIZE: f32 = 0.72;
/// Raki's clothes: warm dark brown, nowhere near Claymore silver.
pub(super) const RAKI_TINT: [f32; 3] = [0.26, 0.13, 0.05];
/// Negative rim = ink outline on Raki's silhouette.
pub(super) const KID_RIM: [f32; 4] = [1.0, 1.0, 1.0, -1.2];

/// Per-kind body shaping on top of the uniform archive scale.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ArchiveBody {
    /// XZ multiplier relative to height (> 1 = bulkier silhouette).
    pub width: f32,
    /// Unlit lift added to the unit glow.
    pub glow: f32,
    /// `HuntLocal::rim` (rgb, strength).
    pub rim: [f32; 4],
}

pub(super) fn archive_body(kind: ArchiveKind) -> ArchiveBody {
    match kind {
        ArchiveKind::Vika => ArchiveBody { width: 1.0, glow: 0.12, rim: NO_RIM },
        // A kid's frame is stockier than an adult warrior's, and an ink outline
        // (negative rim) keeps the small dark figure from washing out under
        // the actor lamp next to Clare.
        ArchiveKind::Kid => ArchiveBody { width: 1.3, glow: 0.05, rim: KID_RIM },
        ArchiveKind::Valefor => yoma_body(),
    }
}

/// Yoma bulk at default zoom. Valefor is a thin winged frame that read as
/// wispy red strokes; side-by-side (see PR) a wider XZ stance + darker ink
/// albedo + blood rim read heaviest, ahead of rim-only, wide-only and the
/// T-posed ifrit mesh.
pub(super) const YOMA_WIDTH: f32 = 1.4;
/// Darkens the catalog yoma colour into wet, heavy ink before lighting.
pub(super) const YOMA_INK: [f32; 3] = [0.5, 0.4, 0.4];
/// Blood rim on the yoma silhouette so the dark mass separates from the board.
pub(super) const YOMA_RIM: [f32; 4] = [0.85, 0.10, 0.08, 0.8];

fn yoma_body() -> ArchiveBody {
    ArchiveBody { width: YOMA_WIDTH, glow: 0.0, rim: YOMA_RIM }
}

/// Albedo the hunt shader lights for an archive body.
pub(super) fn archive_tint(kind: ArchiveKind, color: [f32; 3]) -> [f32; 3] {
    match kind {
        // Pale Clare tint reads as ink-on-ink; lift luminance a bit for vika.
        ArchiveKind::Vika => [
            (color[0] * 1.12 + 0.06).min(1.0),
            (color[1] * 1.10 + 0.05).min(1.0),
            (color[2] * 1.08 + 0.04).min(1.0),
        ],
        ArchiveKind::Kid => RAKI_TINT,
        ArchiveKind::Valefor => [color[0] * YOMA_INK[0], color[1] * YOMA_INK[1], color[2] * YOMA_INK[2]],
    }
}

/// Archive mesh scale per hex. Vika is slim; valefor is wide.
pub(super) const VIKA_SCALE: f32 = 1.35;
/// Yoma body scale (0.9, then 1.2, both read tiny at default zoom). A
/// common yoma now stands a little taller than Clare; tiers stack on top.
pub(super) const VALEFOR_SCALE: f32 = 1.5;
const VALEFOR_SCALE_BASE: f32 = 0.9;

pub(super) fn archive_scale_mul(kind: ArchiveKind) -> f32 {
    match kind {
        ArchiveKind::Vika | ArchiveKind::Kid => VIKA_SCALE,
        ArchiveKind::Valefor => VALEFOR_SCALE,
    }
}

/// Claymore reads as a lone slim warrior against looming yoma: enemy bodies
/// scale by tier on top of the common-yoma mesh scale. Players stay 1.0.
pub(crate) fn unit_tier_scale(template_id: &str, side: Side) -> f32 {
    if side != Side::Enemy {
        return 1.0;
    }
    match template_id {
        "yoma" => YOMA_TIER,
        "yoma_stretch" => LONG_YOMA_TIER,
        "ophelia" | "worm" => AWAKENED_TIER,
        _ => YOMA_TIER,
    }
}

/// Common yoma: one hex, already bumped via `VALEFOR_SCALE`.
pub(super) const YOMA_TIER: f32 = 1.0;
/// Long Yoma (stretch) towers over the pack.
pub(super) const LONG_YOMA_TIER: f32 = 1.3;
/// Awakened Beings occupy several cells; the body follows.
pub(super) const AWAKENED_TIER: f32 = 1.6;

/// Fallback (no-mesh) enemy portraits grow with the yoma body bump.
pub(crate) fn enemy_portrait_scale() -> f32 {
    VALEFOR_SCALE / VALEFOR_SCALE_BASE
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArchiveKind {
    Vika,
    /// Raki: vika mesh, kid scale (see `RAKI_SIZE`) and his own tint.
    Kid,
    Valefor,
}

/// Which archive GLB (if any) is bound for this unit template.
pub(super) fn archive_mesh_for<'a>(
    r: &'a Renderer,
    template_id: &str,
    side: Side,
) -> Option<(ArchiveKind, gpu::Buffer, u32)> {
    let pick = |kind: ArchiveKind, slot: Option<(gpu::Buffer, u32)>| {
        slot.map(|(buf, count)| (kind, buf, count))
    };
    match template_id {
        "clare" | "miria" | "helen" | "deneve" => pick(ArchiveKind::Vika, r.vika),
        "yoma" | "yoma_stretch" => pick(ArchiveKind::Valefor, r.valefor),
        // Raki: the same archive human at kid scale, in warm dark travel clothes.
        "raki" => pick(ArchiveKind::Kid, r.vika),
        "ophelia" | "worm" => pick(ArchiveKind::Valefor, r.valefor),
        _ => match side {
            Side::Player => pick(ArchiveKind::Vika, r.vika),
            Side::Enemy => pick(ArchiveKind::Valefor, r.valefor),
        },
    }
}

/// True when the unit draws an archive GLB (so the floating 2D portrait should step aside).
pub(super) fn unit_has_archive_mesh(r: &Renderer, template_id: &str, side: Side) -> bool {
    archive_mesh_for(r, template_id, side).is_some()
}

/// Base albedo per terrain. Tuned so Lavapipe (software Vulkan) can tell types apart
/// at a glance while staying in DESIGN.md Ink / Ash / Steel / Blood — no gold/purple.
pub(super) fn terrain_tint(terrain: Terrain) -> [f32; 3] {
    match terrain {
        Terrain::Water => [0.03, 0.08, 0.22], // darker, bluer
        Terrain::Mud => [0.42, 0.16, 0.06],   // browner earth
        Terrain::Ruin => [0.42, 0.41, 0.44],  // cooler steel grey
        Terrain::Grass => [0.08, 0.34, 0.07], // greener moss
    }
}

#[cfg(test)]
mod archive_scale_tests {
    use super::{archive_scale_mul, enemy_portrait_scale, ArchiveKind};

    #[test]
    fn vika_reads_at_hex_size() {
        // Vika is slim, so it still needs a lift; yoma now loom over it
        // (see `tier_scale_tests`).
        assert!(archive_scale_mul(ArchiveKind::Vika) >= 1.2);
    }

    #[test]
    fn yoma_reads_bigger_and_fallback_portraits_follow() {
        let v = archive_scale_mul(ArchiveKind::Valefor);
        assert!(v > 0.9 && v <= 1.6, "yoma body {v}");
        // Fallback portrait grows by the same ratio as the mesh.
        assert!((enemy_portrait_scale() - v / 0.9).abs() < 1e-6);
    }
}

#[cfg(test)]
mod archive_look_tests {
    use super::*;

    fn luma(c: [f32; 3]) -> f32 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    #[test]
    fn raki_is_a_small_warm_dark_figure_next_to_clare() {
        let clare = archive_tint(ArchiveKind::Vika, [0.78, 0.80, 0.83]);
        let raki = archive_tint(ArchiveKind::Kid, [0.62, 0.48, 0.36]);
        // Warm clothing (red > green > blue), far darker than Claymore silver.
        assert!(raki[0] > raki[1] && raki[1] > raki[2], "{raki:?}");
        assert!(luma(raki) < luma(clare) * 0.35, "raki {raki:?} vs clare {clare:?}");
        // Same human mesh scale as Clare, at kid size.
        assert_eq!(archive_scale_mul(ArchiveKind::Kid), archive_scale_mul(ArchiveKind::Vika));
        assert!(RAKI_SIZE > 0.6 && RAKI_SIZE < 0.8);
        let kid = archive_body(ArchiveKind::Kid);
        assert!(kid.width >= 1.0 && kid.width <= 1.4);
        assert!(kid.rim[3] < 0.0, "ink outline");
    }

    #[test]
    fn yoma_read_as_heavy_dark_mass() {
        let body = archive_body(ArchiveKind::Valefor);
        assert!(body.width > 1.2 && body.width <= 1.6, "bulk {}", body.width);
        let cat = [0.54, 0.23, 0.20];
        let ink = archive_tint(ArchiveKind::Valefor, cat);
        assert!(luma(ink) < luma(cat) * 0.6);
        assert!(ink[0] > ink[1] && ink[0] > ink[2], "still blood-red");
        // Blood rim, not gold or white.
        assert!(body.rim[3] > 0.0 && body.rim[0] > body.rim[1] * 4.0);
        // Players keep a plain silhouette.
        assert_eq!(archive_body(ArchiveKind::Vika).rim, NO_RIM);
    }

    #[test]
    fn hunt_shader_rim_field_matches_hunt_local_layout() {
        let src = include_str!("../../assets/hunt.wgsl")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(src.contains(
            "world: vec4<f32>, color: vec4<f32>, pose: vec4<f32>, rim: vec4<f32>, joints: array<mat3x4<f32>, 8>,"
        ));
        assert_eq!(std::mem::size_of::<HuntLocal>(), 4 * 16 + 8 * 48);
        assert!(src.contains("lit += locals.rim.xyz * (locals.rim.w * edge * edge);"));
    }
}

#[cfg(test)]
mod tier_scale_tests {
    use super::*;

    #[test]
    fn yoma_tiers_grow_and_players_stay_put() {
        let y = unit_tier_scale("yoma", Side::Enemy);
        let long = unit_tier_scale("yoma_stretch", Side::Enemy);
        let awake = unit_tier_scale("ophelia", Side::Enemy);
        assert!((y - 1.0).abs() < 1e-6);
        assert!(long > y && awake > long);
        assert_eq!(unit_tier_scale("worm", Side::Enemy), awake);
        assert_eq!(unit_tier_scale("clare", Side::Player), 1.0);
        assert_eq!(unit_tier_scale("yoma", Side::Player), 1.0);
        // A Long Yoma may loom but should stay around two tiles tall.
        assert!(VALEFOR_SCALE * long <= 2.0, "long yoma {}", VALEFOR_SCALE * long);
        // Common yoma now stand at least as tall as the slim Claymore mesh.
        assert!(VALEFOR_SCALE * y >= VIKA_SCALE);
    }
}

#[cfg(test)]
mod terrain_tint_tests {
    use super::terrain_tint;
    use crate::combat::Terrain;

    fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
        let dr = a[0] - b[0];
        let dg = a[1] - b[1];
        let db = a[2] - b[2];
        (dr * dr + dg * dg + db * db).sqrt()
    }

    #[test]
    fn terrain_types_are_visibly_separated() {
        let w = terrain_tint(Terrain::Water);
        let m = terrain_tint(Terrain::Mud);
        let r = terrain_tint(Terrain::Ruin);
        let g = terrain_tint(Terrain::Grass);
        // Pairwise RGB distance must clear Lavapipe wash-out (~0.08 was indistinguishable).
        for (a, b, label) in [
            (w, m, "water-mud"),
            (w, r, "water-ruin"),
            (w, g, "water-grass"),
            (m, r, "mud-ruin"),
            (m, g, "mud-grass"),
            (r, g, "ruin-grass"),
        ] {
            assert!(dist(a, b) >= 0.22, "{label} too close: {}", dist(a, b));
        }
        assert!(g[1] > g[0] && g[1] > g[2], "grass should be greener");
        assert!(m[0] > m[1] && m[0] > m[2], "mud should be browner (red-led)");
        assert!((r[0] - r[1]).abs() < 0.04 && (r[1] - r[2]).abs() < 0.04, "ruin should be grey");
        assert!(w[2] > w[0] && w[2] > w[1], "water should be bluer");
        assert!(w[0] + w[1] + w[2] < g[0] + g[1] + g[2], "water darker than grass");
    }
}

fn combat_lamps(
    combat: &CombatState,
    actor: Option<&crate::combat::Unit>,
    size: f32,
    yaw: u8,
    game: &Game,
) -> ([f32; 4], [f32; 4], [f32; 4], [f32; 4]) {
    let place = |hex: crate::hex::Axial, y: f32| {
        let (x, z) = axial_to_world_yaw(hex, size, yaw);
        [x, y, z, 1.35]
    };
    let (lamp0, lamp0c) = if let Some(u) = actor {
        let p = place(core_hex(u), size * 0.55);
        // Ash/Steel lift — gold washed terrain types together on Lavapipe.
        let ash = [0.82, 0.78, 0.68, size * 2.2];
        (p, ash)
    } else {
        ([0.0, size, 0.0, 0.0], [0.0, 0.0, 0.0, size])
    };
    let (lamp1, lamp1c) = if let Some(hex) = game.ui.hover {
        let p = place(hex, size * 0.35);
        let tint = if game.ui.selected_skill.is_some() {
            [0.95, 0.28, 0.18, size * 2.4]
        } else {
            [0.45, 0.75, 0.35, size * 2.2]
        };
        (p, tint)
    } else if let Some(e) = combat.units.iter().find(|u| !u.dead && u.side == Side::Enemy) {
        let p = place(core_hex(e), size * 0.5);
        ([p[0], p[1], p[2], 0.85 + e.trans as f32 * 0.006], [0.85, 0.18, 0.14, size * 2.8])
    } else {
        ([0.0, size, 0.0, 0.0], [0.0, 0.0, 0.0, size])
    };
    (lamp0, lamp0c, lamp1, lamp1c)
}

