use super::*;

impl Renderer {
    pub(super) fn draw_combat_overlay(
        &self,
        pass: &mut gpu::RenderCommandEncoder,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let Some(combat) = game.combat.as_ref() else {
            return;
        };
        let mut rc = pass.with(&self.flat);
        let size = board_size(combat.cols, combat.rows, w, h);
        let pan = Self::pan(game, w, h);
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            pan,
            game.ui.zoom,
        );
        let preview = game.preview_zone();
        let skill_on = game.ui.selected_skill.is_some();
        let overlay = if skill_on {
            "kenney/ui/hex-hit.png"
        } else {
            "kenney/ui/hex-move.png"
        };
        let otint = if skill_on {
            [0.85, 0.22, 0.16, 0.72]
        } else {
            [0.45, 0.62, 0.28, 0.65]
        };
        for hex in &preview {
            let (wx, wz) = axial_to_world(*hex, size);
            let (sx, sy) = world_to_iso(wx, size * 0.16, wz);
            let px = (ox + sx * game.ui.zoom) / w;
            let py = (oy + sy * game.ui.zoom) / h;
            let hw = (size * game.ui.zoom / w) * 1.2;
            let hh = (size * game.ui.zoom / h) * 0.7;
            self.blit_px(
                &mut rc,
                self.tex(overlay),
                [px - hw * 0.5, py - hh * 0.45, hw, hh],
                otint,
            );
        }
        for (hex, terrain) in &combat.terrain {
            if *terrain != Terrain::Ruin {
                continue;
            }
            let (wx, wz) = axial_to_world(*hex, size);
            let (sx, sy) = world_to_iso(wx, size * 0.42, wz);
            let px = (ox + sx * game.ui.zoom) / w;
            let py = (oy + sy * game.ui.zoom) / h;
            self.blit_px(
                &mut rc,
                self.tex("kenney/iso/column.png"),
                [px - 0.012, py - 0.055, 0.024, 0.07],
                [0.55, 0.5, 0.44, 0.9],
            );
        }
        for u in &combat.units {
            if u.dead {
                continue;
            }
            let c = core_hex(u);
            let (wx, wz) = axial_to_world(c, size);
            let (sx, sy) = world_to_iso(wx, size * 0.4, wz);
            let hash = (u.id.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
                as f32)
                * 0.01;
            let bob = (game.fx.time * 3.1 + hash).sin() * 0.007;
            let px = (ox + sx * game.ui.zoom) / w - 0.04;
            let py = (oy + sy * game.ui.zoom) / h - 0.12 + bob;
            let spr = if self.images.contains_key(&u.sprite) {
                self.tex(&u.sprite)
            } else {
                self.white.view
            };
            let squash = 1.0 + (game.fx.time * 3.1 + hash).sin() * 0.03;
            self.blit(
                &mut rc,
                spr,
                [px, py, 0.08 * (2.0 - squash), 0.16 * squash],
                [1.0, 1.0, 1.0, 1.0],
            );
            let frac = if u.max_hp > 0 {
                u.hp as f32 / u.max_hp as f32
            } else {
                0.0
            };
            self.bar(&mut rc, px, py - 0.014, 0.08, 0.01, frac, BLOOD);
            if u.trans > 0 {
                self.bar(
                    &mut rc,
                    px,
                    py - 0.024,
                    0.08,
                    0.008,
                    u.trans as f32 / 100.0,
                    [0.72, 0.22, 0.28, 1.0],
                );
            }
        }
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel.png"),
            [0.0, 0.85, 1.0, 0.15],
            [0.38, 0.30, 0.22, 0.92],
        );
        if let Some(u) = current_unit(combat) {
            self.blit(
                &mut rc,
                self.tex(&u.portrait),
                [0.012, 0.862, 0.08, 0.128],
                [1.0; 4],
            );
            self.text(
                &mut rc,
                &format!("{}  AP {}", u.name, u.ap),
                0.10,
                0.862,
                0.012,
            );
            self.bar(
                &mut rc,
                0.10,
                0.888,
                0.22,
                0.016,
                u.hp as f32 / u.max_hp.max(1) as f32,
                BLOOD,
            );
            self.bar(
                &mut rc,
                0.10,
                0.908,
                0.22,
                0.012,
                u.trans as f32 / 100.0,
                [0.7, 0.22, 0.26, 1.0],
            );
        }
        let bar = hud::combat_bar();
        let sel = game.ui.selected_skill.as_deref();
        self.kenney_btn(&mut rc, "kenney/ui/button-grey.png", bar.wait, "WAIT", false);
        self.kenney_btn(&mut rc, "kenney/ui/button.png", bar.raise, "RAISE", false);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-brown.png",
            bar.guard,
            "GUARD",
            sel == Some("guard"),
        );
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button.png",
            bar.cut,
            "CUT",
            sel == Some("cut"),
        );
        let slot_label = current_unit(combat)
            .and_then(|u| u.skills.get(4))
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_else(|| "—".into());
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-grey.png",
            bar.slot,
            &slot_label,
            false,
        );
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button-red.png",
            bar.forfeit,
            "FALL",
            false,
        );
        self.prompt(&mut rc, "kenney/prompt/space.png", bar.wait.x + 0.03, 0.868, 0.028);
        self.prompt(&mut rc, "kenney/prompt/1.png", bar.cut.x + 0.035, 0.868, 0.024);
        self.prompt(&mut rc, "kenney/prompt/2.png", bar.guard.x + 0.04, 0.868, 0.024);
        if let Some(line) = combat.log.first() {
            self.text(&mut rc, &line.text, 0.58, 0.862, 0.011);
        }
        self.text(&mut rc, combat.title.as_str(), 0.02, 0.02, 0.014);
        self.prompt(&mut rc, "kenney/prompt/esc.png", 0.92, 0.02, 0.04);
        self.draw_fx(&mut rc, game);
    }

}
