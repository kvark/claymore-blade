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
        let yaw = game.ui.yaw;
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            pan,
            game.ui.zoom,
            yaw,
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
            let (wx, wz) = axial_to_world_yaw(*hex, size, yaw);
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
            let (wx, wz) = axial_to_world_yaw(*hex, size, yaw);
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
            let (wx, wz) = axial_to_world_yaw(c, size, yaw);
            let hash = (u.id.bytes().fold(0u32, |a, b| a.wrapping_mul(31).wrapping_add(b as u32))
                as f32)
                * 0.01;
            let bob = (game.fx.time * 3.2 + hash).sin() * 0.004;
            let acting = Some(u.id.as_str()) == current_unit(combat).map(|x| x.id.as_str());
            // Archive GLB units already show a 3D body — skip the large floating
            // portrait so it does not occlude the mesh (HUD bar portrait stays).
            let has_mesh = super::board::unit_has_archive_mesh(self, &u.template_id, u.side);
            if has_mesh {
                let ground = combat
                    .terrain
                    .iter()
                    .find(|(h, _)| *h == c)
                    .map(|(_, tr)| terrain_height(*tr, size).max(0.0))
                    .unwrap_or(0.0);
                // Anchor HP/selection under the mesh feet (archive sits on hex top).
                let (sx, sy) = world_to_iso(wx, ground + size * 0.02, wz);
                let px = (ox + sx * game.ui.zoom) / w;
                let py = (oy + sy * game.ui.zoom) / h + bob;
                let bar_w = (size * game.ui.zoom / w) * 0.7;
                let hp = mesh_hp_bar(px, py, bar_w);
                if acting {
                    let ring = (size * game.ui.zoom / w) * 0.55;
                    self.blit_px(
                        &mut rc,
                        self.tex("kenney/ui/hex.png"),
                        [px - ring * 0.5, py - ring * 0.35, ring, ring * 0.7],
                        [0.95, 0.82, 0.35, 0.4],
                    );
                }
                self.bar(
                    &mut rc,
                    hp[0],
                    hp[1],
                    hp[2],
                    hp[3],
                    u.hp as f32 / u.max_hp.max(1) as f32,
                    BLOOD,
                );
                continue;
            }
            let (sx, sy) = world_to_iso(wx, size * 0.4, wz);
            let px = (ox + sx * game.ui.zoom) / w - 0.04;
            let py = (oy + sy * game.ui.zoom) / h - 0.12 + bob;
            let tint = if u.side == Side::Player {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.92, 0.55, 0.48, 1.0]
            };
            let portrait = [px, py, 0.08, 0.12];
            self.blit(
                &mut rc,
                self.tex(&u.portrait),
                portrait,
                tint,
            );
            if acting {
                self.blit_px(
                    &mut rc,
                    self.tex("kenney/ui/hex.png"),
                    [px - 0.005, py - 0.01, 0.09, 0.14],
                    [0.95, 0.82, 0.35, 0.55],
                );
            }
            let hp = portrait_hp_bar(portrait[0], portrait[1], portrait[2], portrait[3]);
            self.bar(
                &mut rc,
                hp[0],
                hp[1],
                hp[2],
                hp[3],
                u.hp as f32 / u.max_hp.max(1) as f32,
                BLOOD,
            );
        }
        let bar = hud::combat_bar();
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel.png"),
            [0.0, bar.wait.y - 0.03, 1.0, 1.0 - (bar.wait.y - 0.03)],
            [0.32, 0.26, 0.20, 0.92],
        );
        if let Some(u) = current_unit(combat) {
            let py = bar.wait.y - 0.018;
            self.blit(
                &mut rc,
                self.tex(&u.portrait),
                [0.012, py, 0.08, 0.118],
                [1.0; 4],
            );
            self.text(
                &mut rc,
                &format!("{}  AP {}", u.name, u.ap),
                0.10,
                py,
                0.012,
            );
            self.bar(
                &mut rc,
                0.10,
                py + 0.026,
                0.22,
                0.016,
                u.hp as f32 / u.max_hp.max(1) as f32,
                BLOOD,
            );
            // Trans is danger (open the bar), not a second HP bar — number + band + tint.
            self.bar(
                &mut rc,
                0.10,
                py + 0.046,
                0.22,
                0.012,
                u.trans as f32 / 100.0,
                trans_fill_tint(u.trans),
            );
            self.text_tint(
                &mut rc,
                &trans_meter_label(u.trans),
                0.33,
                py + 0.044,
                0.010,
                ASH_TYPE,
            );
        }
        let sel = game.ui.selected_skill.as_deref();
        // Larger ash glyphs on the bar only — inactive WAIT/RAISE were weak tan-on-tan.
        self.kenney_btn_ex(&mut rc, "kenney/ui/button-grey.png", bar.wait, "WAIT", false, 0.026);
        self.kenney_btn_ex(&mut rc, "kenney/ui/button.png", bar.raise, "RAISE", false, 0.026);
        self.kenney_btn_ex(
            &mut rc,
            "kenney/ui/button-brown.png",
            bar.guard,
            "GUARD",
            sel == Some("guard"),
            0.026,
        );
        self.kenney_btn_ex(
            &mut rc,
            "kenney/ui/button.png",
            bar.cut,
            "CUT",
            sel == Some("cut"),
            0.026,
        );
        let slot_id = current_unit(combat)
            .and_then(|u| u.skills.get(4))
            .map(|s| s.as_str());
        let slot_label = slot_id
            .map(|s| s.to_ascii_uppercase())
            .unwrap_or_else(|| "—".into());
        let slot_hot = match (sel, slot_id) {
            (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
            _ => false,
        };
        let slot_ready = match (
            current_unit(combat),
            slot_id.and_then(|id| crate::catalog::skill(id)),
        ) {
            (Some(u), Some(skill)) => skill_slot_ready(u.trans, skill.trans),
            _ => false,
        };
        // Grey = locked under the skill's trans gate; brown plate when the bar is open.
        let slot_tex = if slot_ready {
            "kenney/ui/button-brown.png"
        } else {
            "kenney/ui/button-grey.png"
        };
        self.kenney_btn_ex(
            &mut rc,
            slot_tex,
            bar.slot,
            &slot_label,
            slot_hot,
            0.026,
        );
        self.kenney_btn_ex(
            &mut rc,
            "kenney/ui/button-red.png",
            bar.forfeit,
            "FALL",
            false,
            0.026,
        );
        let prompt_y = bar.wait.y - 0.028;
        self.prompt(&mut rc, "kenney/prompt/space.png", bar.wait.x + 0.03, prompt_y, 0.028);
        self.prompt(&mut rc, "kenney/prompt/1.png", bar.cut.x + 0.035, prompt_y, 0.024);
        self.prompt(&mut rc, "kenney/prompt/2.png", bar.guard.x + 0.04, prompt_y, 0.024);
        self.prompt(&mut rc, "kenney/prompt/3.png", bar.slot.x + 0.045, prompt_y, 0.024);
        if let Some(line) = combat.log.first() {
            self.text(&mut rc, &line.text, 0.58, bar.wait.y - 0.018, 0.011);
        }
        self.text(&mut rc, combat.title.as_str(), 0.02, 0.02, 0.014);
        self.text(&mut rc, "Q / E  ROTATE", 0.72, 0.02, 0.011);
        self.prompt(&mut rc, "kenney/prompt/esc.png", 0.92, 0.02, 0.04);
        if game.esc_arm > 0.0 {
            self.text_tint(
                &mut rc,
                "ESC AGAIN TO FLEE",
                0.34,
                bar.wait.y - 0.05,
                0.016,
                [0.95, 0.82, 0.45, 1.0],
            );
        }
        self.draw_fx(&mut rc, game);
    }
}

/// DESIGN §10 band short name for the combat HUD (open the bar, not power-up).
pub(crate) fn trans_band(trans: i32) -> &'static str {
    match trans.clamp(0, 100) {
        0..=19 => "Human",
        20..=39 => "Silver",
        40..=59 => "Edge",
        60..=79 => "Demon",
        80..=99 => "Breaking",
        _ => "Awake",
    }
}

/// Fill tint by band: ash → steel → blood danger from Edge up (never cool gold).
pub(crate) fn trans_fill_tint(trans: i32) -> [f32; 4] {
    // Blood #9a2430 for Edge+ only (DESIGN palette — high trans / danger chips).
    const TRANS_DANGER: [f32; 4] = [0.604, 0.141, 0.188, 1.0];
    match trans.clamp(0, 100) {
        0..=19 => [0.72, 0.64, 0.52, 1.0],   // ash
        20..=39 => [0.784, 0.800, 0.831, 1.0], // steel
        _ => TRANS_DANGER,
    }
}

/// e.g. `BAR 23 SILVER` — reads as the yoki meter, not a second HP bar.
pub(crate) fn trans_meter_label(trans: i32) -> String {
    let t = trans.clamp(0, 100);
    format!("BAR {} {}", t, trans_band(t).to_ascii_uppercase())
}

/// Skill slot plate lights when unit trans meets the skill's gate (Flash ≥40).
pub(crate) fn skill_slot_ready(unit_trans: i32, skill_trans: i32) -> bool {
    unit_trans >= skill_trans
}

/// Tiny blood HP bar under a board combat portrait (`[x,y,w,h]` normalized).
pub(crate) fn portrait_hp_bar(px: f32, py: f32, pw: f32, ph: f32) -> [f32; 4] {
    const BAR_H: f32 = 0.01;
    const GAP: f32 = 0.002;
    [px, py + ph + GAP, pw, BAR_H]
}

/// HP bar centered under an archive-mesh unit (no floating portrait).
pub(crate) fn mesh_hp_bar(cx: f32, cy: f32, bar_w: f32) -> [f32; 4] {
    const BAR_H: f32 = 0.01;
    const GAP: f32 = 0.006;
    [cx - bar_w * 0.5, cy + GAP, bar_w, BAR_H]
}

#[cfg(test)]
mod tests {
    use super::{
        mesh_hp_bar, portrait_hp_bar, skill_slot_ready, trans_band, trans_fill_tint,
        trans_meter_label,
    };

    #[test]
    fn trans_bands_match_design() {
        assert_eq!(trans_band(0), "Human");
        assert_eq!(trans_band(19), "Human");
        assert_eq!(trans_band(20), "Silver");
        assert_eq!(trans_band(39), "Silver");
        assert_eq!(trans_band(40), "Edge");
        assert_eq!(trans_band(59), "Edge");
        assert_eq!(trans_band(60), "Demon");
        assert_eq!(trans_band(80), "Breaking");
        assert_eq!(trans_band(100), "Awake");
    }

    #[test]
    fn trans_tint_is_ash_then_steel_then_blood() {
        let ash = trans_fill_tint(8);
        let steel = trans_fill_tint(23);
        let edge = trans_fill_tint(40);
        let awake = trans_fill_tint(100);
        assert!(ash[0] > ash[2], "Human band stays warm ash, not blood");
        assert!(steel[2] > steel[0], "Silver band is steel-cool");
        assert!(
            edge[0] > edge[1] && edge[0] > edge[2],
            "Edge+ must read blood/danger, not gold"
        );
        assert!((edge[0] - awake[0]).abs() < 1e-6, "Awake stays danger, not cool gold");
        // Never gold: G should not dominate R on high bands.
        assert!(edge[1] < edge[0] * 0.5);
    }

    #[test]
    fn trans_meter_label_shows_number_and_band() {
        assert_eq!(trans_meter_label(23), "BAR 23 SILVER");
        assert_eq!(trans_meter_label(40), "BAR 40 EDGE");
        assert_eq!(trans_meter_label(8), "BAR 8 HUMAN");
    }

    #[test]
    fn flash_slot_ready_at_gate_40() {
        assert!(!skill_slot_ready(39, 40));
        assert!(skill_slot_ready(40, 40));
        assert!(skill_slot_ready(53, 40));
    }

    #[test]
    fn portrait_hp_bar_sits_under_portrait() {
        let r = portrait_hp_bar(0.1, 0.2, 0.08, 0.12);
        assert!((r[0] - 0.1).abs() < 1e-6);
        assert!((r[1] - 0.322).abs() < 1e-6);
        assert!((r[2] - 0.08).abs() < 1e-6);
        assert!(r[3] >= 0.008 && r[3] <= 0.012);
    }

    #[test]
    fn mesh_hp_bar_centered_under_unit() {
        let r = mesh_hp_bar(0.5, 0.4, 0.06);
        assert!((r[0] - 0.47).abs() < 1e-6);
        assert!(r[1] > 0.4);
        assert!((r[2] - 0.06).abs() < 1e-6);
        assert!(r[3] >= 0.008 && r[3] <= 0.012);
    }
}
