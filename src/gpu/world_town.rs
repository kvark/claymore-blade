use super::*;
use super::util::{loc_prop, prop_size};

impl Renderer {
    pub(super) fn draw_world(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/world-map.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [0.92, 0.88, 0.82, 1.0],
        );
        // Fog of war: deep ink on unexplored; softer ash rim at the vision edge.
        {
            use crate::world::{is_explored, is_explored_rim, EXPLORED_H, EXPLORED_W};
            let deep = [0.04, 0.03, 0.05, 0.92];
            let rim = [0.12, 0.10, 0.11, 0.55];
            let ash = [0.55, 0.48, 0.40, 0.28];
            for row in 0..EXPLORED_H {
                let mut col = 0usize;
                while col < EXPLORED_W {
                    if is_explored(&game.world, col, row) {
                        col += 1;
                        continue;
                    }
                    let rim_cell = is_explored_rim(&game.world, col, row);
                    let start = col;
                    while col < EXPLORED_W
                        && !is_explored(&game.world, col, row)
                        && is_explored_rim(&game.world, col, row) == rim_cell
                    {
                        col += 1;
                    }
                    let x0 = start as f32 / EXPLORED_W as f32;
                    let x1 = col as f32 / EXPLORED_W as f32;
                    let y0 = row as f32 / EXPLORED_H as f32;
                    let y1 = (row + 1) as f32 / EXPLORED_H as f32;
                    let ink = if rim_cell { rim } else { deep };
                    self.rect(&mut rc, [x0, y0, x1 - x0, y1 - y0], ink);
                    if rim_cell {
                        // Thin ash hairline toward the revealed side.
                        let h = (y1 - y0) * 0.35;
                        self.rect(&mut rc, [x0, y0 + (y1 - y0 - h) * 0.5, x1 - x0, h], ash);
                    }
                }
            }
        }
        for loc in crate::catalog::LOCATIONS {
            // Hide pins under the shroud until their cell is explored.
            if !crate::world::map_explored_at(&game.world, loc.x, loc.y) {
                continue;
            }
            let st = game.world.locations.get(loc.id).map(|s| s.status);
            let (tint, prop) = match st {
                Some(crate::world::WorldStatus::Beacon) => (
                    [0.92, 0.55, 0.18, 1.0],
                    loc_prop(loc.id, loc.kind),
                ),
                Some(crate::world::WorldStatus::Cleared) => {
                    ([0.38, 0.52, 0.32, 1.0], loc_prop(loc.id, loc.kind))
                }
                Some(crate::world::WorldStatus::Locked) => ([0.22, 0.20, 0.18, 0.75], loc_prop(loc.id, loc.kind)),
                Some(crate::world::WorldStatus::Dead) => ([0.55, 0.12, 0.10, 1.0], "kenney/prop/ruins.png"),
                _ => ([0.70, 0.62, 0.48, 1.0], loc_prop(loc.id, loc.kind)),
            };
            let (pw, ph) = prop_size(loc.kind);
            self.blit_px(
                &mut rc,
                self.tex(prop),
                [loc.x - pw * 0.5, loc.y - ph * 0.78, pw, ph],
                tint,
            );
            if st == Some(crate::world::WorldStatus::Beacon) {
                if let Some(ls) = game.world.locations.get(loc.id) {
                    if ls.hours_left > 0.0 {
                        self.text(
                            &mut rc,
                            &format!("{:.0}h", ls.hours_left),
                            loc.x - 0.018,
                            loc.y + ph * 0.18,
                            0.011,
                        );
                    }
                }
            } else if st == Some(crate::world::WorldStatus::Dead) {
                self.text(&mut rc, "DARK", loc.x - 0.018, loc.y + ph * 0.18, 0.011);
            }
        }
        // Clare: face walk direction; bob/lean share step_acc cadence with footfalls.
        let facing = if game.facing >= 0.0 { 1.0 } else { -1.0 };
        let step = if game.walking {
            (game.step_acc / crate::game::WORLD_STEP_PERIOD * std::f32::consts::TAU).sin()
        } else {
            (game.fx.time * 2.2).sin()
        };
        let bob = if game.walking {
            step.abs() * 0.014 + step * 0.003
        } else {
            (game.fx.time * 3.4).sin() * 0.004
        };
        let lean_x = if game.walking { facing * 0.007 } else { 0.0 };
        let lean_y = if game.walking { game.facing_y * 0.006 } else { 0.0 };
        let squash = if game.walking {
            1.0 + step.abs() * 0.09
        } else {
            1.0
        };
        let stretch = if game.walking {
            1.0 - step.abs() * 0.07
        } else {
            1.0
        };
        let base_w = 0.032 * squash;
        let base_h = 0.058 * stretch;
        let px = game.world.party_x + lean_x;
        let py = game.world.party_y + lean_y;
        self.blit_px(
            &mut rc,
            self.tex("kenney/prop/banner.png"),
            [
                px - 0.012,
                py - 0.038 + bob * 0.5 - lean_y * 0.4,
                0.024,
                0.046,
            ],
            [0.92, 0.88, 0.72, 1.0],
        );
        if self.images.contains_key("sprites/clare.png") {
            // Flip UVs when facing left so the portrait looks the way she walks.
            let uv = if facing < 0.0 {
                [1.0, 0.0, -1.0, 1.0]
            } else {
                [0.0, 0.0, 1.0, 1.0]
            };
            self.blit_uv(
                &mut rc,
                self.tex("sprites/clare.png"),
                self.sampler,
                [
                    px - base_w * 0.5,
                    py - base_h * 0.88 + bob,
                    base_w,
                    base_h,
                ],
                uv,
                [1.0, 1.0, 1.0, 1.0],
            );
        }
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel.png"),
            [0.0, 0.0, 1.0, 0.08],
            [0.42, 0.34, 0.26, 0.92],
        );
        self.text(
            &mut rc,
            &format!(
                "CLARE  NO.{}   {}   KARMA {}{}",
                game.world.rank,
                clock_label(game.world.hours),
                game.world.karma,
                if game.world.raki { "   RAKI" } else { "" }
            ),
            0.03,
            0.025,
            0.014,
        );
        self.prompt(&mut rc, "kenney/prompt/w.png", 0.03, 0.90, 0.04);
        self.prompt(&mut rc, "kenney/prompt/a.png", 0.075, 0.945, 0.04);
        self.prompt(&mut rc, "kenney/prompt/s.png", 0.12, 0.90, 0.04);
        self.prompt(&mut rc, "kenney/prompt/d.png", 0.165, 0.945, 0.04);
        self.text(&mut rc, "WALK", 0.22, 0.95, 0.012);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-brown.png",
            hud::world_codex(),
            "CODEX",
            false,
        );
        self.draw_fx(&mut rc, game);
    }

    pub(super) fn draw_town(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        let id = game
            .world
            .last_town
            .clone()
            .unwrap_or_else(|| "doga".into());
        let loc = crate::catalog::location(&id);
        let art = loc.map(|l| l.art).unwrap_or("art/tavern.jpg");
        self.blit(&mut rc, self.tex(art), [0.0, 0.0, 1.0, 1.0], [1.0; 4]);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.42]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.05, 0.07, 0.55, 0.28],
            [0.48, 0.38, 0.28, 0.78],
        );
        if let Some(loc) = loc {
            self.text(&mut rc, loc.name, 0.08, 0.10, 0.024);
            self.text(&mut rc, loc.region, 0.08, 0.16, 0.014);
            let dead = game
                .world
                .locations
                .get(loc.id)
                .map(|s| s.status == crate::world::WorldStatus::Dead)
                .unwrap_or(false);
            let blurb = if dead {
                crate::dialog::RESULT_LATE
            } else {
                loc.blurb
            };
            self.text(&mut rc, blurb, 0.08, 0.24, 0.012);
        }
        let nest = game
            .world
            .last_town
            .as_deref()
            .and_then(|id| game.world.locations.get(id))
            .map(|s| s.status == crate::world::WorldStatus::Dead)
            .unwrap_or(false);
        self.kenney_btn(
            &mut rc,
            if nest { "kenney/ui/button-red.png" } else { "kenney/ui/button.png" },
            hud::town_hunt(),
            if nest { "NEST" } else { "HUNT" },
            false,
        );
        self.kenney_btn(&mut rc, "kenney/ui/button-grey.png", hud::town_rest(), "REST", false);
        self.kenney_btn(&mut rc, "kenney/ui/button-brown.png", hud::town_leave(), "LEAVE", false);
        self.draw_fx(&mut rc, game);
    }
}
