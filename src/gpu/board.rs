use super::*;
use crate::skel::{identity_palette, joint_palette, pose_fighter, PoseInput};

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
        let preview = game.preview_zone();
        let skill_on = game.ui.selected_skill.is_some();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let (x, z) = axial_to_world_yaw(*hex, size, yaw);
            let mut color = match terrain {
                Terrain::Water => [0.10, 0.12, 0.14],
                Terrain::Mud => [0.22, 0.16, 0.12],
                Terrain::Ruin => [0.26, 0.24, 0.22],
                Terrain::Grass => [0.14, 0.15, 0.12],
            };
            if preview.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = if skill_on {
                    [0.42, 0.14, 0.12]
                } else {
                    [0.28, 0.34, 0.18]
                };
            }
            if game.ui.hover == Some(*hex) {
                color = [0.48, 0.40, 0.22];
            }
            rc.bind(
                1,
                &HuntDraw {
                    locals: HuntLocal {
                        world: [x, 0.0, z, size * 0.92],
                        color: [color[0], color[1], color[2], height.max(0.04)],
                        pose: [1.0, 0.0, 0.0, 0.0],
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
            let bones = pose_fighter(&PoseInput {
                x,
                z,
                size,
                facing: u.facing,
                cam_yaw: yaw,
                time: t + (u.id.bytes().fold(0u32, |a, b| a.wrapping_add(b as u32)) as f32) * 0.07,
                color: u.color,
                side: u.side,
                acting,
                hurt: hurt && acting,
                trans: u.trans,
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
            rc.bind_vertex(0, self.prism.into());
        }
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
        let gold = [1.0, 0.78, 0.42, size * 3.2];
        (p, gold)
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
