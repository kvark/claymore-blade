use super::*;
use super::chrome;

impl Renderer {
    /// Move / skill / claw-threat hex overlays, drawn between the board cells
    /// and the fighters so a translucent overlay never washes over a body.
    pub(super) fn draw_hex_overlays(
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
        let zoom = game.view_camera().1;
        let yaw = game.ui.yaw;
        let (ox, oy) = camera_origin(combat.cols, combat.rows, size, w, h, pan, zoom, yaw);
        let preview = game.preview_zone();
        let threat = game.threat_zone();
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
            let px = (ox + sx * zoom) / w;
            let py = (oy + sy * zoom) / h;
            let hw = (size * zoom / w) * 1.2;
            let hh = (size * zoom / h) * 0.7;
            self.blit_px(
                &mut rc,
                self.tex(overlay),
                [px - hw * 0.5, py - hh * 0.45, hw, hh],
                otint,
            );
        }
        // Yoma claw tell — same hex language as skill preview, blood/danger tint.
        let ttint = [0.78, 0.12, 0.14, 0.78];
        for hex in &threat {
            let (wx, wz) = axial_to_world_yaw(*hex, size, yaw);
            let (sx, sy) = world_to_iso(wx, size * 0.16, wz);
            let px = (ox + sx * zoom) / w;
            let py = (oy + sy * zoom) / h;
            let hw = (size * zoom / w) * 1.2;
            let hh = (size * zoom / h) * 0.7;
            self.blit_px(
                &mut rc,
                self.tex("kenney/ui/hex-hit.png"),
                [px - hw * 0.5, py - hh * 0.45, hw, hh],
                ttint,
            );
        }
    }

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
        let zoom = game.view_camera().1;
        let yaw = game.ui.yaw;
        let (ox, oy) = camera_origin(
            combat.cols,
            combat.rows,
            size,
            w,
            h,
            pan,
            zoom,
            yaw,
        );
        for (hex, terrain) in &combat.terrain {
            if *terrain != Terrain::Ruin {
                continue;
            }
            let (wx, wz) = axial_to_world_yaw(*hex, size, yaw);
            let (sx, sy) = world_to_iso(wx, size * 0.42, wz);
            let px = (ox + sx * zoom) / w;
            let py = (oy + sy * zoom) / h;
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
                let px = (ox + sx * zoom) / w;
                let py = (oy + sy * zoom) / h + bob;
                let bar_w = (size * zoom / w) * 0.7;
                let hp = mesh_hp_bar(px, py, bar_w);
                if acting {
                    let ring = (size * zoom / w) * 0.55;
                    self.blit_px(
                        &mut rc,
                        self.tex("kenney/ui/hex.png"),
                        [px - ring * 0.5, py - ring * 0.35, ring, ring * 0.7],
                        [0.95, 0.82, 0.35, 0.4],
                    );
                }
                self.ink_bar(&mut rc, hp, u.hp as f32 / u.max_hp.max(1) as f32, BLOOD, w, h);
                continue;
            }
            let ground = combat.terrain.iter()
                .find(|(hex, _)| *hex == c)
                .map(|(_, terrain)| terrain_height(*terrain, size).max(0.0))
                .unwrap_or(0.0);
            let (sx, sy) = world_to_iso(wx, ground + size * 0.02, wz);
            let px = (ox + sx * zoom) / w;
            let py = (oy + sy * zoom) / h + bob;
            let tint = if u.side == Side::Player {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.92, 0.55, 0.48, 1.0]
            };
            let aspect = self.images.get(&u.portrait).map_or(1.0, |texture| texture.aspect);
            let body = if u.side == Side::Enemy {
                super::board::enemy_portrait_scale()
                    * super::board::unit_tier_scale(&u.template_id, u.side)
            } else {
                1.0
            };
            let portrait = board_portrait_rect(px, py, size * zoom * body, w, h, aspect);
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
                    [portrait[0] - 2.0 / w, portrait[1] - 2.0 / h,
                        portrait[2] + 4.0 / w, portrait[3] + 4.0 / h],
                    [0.95, 0.82, 0.35, 0.55],
                );
            }
            let hp = portrait_hp_bar(portrait[0], portrait[1], portrait[2], portrait[3]);
            self.ink_bar(&mut rc, hp, u.hp as f32 / u.max_hp.max(1) as f32, BLOOD, w, h);
        }
        let bar = hud::combat_bar();
        let panel_top = bar.wait.y - 0.03;
        self.nest_vignette(&mut rc, panel_top);
        self.draw_cutin(&mut rc, game, w, h);
        // Ink action panel with a worn-steel seam over a dried-blood hairline.
        self.rect(&mut rc, [0.0, panel_top, 1.0, 1.0 - panel_top], chrome::INK);
        self.rect(&mut rc, [0.0, panel_top, 1.0, 1.0 / h], chrome::STEEL_EDGE);
        self.rect(&mut rc, [0.0, panel_top + 1.0 / h, 1.0, 2.0 / h], chrome::BLOOD_INK);
        if let Some(u) = current_unit(combat) {
            // Unit card sits above the action panel (it used to hide under
            // WAIT / RAISE), so HP, trans and the BAR N BAND label all read.
            let card = unit_card(bar.wait.y);
            self.rect(&mut rc, card.plate, chrome::INK);
            self.steel_frame(&mut rc, card.plate, w, h);
            self.blit(&mut rc, self.tex(&u.portrait), card.portrait, [1.0; 4]);
            self.steel_frame(&mut rc, card.portrait, w, h);
            self.text(&mut rc, &format!("{}  AP {}", u.name, u.ap), card.name[0], card.name[1], card.name[3]);
            self.ink_bar(&mut rc, card.hp, u.hp as f32 / u.max_hp.max(1) as f32, BLOOD, w, h);
            // Trans is danger (open the bar), not a second HP bar — number + band + tint.
            self.ink_bar(&mut rc, card.trans, u.trans as f32 / 100.0, trans_fill_tint(u.trans), w, h);
            self.text_tint(&mut rc, &trans_meter_label(u.trans), card.label[0], card.label[1], card.label[3], ASH_TYPE);
        }
        let sel = game.ui.selected_skill.as_deref();
        // Same predicate as input: turn buttons dim while yoma beats / windups play.
        let live = game.combat_input_open();
        // Larger ash glyphs on the bar only — inactive WAIT/RAISE were weak tan-on-tan.
        self.ink_btn(&mut rc, bar.wait, "WAIT", false, false, false, live, w, h);
        self.ink_btn(&mut rc, bar.raise, "RAISE", false, false, false, live, w, h);
        self.ink_btn(&mut rc, bar.guard, "GUARD", sel == Some("guard"), false, false, live, w, h);
        self.ink_btn(&mut rc, bar.cut, "CUT", sel == Some("cut"), false, false, live, w, h);
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
        // Blood tick = the trans gate is open; a locked slot keeps a bare plate.
        self.ink_btn(&mut rc, bar.slot, &slot_label, slot_hot, slot_ready, false, live, w, h);
        // FALL stays lit: forfeit is a retreat, not a turn action, and it is
        // accepted during enemy beats (like Esc-Esc flee).
        self.ink_btn(&mut rc, bar.forfeit, "FALL", false, false, true, true, w, h);
        let prompt_y = bar.wait.y - 0.028;
        self.prompt_state(&mut rc, "kenney/prompt/space.png", bar.wait.x + 0.03, prompt_y, 0.028, live);
        self.prompt_state(&mut rc, "kenney/prompt/1.png", bar.cut.x + 0.035, prompt_y, 0.024, live);
        self.prompt_state(&mut rc, "kenney/prompt/2.png", bar.guard.x + 0.04, prompt_y, 0.024, live);
        self.prompt_state(&mut rc, "kenney/prompt/3.png", bar.slot.x + 0.045, prompt_y, 0.024, live);
        if !live && combat.over.is_none() {
            // Enemy phase caption over the dimmed row, plus the fast-forward hint.
            let ff = game.fast_forwarding();
            let (text, tint) = enemy_phase_caption(ff);
            self.text_tint(&mut rc, text, bar.wait.x + 0.005, trans_meter_label_y(bar.wait.y), 0.011, tint);
        }
        if let Some(line) = combat.log.last() {
            self.text(&mut rc, &line.text, 0.58, bar.wait.y - 0.018, 0.011);
        }
        self.text(&mut rc, combat.title.as_str(), 0.02, 0.02, 0.014);
        self.text(&mut rc, "Q / E  ROTATE", 0.72, 0.02, 0.011);
        self.prompt(&mut rc, "kenney/prompt/esc.png", 0.92, 0.02, 0.04);
        self.draw_turn_queue(&mut rc, combat, w, h);
        if game.fast_forwarding() {
            // Small "x2" indicator under the rotate hint while beats are sped up.
            self.text_tint(&mut rc, "X2  FAST", 0.72, 0.05, 0.016, [0.95, 0.82, 0.45, 1.0]);
        }
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

    /// Anime cut-in (claw: blood-ink, FLASH: pale yoki glare): vignette + letterbox bar, and horizontal
    /// speed streaks rushing in from the edges that stop short of the board,
    /// its units and the caption. Drawn under the HUD so buttons and the
    /// queue stay crisp.
    fn draw_cutin(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let (Some(cut), Some(combat)) = (game.cutin.as_ref(), game.combat.as_ref()) else {
            return;
        };
        let k = cut.strength(game.fx.time);
        if k <= 0.01 {
            return;
        }
        let Some(u) = combat.units.iter().find(|u| u.id == cut.unit_id) else {
            return;
        };
        let focus = game.hex_screen(core_hex(u));
        let panel_top = hud::combat_bar().wait.y - 0.03;
        use crate::game::beats::{self, CutInKind};
        let (wash, ink) = match cut.kind {
            CutInKind::Claw => (beats::cutin_wash(k, panel_top), [0.96, 0.93, 0.86]),
            CutInKind::Flash => (beats::flash_wash(k, panel_top), beats::FLASH_STREAK),
        };
        for (r, c) in wash {
            self.rect(rc, r, c);
        }
        let keep_out = game.cutin_keep_out();
        for l in beats::speed_lines(k, game.fx.time, focus, w / h.max(1.0), &keep_out) {
            self.rect(rc, [l[0], l[1], l[2], l[3]], [ink[0], ink[1], ink[2], l[4]]);
        }
    }

    /// Compact initiative strip under the title — portraits + short names, current ringed.
    pub(super) fn draw_turn_queue(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        combat: &CombatState,
        w: f32,
        h: f32,
    ) {
        let ids = queue_ids(&combat.order, combat.turn, &combat.units);
        if ids.is_empty() {
            return;
        }
        // Top edge, centered — clear of skill bar, trans meter, and clickable hexes.
        const CHIP_W: f32 = 0.052;
        const CHIP_H: f32 = 0.070;
        const GAP: f32 = 0.010;
        const Y: f32 = 0.048;
        const MAX: usize = 8;
        let show: Vec<&String> = ids.iter().take(MAX).collect();
        let n = show.len() as f32;
        let total_w = n * CHIP_W + (n - 1.0).max(0.0) * GAP;
        let start_x = ((1.0 - total_w) * 0.5).max(0.02);
        // Ink plate behind the strip (existing HUD language).
        let plate = [start_x - 0.012, Y - 0.010, total_w + 0.024, CHIP_H + 0.028];
        self.rect(rc, plate, chrome::INK);
        self.steel_frame(rc, plate, w, h);
        for (i, id) in show.iter().enumerate() {
            let Some(u) = combat.units.iter().find(|u| u.id == **id) else {
                continue;
            };
            let x = start_x + i as f32 * (CHIP_W + GAP);
            let portrait = [x, Y + 0.006, CHIP_W, CHIP_H * 0.72];
            if i == 0 {
                // Same gold ring language as acting units on the board; blood
                // when a yoma owns the beat so the enemy phase reads at a glance.
                self.blit_px(
                    rc,
                    self.tex("kenney/ui/hex.png"),
                    [x - 0.005, Y + 0.001, CHIP_W + 0.010, CHIP_H * 0.80],
                    queue_ring_tint(u.side),
                );
            }
            let tint = if u.side == Side::Player {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.92, 0.55, 0.48, 1.0]
            };
            self.blit(rc, self.tex(&u.portrait), portrait, tint);
            let label = queue_chip_label(&u.name);
            let glyph_h = 0.009;
            let glyph_w = glyph_h * 0.7;
            let nch = label.chars().count() as f32;
            let text_w = if nch > 0.0 {
                glyph_w * ((nch - 1.0) * 0.85 + 1.0)
            } else {
                0.0
            };
            let tx = x + (CHIP_W - text_w) * 0.5;
            let ty = Y + CHIP_H * 0.78;
            self.text_tint(rc, &label, tx, ty, glyph_h, ASH_TYPE);
        }
    }
}

/// Current-chip ring: gold for the party, blood for an enemy beat.
/// Ink veil laid over a disabled combat button plate (darkens + desaturates).
pub(crate) const DISABLED_VEIL: [f32; 4] = [0.03, 0.026, 0.022, 0.72];
/// Dim ash for a disabled button label.
pub(crate) const DISABLED_TYPE: [f32; 4] = [0.40, 0.38, 0.36, 0.62];
/// Dim key-hint chip tint while turn input is blocked.
pub(crate) const DISABLED_PROMPT: [f32; 4] = [0.40, 0.38, 0.34, 0.28];

/// Caption over the dimmed bar during the enemy phase.
pub(crate) fn enemy_phase_caption(fast: bool) -> (&'static str, [f32; 4]) {
    if fast {
        ("YOMA ACT  X2", [0.95, 0.82, 0.45, 1.0])
    } else {
        ("YOMA ACT  HOLD SHIFT X2", [0.72, 0.64, 0.52, 0.9])
    }
}

pub(crate) fn queue_ring_tint(side: Side) -> [f32; 4] {
    match side {
        Side::Player => [0.95, 0.82, 0.35, 0.55],
        Side::Enemy => [0.86, 0.20, 0.16, 0.80],
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

/// Normalized Y for the trans label: clear of the action-button tops (`button_y`).
pub(crate) fn trans_meter_label_y(button_y: f32) -> f32 {
    button_y - 0.040
}

/// Current-unit card above the action panel: `[x, y, w, h]` rects. Text
/// rects give the glyph origin and height (`[x, y, width, glyph_h]`).
pub(crate) struct UnitCard {
    pub plate: [f32; 4],
    pub portrait: [f32; 4],
    pub name: [f32; 4],
    pub hp: [f32; 4],
    pub trans: [f32; 4],
    pub label: [f32; 4],
}

/// Laid out from the button row's top so it clears WAIT / RAISE and the
/// enemy-phase caption line (`trans_meter_label_y`) under it.
pub(crate) fn unit_card(button_y: f32) -> UnitCard {
    let bottom = trans_meter_label_y(button_y) - 0.010;
    let plate = [0.012, bottom - 0.086, 0.25, 0.086];
    let (x, y) = (plate[0] + 0.058, plate[1]);
    UnitCard {
        plate,
        portrait: [plate[0] + 0.006, y + 0.007, 0.046, plate[3] - 0.014],
        name: [x, y + 0.010, 0.18, 0.012],
        hp: [x, y + 0.030, 0.18, 0.014],
        trans: [x, y + 0.050, 0.18, 0.010],
        label: [x, y + 0.066, 0.18, 0.011],
    }
}

/// Skill slot plate lights when unit trans meets the skill's gate (Flash ≥40).
pub(crate) fn skill_slot_ready(unit_trans: i32, skill_trans: i32) -> bool {
    unit_trans >= skill_trans
}


/// Living initiative queue starting at `turn` (wraps). Current actor first; dead skipped.
pub(crate) fn queue_ids(order: &[String], turn: usize, units: &[crate::combat::Unit]) -> Vec<String> {
    if order.is_empty() {
        return Vec::new();
    }
    let n = order.len();
    let start = turn % n;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let id = &order[(start + i) % n];
        let alive = units.iter().any(|u| u.id == *id && !u.dead);
        if alive {
            out.push(id.clone());
        }
    }
    out
}

/// Compact chip label — short enough for the top-edge queue.
pub(crate) fn queue_chip_label(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "?".into();
    }
    let mut out = String::new();
    for (i, ch) in trimmed.chars().enumerate() {
        if i >= 5 {
            break;
        }
        out.push(ch.to_ascii_uppercase());
    }
    out
}

/// Tile-scaled fallback portrait, centered over its feet and never stretched.
pub(crate) fn board_portrait_rect(cx: f32, feet_y: f32, tile_radius: f32,
    w: f32, h: f32, aspect: f32) -> [f32; 4] {
    let aspect = aspect.max(0.01);
    let max_width = (tile_radius * 1.4).clamp(8.0, 56.0);
    let max_height = (tile_radius * 2.0).clamp(12.0, 84.0);
    let height_px = max_height.min(max_width / aspect);
    let pw = height_px * aspect / w.max(1.0);
    let ph = height_px / h.max(1.0);
    [cx - pw * 0.5, feet_y - ph, pw, ph]
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
        board_portrait_rect, mesh_hp_bar, portrait_hp_bar, queue_chip_label, queue_ids, skill_slot_ready, trans_band,
        trans_fill_tint, trans_meter_label, trans_meter_label_y, unit_card,
    };
    use crate::combat::{Side, Stats, Unit};
    use crate::hex::Axial;

    #[test]
    fn board_portraits_preserve_aspect_and_feet_at_every_scale() {
        for (w, h) in [(390.0, 844.0), (1280.0, 800.0), (1920.0, 1080.0)] {
            for radius in [5.0, 20.0, 62.0, 96.0] {
                for aspect in [2.0 / 3.0, 871.0 / 1093.0, 636.0 / 1255.0] {
                    let p = board_portrait_rect(0.45, 0.6, radius, w, h, aspect);
                    assert!((p[0] + p[2] * 0.5 - 0.45).abs() < 1e-6);
                    assert!((p[1] + p[3] - 0.6).abs() < 1e-6);
                    assert!((p[2] * w / (p[3] * h) - aspect).abs() < 1e-5);
                    assert!(p[2] * w <= 56.01 && p[3] * h <= 84.01);
                    assert!(p[2] * w <= (radius * 1.4).max(8.0) + 0.01);
                }
            }
        }
    }

    #[test]
    fn board_portraits_follow_zoom_without_covering_neighboring_columns() {
        let small = board_portrait_rect(0.5, 0.5, 20.0 * 0.7, 1280.0, 800.0, 2.0 / 3.0);
        let normal = board_portrait_rect(0.5, 0.5, 20.0, 1280.0, 800.0, 2.0 / 3.0);
        let large = board_portrait_rect(0.5, 0.5, 20.0 * 1.55, 1280.0, 800.0, 2.0 / 3.0);
        assert!(small[2] < normal[2] && normal[2] < large[2]);
        assert!(normal[2] * 1280.0 < 30.0); // adjacent q centers are 1.5 tiles apart
        assert!((normal[3] * 800.0 - 40.0).abs() < 1e-4);
    }

    fn stub_unit(id: &str, dead: bool) -> Unit {
        Unit {
            id: id.into(),
            template_id: id.into(),
            name: id.into(),
            title: String::new(),
            rank: 1,
            side: Side::Player,
            portrait: String::new(),
            sprite: String::new(),
            origin: Axial::new(0, 0),
            facing: 0,
            footprint: vec![Axial::new(0, 0)],
            core_index: 0,
            parts: Vec::new(),
            hp: if dead { 0 } else { 10 },
            max_hp: 10,
            yoki: 0,
            max_yoki: 0,
            trans: 0,
            ap: 2,
            max_ap: 2,
            stats: Stats {
                s: 1,
                a: 1,
                c: 1,
                p: 1,
                w: 1,
            },
            skills: Vec::new(),
            statuses: Vec::new(),
            raised_trans: false,
            next_hint: None,
            color: [1.0, 1.0, 1.0],
            dead,
        }
    }

    #[test]
    fn queue_ids_wraps_skips_dead_current_first() {
        let order = vec![
            "clare".into(),
            "yoma-0".into(),
            "raki".into(),
            "yoma-1".into(),
        ];
        let units = vec![
            stub_unit("clare", false),
            stub_unit("yoma-0", true),
            stub_unit("raki", false),
            stub_unit("yoma-1", false),
        ];
        // turn at raki → raki, yoma-1, clare (dead yoma-0 skipped; wrap)
        assert_eq!(
            queue_ids(&order, 2, &units),
            vec!["raki".to_string(), "yoma-1".into(), "clare".into()]
        );
        // turn at start → clare first, skip dead
        assert_eq!(
            queue_ids(&order, 0, &units),
            vec!["clare".to_string(), "raki".into(), "yoma-1".into()]
        );
        // empty order
        assert!(queue_ids(&[], 0, &units).is_empty());
    }

    #[test]
    fn queue_chip_label_is_short_ash() {
        assert_eq!(queue_chip_label("Clare"), "CLARE");
        assert_eq!(queue_chip_label("Yoma"), "YOMA");
        assert_eq!(queue_chip_label("Stretch"), "STRET");
    }

    #[test]
    fn doga_queue_lists_living_from_current() {
        use crate::combat::{act, create_battle, current_unit, PlayerAction};
        let enc = crate::catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 7);
        let q0 = queue_ids(&s.order, s.turn, &s.units);
        assert!(!q0.is_empty());
        assert_eq!(q0[0], current_unit(&s).unwrap().id);
        assert!(q0.iter().all(|id| s.units.iter().any(|u| u.id == *id && !u.dead)));
        // Clare typically leads Doga; after her Wait the next living id heads the queue.
        if current_unit(&s).map(|u| u.id.as_str()) == Some("clare") {
            act(&mut s, PlayerAction::Wait, true);
            let q1 = queue_ids(&s.order, s.turn, &s.units);
            assert_ne!(q1[0], "clare");
            assert_eq!(q1[0], current_unit(&s).unwrap().id);
        }
    }

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
    fn trans_meter_label_sits_above_action_buttons() {
        // combat_bar wait.y is 0.865; old label used py+0.044 ≈ 0.891 (under plates).
        let button_y = 0.865;
        let y = trans_meter_label_y(button_y);
        assert!(
            y < button_y - 0.01,
            "label y={y} must clear button tops at {button_y}"
        );
        assert!(y > button_y - 0.08, "label should stay on the combat panel strip");
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

    fn overlaps(a: [f32; 4], b: [f32; 4]) -> bool {
        a[0] < b[0] + b[2] && a[0] + a[2] > b[0] && a[1] < b[1] + b[3] && a[1] + a[3] > b[1]
    }

    #[test]
    fn unit_card_bars_clear_the_buttons_and_read() {
        let bar = crate::hud::combat_bar();
        let buttons = [bar.wait, bar.raise, bar.guard, bar.cut, bar.slot, bar.forfeit];
        let panel_top = bar.wait.y - 0.03;
        let card = unit_card(bar.wait.y);
        // Longest name + AP and band label must fit the text rects.
        let text_w = |s: &str, g: f32| g * 0.7 * ((s.chars().count() as f32 - 1.0) * 0.85 + 1.0);
        assert!(text_w("DENEVE  AP 4", card.name[3]) <= card.name[2]);
        assert!(text_w(&trans_meter_label(100), card.label[3]) <= card.label[2]);
        let caption_y = trans_meter_label_y(bar.wait.y);
        for (w, h) in [(390.0, 844.0), (1024.0, 768.0), (1280.0, 800.0), (1920.0, 1080.0)] {
            for r in [card.plate, card.portrait, card.name, card.hp, card.trans, card.label] {
                for b in buttons {
                    assert!(!overlaps(r, b.pos()), "{r:?} under button {:?} at {w}x{h}", b.pos());
                }
                // Above the action panel and the enemy-phase caption line.
                assert!(r[1] + r[3] <= panel_top && r[1] + r[3] <= caption_y, "{r:?}");
                assert!(r[0] >= 0.0 && r[0] + r[2] <= 1.0 && r[1] > 0.5);
                // Inside the ink plate.
                let p = card.plate;
                assert!(r[0] >= p[0] && r[1] >= p[1] && r[0] + r[2] <= p[0] + p[2] + 1e-6 && r[1] + r[3] <= p[1] + p[3] + 1e-6);
            }
            // Bars stay readable: HP at least 8 px tall, trans at least 6 px.
            assert!(card.hp[3] * h >= 8.0 && card.trans[3] * h >= 6.0, "{w}x{h}");
            assert!(card.hp[2] * w >= 60.0);
        }
        // Rows stack without overlapping each other.
        let rows = [card.name, card.hp, card.trans, card.label];
        for i in 0..rows.len() {
            for j in i + 1..rows.len() {
                assert!(!overlaps(rows[i], rows[j]), "{:?} vs {:?}", rows[i], rows[j]);
            }
            assert!(!overlaps(rows[i], card.portrait));
        }
    }
}
