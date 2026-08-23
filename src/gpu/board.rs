use super::*;

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
        // Keep lighting direction consistent in world space while the board rotates.
        let (lx, lz) = rotate_yaw(0.55, -0.15, yaw);
        let mut rc = pass.with(&self.hunt);
        rc.bind(
            0,
            &HuntFrame {
                globals: HuntGlobals {
                    origin_zoom: [ox, oy, game.ui.zoom, 0.0],
                    screen: [w, h, 0.0, 0.0],
                    light_dir: [lx, 0.72, lz, 0.0],
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
                // Stagnant, oily water
                Terrain::Water => [0.10, 0.12, 0.14],
                // Packed ash-dirt
                Terrain::Mud => [0.22, 0.16, 0.12],
                // Broken concrete / brick
                Terrain::Ruin => [0.26, 0.24, 0.22],
                // Dead grass / scorched ground
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
                    },
                },
            );
            rc.draw(0, self.prism_count, 0, 1);
        }
        for u in &combat.units {
            if u.dead {
                continue;
            }
            for cell in live_cells(u) {
                let (x, z) = axial_to_world_yaw(cell, size, yaw);
                let hgt = size * if u.side == Side::Enemy { 0.62 } else { 0.44 };
                rc.bind(
                    1,
                    &HuntDraw {
                        locals: HuntLocal {
                            world: [x, 0.0, z, size * 0.55],
                            color: [u.color[0], u.color[1], u.color[2], hgt],
                        },
                    },
                );
                rc.draw(0, self.prism_count, 0, 1);
            }
        }
    }
}
