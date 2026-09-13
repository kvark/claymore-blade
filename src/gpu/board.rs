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
        let yaw = game.ui.yaw;
        let (ox, oy) = camera_origin(combat.cols, combat.rows, size, w, h, pan, game.ui.zoom, yaw);
        let (lx, lz) = rotate_yaw(0.55, -0.15, yaw);
        let actor = current_unit(combat);
        let (lamp0, lamp0c, lamp1, lamp1c) = combat_lamps(combat, actor, size, yaw, game);
        let mut rc = pass.with(&self.hunt);
        rc.bind(
            0,
            &HuntFrame {
                globals: HuntGlobals {
                    origin_zoom: [ox, oy, game.ui.zoom, 0.0],
                    screen: [w, h, 0.0, 0.0],
                    light_dir: [lx, 0.72, lz, 0.0],
                    lamp0,
                    lamp0c,
                    lamp1,
                    lamp1c,
                },
            },
        );
        rc.bind_vertex(0, self.prism.into());
        // Dark ground disc under the prism grid so gaps / lavapipe void do not read as pure black.
        let mid = crate::hex::Axial::new((combat.cols - 1) / 2, (combat.rows - 1) / 2);
        let (gx, gz) = axial_to_world_yaw(mid, size, yaw);
        let ground_r = size * (combat.cols.max(combat.rows) as f32 * 0.72 + 1.4);
        rc.bind(
            1,
            &HuntDraw {
                locals: HuntLocal {
                    world: [gx, -size * 0.12, gz, ground_r],
                    color: [0.07, 0.06, 0.05, (size * 0.05).max(0.04)],
                    pose: [1.0, 0.0, 0.0, 0.04],
                    joints: identity_palette(),
                },
            },
        );
        rc.draw(0, self.prism_count, 0, 1);
        let preview = game.preview_zone();
        let threat = game.threat_zone();
        let skill_on = game.ui.selected_skill.is_some();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let (x, z) = axial_to_world_yaw(*hex, size, yaw);
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
                        world: [x, 0.0, z, size * 0.92],
                        color: [color[0], color[1], color[2], height.max(0.04)],
                        // Unlit albedo lift so terrain hue survives Lavapipe + actor lamps.
                        pose: [1.0, 0.0, 0.0, 0.14],
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
                size * 0.72
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
                // Vika is a slim humanoid (XZ ~0.72); scale up so body reads at hex size
                // like wide valefor. Valefor keeps ~0.9 — playtest liked it.
                let scale = match kind {
                    ArchiveKind::Vika => unit_size * 1.35,
                    ArchiveKind::Valefor => unit_size * 0.9,
                };
                // Sit on hex top so the mesh is not buried in the prism.
                let ground = combat
                    .terrain
                    .iter()
                    .find(|(h, _)| *h == cell)
                    .map(|(_, tr)| terrain_height(*tr, size).max(0.0))
                    .unwrap_or(0.0);
                // Pale Clare tint reads as ink-on-ink; lift luminance a bit for vika.
                let (cr, cg, cb) = match kind {
                    ArchiveKind::Vika => (
                        (u.color[0] * 1.12 + 0.06).min(1.0),
                        (u.color[1] * 1.10 + 0.05).min(1.0),
                        (u.color[2] * 1.08 + 0.04).min(1.0),
                    ),
                    ArchiveKind::Valefor => (u.color[0], u.color[1], u.color[2]),
                };
                let mesh_glow = glow + match kind {
                    ArchiveKind::Vika => 0.12,
                    ArchiveKind::Valefor => 0.0,
                };
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
                            world: [x, ground, z, scale],
                            color: [cr, cg, cb, scale],
                            pose: [face.cos(), face.sin(), 0.0, mesh_glow],
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ArchiveKind {
    Vika,
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
        "raki" => None, // keep procedural kid
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
    use super::ArchiveKind;

    fn archive_scale_mul(kind: ArchiveKind) -> f32 {
        match kind {
            ArchiveKind::Vika => 1.35,
            ArchiveKind::Valefor => 0.9,
        }
    }

    #[test]
    fn vika_scaled_larger_than_valefor() {
        assert!(archive_scale_mul(ArchiveKind::Vika) > archive_scale_mul(ArchiveKind::Valefor));
        assert!(archive_scale_mul(ArchiveKind::Vika) >= 1.2);
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
