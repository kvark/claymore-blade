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
        let (ox, oy) = camera_origin(combat.cols, combat.rows, size, w, h, pan, game.ui.zoom);
        let mut rc = pass.with(&self.hunt);
        rc.bind(
            0,
            &HuntFrame {
                globals: HuntGlobals {
                    origin_zoom: [ox, oy, game.ui.zoom, 0.0],
                    screen: [w, h, 0.0, 0.0],
                    light_dir: [0.35, 0.82, 0.42, 0.0],
                },
            },
        );
        rc.bind_vertex(0, self.prism.into());
        let preview = game.preview_zone();
        let skill_on = game.ui.selected_skill.is_some();
        for (hex, terrain) in &combat.terrain {
            let height = terrain_height(*terrain, size);
            let (x, z) = axial_to_world(*hex, size);
            let mut color = match terrain {
                Terrain::Water => [0.16, 0.20, 0.22],
                Terrain::Mud => [0.28, 0.20, 0.14],
                Terrain::Ruin => [0.32, 0.30, 0.28],
                Terrain::Grass => [0.18, 0.19, 0.15],
            };
            if preview.iter().any(|h| h.q == hex.q && h.r == hex.r) {
                color = if skill_on {
                    [0.48, 0.18, 0.14]
                } else {
                    [0.36, 0.42, 0.22]
                };
            }
            if game.ui.hover == Some(*hex) {
                color = [0.55, 0.46, 0.26];
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
                let (x, z) = axial_to_world(cell, size);
                let hgt = size * if u.side == Side::Enemy { 0.55 } else { 0.38 };
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
