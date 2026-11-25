//! Claymore combat chrome: ink plates, worn-steel hairlines, blood accents.
//! Same rects and actions as before; only the paint changes. Tints are
//! linear (the swapchain is sRGB): 0.004 here is ~#0e0c0b on screen.
use super::*;

/// Ink plate fill (near-black, faintly warm).
pub(crate) const INK: [f32; 4] = [0.0045, 0.0038, 0.0034, 1.0];
/// Worn steel hairline, lit edge.
pub(crate) const STEEL_EDGE: [f32; 4] = [0.22, 0.23, 0.25, 0.75];
/// Worn steel hairline, shadowed edge.
pub(crate) const STEEL_SHADOW: [f32; 4] = [0.012, 0.012, 0.014, 0.95];
/// Dried-blood accent (hot ticks, danger plate, panel seam).
pub(crate) const BLOOD_INK: [f32; 4] = [0.20, 0.008, 0.007, 0.95];
/// Forfeit plate: deep blood instead of candy red.
pub(crate) const BLOOD_PLATE: [f32; 4] = [0.075, 0.004, 0.004, 0.96];
/// Crit splash: fresh blood, the one saturated colour on screen.
pub(crate) const SPLASH_RED: [f32; 3] = [0.50, 0.008, 0.012];
/// Kill-only impact wash over the whole scene (frame 0); plain SOLIDs skip it.
pub(crate) const KILL_WASH: [f32; 4] = [0.16, 0.0, 0.004, 0.09];

/// C2 manga panel: bone hairline inset inside the steel edge, leaving a
/// thick black frame between them, with the top-right corner cut at 45°.
pub(crate) const PANEL_LINE: [f32; 4] = [0.40, 0.38, 0.35, 0.85];
pub(crate) const PANEL_INSET_PX: f32 = 3.0;
pub(crate) const PANEL_CUT_PX: f32 = 8.0;
/// Screentone dots: ash on the black panels, ink over portraits.
pub(crate) const TONE_ASH: [f32; 4] = [0.032, 0.032, 0.036, 0.75];
pub(crate) const TONE_INK: [f32; 4] = [0.0, 0.0, 0.0, 0.8];
/// `flat.wgsl` ink-portrait mode (`tone.x < 0`): HUD yoma get the board's
/// dark flesh (`YOMA_FLESH_SAT` chroma), not salmon; a touch darker than
/// `YOMA_DARK` since unlit sprites read lighter than the torch-lit mesh.
pub(crate) const YOMA_PORTRAIT_DARK: f32 = 0.4;
pub(crate) const YOMA_PORTRAIT_TONE: [f32; 4] = [-1.0, super::board::YOMA_FLESH_SAT, YOMA_PORTRAIT_DARK, 0.0];
/// Result screen backdrop: title art through the same ink mode, C1 ash-grey
/// (a trace of chroma) and dark so the ink plate carries the read.
pub(crate) const RESULT_ART_TONE: [f32; 4] = [-1.0, 0.25, 0.16, 0.0];
/// Result plate `[x, y, w, h]` (title + body; RETURN sits below it).
pub(crate) const RESULT_PLATE: [f32; 4] = [0.08, 0.28, 0.62, 0.32];
/// Floater ink stroke width at 800 px tall; scales with the screen.
pub(crate) const STROKE_PX: f32 = 1.0;
/// Tone band (rect heights) and peak dot coverage for HUD shading.
pub(crate) const HUD_TONE_BAND: f32 = 0.3;
pub(crate) const HUD_TONE_COVER: f32 = 0.5;

/// Screentone over the bottom `frac` of `r`: `(pos, uv, tone)` for
/// `tone_rect`. Dots are densest on the bottom edge and fade to nothing at
/// the strip top; the uv window is narrow in x so the sides never thicken.
pub(crate) fn tone_strip(r: [f32; 4], frac: f32, w: f32, h: f32) -> ([f32; 4], [f32; 4], [f32; 4]) {
    let sh = r[3] * frac.clamp(0.05, 1.0);
    let pos = [r[0], r[1] + r[3] - sh, r[2], sh];
    let uv_h = HUD_TONE_BAND / 3.0; // flat.wgsl bottom band is a third as deep
    let uv = [0.49, 1.0 - uv_h, 0.02, uv_h];
    let aspect = (pos[2] * w / uv[2]) / (pos[3] * h / uv[3]).max(1e-3);
    let pitch = crate::game::beats::tone_pitch(h);
    (pos, uv, [pitch, HUD_TONE_COVER, HUD_TONE_BAND, aspect])
}

/// Normalized `[x, y, w, h]` ink plate behind a caption line (same metrics
/// as `Floater::plate_rect`).
pub(crate) fn caption_plate(text: &str, x: f32, y: f32, glyph_h: f32) -> [f32; 4] {
    let text_w = crate::font::text_w(text, glyph_h);
    let (pad_x, pad_y) = (0.010, 0.006);
    [x - pad_x, y - pad_y, text_w + pad_x * 2.0, glyph_h + pad_y * 2.0]
}

/// Ink button label size: 0.024 tall, shrunk so long labels (scene choices
/// like WALK WITH US) keep a margin inside the plate instead of spilling.
pub(crate) fn btn_glyph_h(label: &str, w: f32) -> f32 {
    crate::font::fit(label, 0.024, w * 0.88)
}

/// Glyph height for text set at `x` inside `plate`, shrunk to keep a 0.02
/// right margin (narrow screens widen the type; see `font::glyph_scale`).
pub(crate) fn plate_text_h(text: &str, glyph_h: f32, plate: [f32; 4], x: f32) -> f32 {
    crate::font::fit(text, glyph_h, plate[0] + plate[2] - 0.02 - x)
}

/// Story-scene dialogue plate `[x, y, w, h]` (speaker, line, step count).
pub(crate) const SCENE_PLATE: [f32; 4] = [0.08, 0.22, 0.70, 0.42];
/// Ash for dialogue / result body type, a step under the bone titles.
pub(crate) const ASH_BODY: [f32; 4] = [0.70, 0.68, 0.64, 1.0];

impl Renderer {
    /// C2 manga ink plate: translucent ink, ash dots on its bottom, panel frame.
    pub(super) fn ink_plate(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: [f32; 4],
        w: f32,
        h: f32,
    ) {
        self.rect(rc, r, [INK[0], INK[1], INK[2], 0.92]);
        self.tone_shade(rc, r, 0.35, TONE_ASH, w, h);
        self.ink_panel(rc, r, w, h);
    }

    /// C2 manga panel frame around an ink plate `r`: steel edge, black gap,
    /// bone inner hairline with a 45° cut corner (top right).
    pub(super) fn ink_panel(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: [f32; 4],
        w: f32,
        h: f32,
    ) {
        self.steel_frame(rc, r, w, h);
        let (px, py) = (1.0 / w.max(1.0), 1.0 / h.max(1.0));
        let (ix, iy) = (PANEL_INSET_PX * px, PANEL_INSET_PX * py);
        let i = [r[0] + ix, r[1] + iy, r[2] - 2.0 * ix, r[3] - 2.0 * iy];
        let (cx, cy) = (PANEL_CUT_PX * px, PANEL_CUT_PX * py);
        self.rect(rc, [i[0], i[1], i[2] - cx, py], PANEL_LINE);
        self.rect(rc, [i[0], i[1] + i[3] - py, i[2], py], PANEL_LINE);
        self.rect(rc, [i[0], i[1], px, i[3]], PANEL_LINE);
        self.rect(rc, [i[0] + i[2] - px, i[1] + cy, px, i[3] - cy], PANEL_LINE);
        for k in 0..PANEL_CUT_PX as usize {
            let k = k as f32;
            self.rect(rc, [i[0] + i[2] - cx + k * px, i[1] + k * py, px, py], PANEL_LINE);
        }
    }

    /// Screentone shading over the bottom `frac` of `r` (screen-locked dots).
    pub(super) fn tone_shade(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: [f32; 4],
        frac: f32,
        tint: [f32; 4],
        w: f32,
        h: f32,
    ) {
        let (pos, uv, tone) = tone_strip(r, frac, w, h);
        self.tone_rect(rc, pos, uv, tint, tone);
    }

    /// Manga caption backing: a black brush bar tapering past the plate, a
    /// 2 px ink stroke, the ink plate and bone hairlines top and bottom.
    pub(super) fn manga_caption(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        p: [f32; 4],
        alpha: f32,
        w: f32,
        h: f32,
    ) {
        let (px, py) = (1.0 / w.max(1.0), 1.0 / h.max(1.0));
        let cy = p[1] + p[3] * 0.5;
        for (k, ext) in [(0.7, 8.0), (0.4, 16.0), (0.18, 26.0)] {
            let bh = p[3] * k;
            self.rect(rc, [p[0] - ext * px, cy - bh * 0.5, p[2] + 2.0 * ext * px, bh], [0.0, 0.0, 0.0, 0.85 * alpha]);
        }
        self.rect(rc, [p[0] - 2.0 * px, p[1] - 2.0 * py, p[2] + 4.0 * px, p[3] + 4.0 * py], [0.0, 0.0, 0.0, 0.95 * alpha]);
        self.rect(rc, p, [INK[0], INK[1], INK[2], 0.96 * alpha]);
        let line = [PANEL_LINE[0], PANEL_LINE[1], PANEL_LINE[2], PANEL_LINE[3] * alpha];
        self.rect(rc, [p[0] + 2.0 * px, p[1] + 2.0 * py, p[2] - 4.0 * px, py], line);
        self.rect(rc, [p[0] + 2.0 * px, p[1] + p[3] - 3.0 * py, p[2] - 4.0 * px, py], line);
    }

    /// Bevelled steel frame one pixel wide around `r` (normalized coords).
    pub(super) fn steel_frame(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: [f32; 4],
        w: f32,
        h: f32,
    ) {
        let (px, py) = (1.0 / w.max(1.0), 1.0 / h.max(1.0));
        self.rect(rc, [r[0], r[1], r[2], py], STEEL_EDGE);
        self.rect(rc, [r[0], r[1] + r[3] - py, r[2], py], STEEL_SHADOW);
        let mut side = STEEL_EDGE;
        side[3] *= 0.6;
        self.rect(rc, [r[0], r[1], px, r[3]], side);
        self.rect(rc, [r[0] + r[2] - px, r[1], px, r[3]], STEEL_SHADOW);
    }

    /// Combat action button: ink plate, steel bevel, blood tick when hot or
    /// armed. Disabled keeps the #24 veil + dim label.
    pub(super) fn ink_btn(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: hud::Rect,
        label: &str,
        hot: bool,
        armed: bool,
        danger: bool,
        enabled: bool,
        w: f32,
        h: f32,
    ) {
        let hot = hot && enabled;
        let mut fill = if danger { BLOOD_PLATE } else { INK };
        if hot {
            fill = [fill[0] + 0.014, fill[1] + 0.013, fill[2] + 0.013, fill[3]];
        }
        let pos = r.pos();
        self.rect(rc, pos, fill);
        // Scuffed lighter band along the top third: worn metal catching light.
        self.rect(
            rc,
            [pos[0], pos[1], pos[2], pos[3] * 0.32],
            [0.30, 0.30, 0.33, 0.02],
        );
        self.steel_frame(rc, pos, w, h);
        if hot || armed {
            let (px, py) = (1.0 / w.max(1.0), 1.0 / h.max(1.0));
            self.rect(
                rc,
                [
                    pos[0] + 2.0 * px,
                    pos[1] + pos[3] * 0.2,
                    3.0 * px,
                    pos[3] * 0.6,
                ],
                BLOOD_INK,
            );
            if hot {
                self.rect(
                    rc,
                    [
                        pos[0] + pos[2] * 0.25,
                        pos[1] + pos[3] - 4.0 * py,
                        pos[2] * 0.5,
                        2.0 * py,
                    ],
                    BLOOD_INK,
                );
            }
        }
        let glyph_h = btn_glyph_h(label, r.w);
        let text_w = crate::font::text_w(label, glyph_h);
        let tx = r.x + (r.w - text_w) * 0.5;
        let ty = r.y + (r.h - glyph_h) * 0.5;
        if !enabled {
            self.rect(rc, pos, super::combat_ui::DISABLED_VEIL);
        }
        let ink = if !enabled {
            super::combat_ui::DISABLED_TYPE
        } else if hot {
            STEEL
        } else {
            ASH_TYPE
        };
        self.text_tint(rc, label, tx, ty, glyph_h, ink);
    }

    /// Thin ink trough + steel hairline + fill (HP / trans in combat).
    pub(super) fn ink_bar(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        r: [f32; 4],
        fill: f32,
        tint: [f32; 4],
        w: f32,
        h: f32,
    ) {
        self.rect(rc, r, [0.01, 0.008, 0.008, 0.92]);
        let f = fill.clamp(0.0, 1.0);
        let (px, py) = (1.0 / w.max(1.0), 1.0 / h.max(1.0));
        if f > 0.01 {
            self.rect(
                rc,
                [r[0] + px, r[1] + py, (r[2] - 2.0 * px) * f, r[3] - 2.0 * py],
                tint,
            );
        }
        self.steel_frame(rc, r, w, h);
    }

    /// C4 crit splash (Darkest Dungeon style): a blood-ink blot, spray and
    /// cut streaks behind a SOLID hit, as `SPLASH_FRAMES` held frames. The
    /// scene vignette / torch now live in hunt.wgsl (smooth, dithered).
    pub(super) fn crit_splash(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        game: &Game,
        w: f32,
        h: f32,
    ) {
        let Some(s) = game.fx.splash else {
            return;
        };
        let f = s.frame() as f32;
        let fade = 1.0 - 0.3 * f;
        let aspect = h / w.max(1.0);
        if f == 0.0 && s.kill {
            // Kill impact frame: a soft blood wash over the whole scene.
            self.rect(rc, [0.0, 0.0, 1.0, 1.0], KILL_WASH);
        }
        let blot = self.tex("kenney/fx/scorch.png");
        let red = |a: f32| [SPLASH_RED[0], SPLASH_RED[1], SPLASH_RED[2], a * fade];
        let splat = |rc: &mut _, x: f32, y: f32, size: f32, flip: bool, tint: [f32; 4]| {
            let uv = if flip { [1.0, 0.0, -1.0, 1.0] } else { [0.0, 0.0, 1.0, 1.0] };
            let pos = [x - size * aspect * 0.5, y - size * 0.5, size * aspect, size];
            self.blit_uv(rc, blot, self.sampler, pos, uv, tint);
        };
        // Ink-black backing, then a doubled blood blot that grows per frame.
        let size = 0.62 + 0.1 * f;
        splat(rc, s.x, s.y, size * 1.2, true, [0.0, 0.0, 0.0, 0.55 * fade]);
        for (k, flip) in [(1.0, false), (0.8, true)] {
            splat(rc, s.x, s.y, size * k, flip, red(1.0));
        }
        // Spray thrown further out each frame (golden-angle spread, fixed per hit).
        for i in 0..14 {
            let a = i as f32 * 2.39996 + s.x * 17.0;
            let r = (0.10 + 0.04 * (i % 4) as f32) * (1.0 + 0.5 * f);
            let d = 0.05 + 0.03 * ((i * 7) % 3) as f32;
            splat(rc, s.x + a.cos() * r * aspect, s.y + a.sin() * r * 0.75, d, i % 2 == 0, red(0.95));
        }
        // Tapered cut streaks: a thick short core over a long hairline.
        for i in 0..4 {
            let len = 0.34 + 0.08 * i as f32 + 0.06 * f;
            let dy = (i as f32 - 1.5) * 0.03 + 0.01 * (i % 2) as f32;
            let x0 = s.x - len * (0.5 + 0.12 * (i % 2) as f32);
            for (k, px) in [(1.0, 1.0), (0.6, 2.0), (0.3, 4.0)] {
                let t = px / h.max(1.0);
                let x = x0 + len * (1.0 - k) * 0.5;
                self.rect(rc, [x, s.y + dy - t * 0.5, len * k, t], red(0.9));
            }
        }
        // Diagonal sword cuts across the blot: blood slash, ink-black core.
        let px = 1.0 / h.max(1.0);
        for (i, (angle, off, len, thick)) in SPLASH_CUTS.into_iter().enumerate() {
            let c = [s.x + off * aspect, s.y - off * 0.4];
            let len = len * (1.0 + 0.12 * f);
            for r in diagonal_cut(c, len, angle, aspect, thick * px, px) {
                self.rect(rc, r, red(0.95));
            }
            if i == 0 {
                for r in diagonal_cut(c, len * 0.8, angle, aspect, thick * 0.45 * px, px) {
                    self.rect(rc, r, [0.0, 0.0, 0.0, 0.85 * fade]);
                }
            }
        }
    }
}

/// Crit-splash diagonal cuts: (angle rad, offset along x in screen heights,
/// length in screen heights, peak thickness px). Two parallel slashes and a
/// thin counter-cut.
const SPLASH_CUTS: [(f32, f32, f32, f32); 3] = [(0.52, 0.0, 0.50, 7.0), (0.52, 0.05, 0.34, 3.5), (-0.38, -0.02, 0.30, 2.5)];
/// Steps per diagonal cut (axis-aligned rects stair-stepped along the line).
const CUT_STEPS: usize = 48;

/// One diagonal sword cut through `c` (normalized screen) as a staircase of
/// small rects: `len` in screen heights at `angle` (rad, rising to the right),
/// `aspect` = h / w, thickness peaking at `thick` mid-cut and tapering to
/// `min_t` at both tips.
fn diagonal_cut(c: [f32; 2], len: f32, angle: f32, aspect: f32, thick: f32, min_t: f32) -> Vec<[f32; 4]> {
    let (dx, dy) = (angle.cos() * len * aspect, -angle.sin() * len);
    let step_w = dx.abs() / CUT_STEPS as f32;
    // Each step also spans the line's rise across it, so the stair joins up.
    let rise = dy.abs() / CUT_STEPS as f32;
    (0..CUT_STEPS)
        .map(|i| {
            let u = (i as f32 + 0.5) / CUT_STEPS as f32;
            let t = (thick * (1.0 - (2.0 * u - 1.0).abs()).powf(0.6)).max(min_t);
            let (x, y) = (c[0] + dx * (u - 0.5), c[1] + dy * (u - 0.5));
            // Steps overlap slightly so the stair never shows gaps.
            [x - step_w * 0.55, y - (t + rise) * 0.5, step_w * 1.1, t + rise]
        })
        .collect()
}

/// Eight-way offsets (normalized) for a C2 ink stroke around floater text:
/// whole pixels, at least one, growing with height so 1080p stays crisp.
pub(crate) fn stroke_offsets(w: f32, h: f32) -> [[f32; 2]; 8] {
    let px = (STROKE_PX * h / 800.0).round().max(1.0);
    let (dx, dy) = (px / w.max(1.0), px / h.max(1.0));
    [[-dx, -dy], [0.0, -dy], [dx, -dy], [-dx, 0.0], [dx, 0.0], [-dx, dy], [0.0, dy], [dx, dy]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_strings_fit_their_rects_on_a_390x844_phone() {
        crate::font::set_screen(390.0, 844.0);
        let inside = |s: &str, x: f32, g: f32, r: [f32; 4]| x >= r[0] && x + crate::font::text_w(s, g) <= r[0] + r[2] + 1e-6;
        let bar = crate::hud::combat_bar();
        for (r, s) in [(bar.wait, "WAIT"), (bar.raise, "RAISE"), (bar.guard, "GUARD"), (bar.cut, "CUT"), (bar.slot, "FLASH"), (bar.forfeit, "FALL")]
            .into_iter()
            .chain([(crate::hud::scene_yes(), "WALK WITH US"), (crate::hud::scene_no(), "NOT YET"), (crate::hud::result_ok(), "RETURN")])
        {
            let g = btn_glyph_h(s, r.w);
            assert!(inside(s, r.x + (r.w - crate::font::text_w(s, g)) * 0.5, g, r.pos()), "{s}");
        }
        let body = crate::dialog::RESULT_WIN_BODY;
        assert!(inside(body, 0.12, plate_text_h(body, 0.014, RESULT_PLATE, 0.12), RESULT_PLATE));
        let line = "THE ORGANIZATION WILL NOT MOURN EITHER CHOICE.";
        assert!(inside(line, 0.12, plate_text_h(line, 0.014, SCENE_PLATE, 0.12), SCENE_PLATE));
        let (x, _, g, s) = super::super::combat_ui::log_line("YOMA GLANCES RAKI FOR 14. IT STAGGERS BACK", &bar);
        assert!(x + crate::font::text_w(&s, g) <= bar.forfeit.x + bar.forfeit.w + 1e-6);
        let card = super::super::combat_ui::unit_card(bar.wait.y);
        let name = "DENEVE  AP 4";
        assert!(inside(name, card.name[0], crate::font::fit(name, card.name[3], card.name[2]), card.name));
        // Type keeps its designed aspect: phone glyphs are just smaller.
        assert!(crate::font::glyph_draw_h(0.024) * 844.0 > 9.0);
        crate::font::set_screen(1280.0, 800.0);
    }

    #[test]
    fn scene_plate_and_choice_labels_fit() {
        let [x, y, w, h] = SCENE_PLATE;
        assert!(x >= 0.0 && y >= 0.0 && x + w <= 1.0 && y + h <= 1.0);
        for r in [crate::hud::scene_yes(), crate::hud::scene_no()] {
            assert!(r.y > y + h, "choices below the plate");
        }
        // Short labels keep the combat size; long ones shrink inside the plate.
        assert_eq!(btn_glyph_h("CUT", 0.16), 0.024);
        for label in ["WALK WITH US", "NOT YET", "RETURN"] {
            let g = btn_glyph_h(label, 0.16);
            let n = label.len() as f32;
            let text_w = g * 0.7 * ((n - 1.0) * 0.85 + 1.0);
            assert!(text_w <= 0.16 * 0.88 + 1e-6 && g >= 0.018, "{label}: {g}");
        }
    }

    #[test]
    fn result_plate_is_on_screen_and_clear_of_return() {
        let [x, y, w, h] = RESULT_PLATE;
        assert!(x >= 0.0 && y >= 0.0 && x + w <= 1.0 && y + h <= 1.0);
        let ok = crate::hud::result_ok().pos();
        assert!(ok[1] > y + h, "RETURN below the plate");
        // Art stays dark ash: well under the plate's bone type.
        assert!(RESULT_ART_TONE[0] < 0.0 && RESULT_ART_TONE[1] <= 0.3 && RESULT_ART_TONE[2] <= 0.2);
    }

    #[test]
    fn hud_yoma_portraits_match_board_flesh_and_floaters_get_a_whole_pixel_stroke() {
        // Mirror of flat.wgsl's ink-portrait branch on the old salmon tint.
        let t = YOMA_PORTRAIT_TONE;
        assert!(t[0] < 0.0);
        let salmon = [0.92, 0.55, 0.48];
        let l = 0.2126 * salmon[0] + 0.7152 * salmon[1] + 0.0722 * salmon[2];
        let ink = salmon.map(|c| (l + (c - l) * t[1]) * t[2]);
        let board = super::super::board::yoma_flesh(salmon);
        let k = YOMA_PORTRAIT_DARK / super::super::board::YOMA_DARK;
        assert!(k < 1.0 && ink.iter().zip(board).all(|(a, b)| (a - b * k).abs() < 1e-6), "{ink:?}");
        let chroma = ink[0].max(ink[1]).max(ink[2]) - ink[0].min(ink[1]).min(ink[2]);
        assert!(chroma < 0.03 && ink[0] >= ink[2], "grey-brown, not salmon: {ink:?}");
        for (w, h) in [(390.0, 844.0), (1280.0, 800.0), (1920.0, 1080.0)] {
            let o = stroke_offsets(w, h);
            let px = (o[4][0] * w, o[6][1] * h);
            assert!(px.0 >= 1.0 - 1e-4 && px.0 <= 2.0 + 1e-4 && (px.0 - px.0.round()).abs() < 1e-3);
            assert!((px.0 - px.1).abs() < 1e-3);
            assert_eq!(o.iter().filter(|d| d[0] == 0.0 && d[1] == 0.0).count(), 0);
        }
    }

    fn luma(c: [f32; 4]) -> f32 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
    }

    #[test]
    fn hud_tone_shades_only_the_bottom_of_its_rect() {
        let card = super::super::combat_ui::unit_card(0.86);
        for (w, h) in [(390.0, 844.0), (1280.0, 800.0), (1920.0, 1080.0)] {
            for r in [card.plate, card.portrait] {
                let (pos, uv, tone) = tone_strip(r, 0.45, w, h);
                // Inside the rect, flush with its bottom edge.
                assert!(pos[0] == r[0] && pos[2] == r[2] && pos[1] >= r[1]);
                assert!((pos[1] + pos[3] - (r[1] + r[3])).abs() < 1e-6);
                // uv ends on the bottom edge (densest) and starts where the
                // flat.wgsl bottom band (x3) reaches zero coverage.
                assert!((uv[1] + uv[3] - 1.0).abs() < 1e-6);
                assert!(((1.0 - uv[1]) * 3.0 - tone[2]).abs() < 1e-5);
                // Sides never thicken: x edge distance (>= 0.49 of the full
                // rect, in rect heights) stays past the band.
                assert!(0.49 * tone[3] >= tone[2], "{w}x{h} aspect {}", tone[3]);
                // Dots never merge; pitch whole screen pixels >= 4.
                assert!(0.56 * tone[1].sqrt() < 0.5 && tone[0] >= 4.0);
            }
        }
        // Caption plate wraps the glyph run with padding.
        let p = caption_plate("YOMA REACHES", 0.5, 0.72, 0.016);
        assert!(p[0] < 0.5 && p[1] < 0.72 && p[0] + p[2] > 0.5 + 0.016 * 0.7 * 11.0 * 0.85);
        assert!(PANEL_LINE[0] > INK[0] * 20.0, "bone hairline reads on ink");
    }

    #[test]
    fn diagonal_cut_is_a_tapered_slash_through_the_hit() {
        let c = [0.5, 0.5];
        let cut = diagonal_cut(c, 0.5, 0.52, 0.625, 0.01, 0.001);
        assert_eq!(cut.len(), CUT_STEPS);
        let mid = |r: &[f32; 4]| [r[0] + r[2] * 0.5, r[1] + r[3] * 0.5];
        // Rises to the right (screen y down) and stays centred on the hit.
        assert!(mid(&cut[0])[0] < mid(&cut[CUT_STEPS - 1])[0]);
        assert!(mid(&cut[0])[1] > mid(&cut[CUT_STEPS - 1])[1]);
        let (a, b) = (mid(&cut[0]), mid(&cut[CUT_STEPS - 1]));
        assert!(((a[0] + b[0]) * 0.5 - 0.5).abs() < 1e-5 && ((a[1] + b[1]) * 0.5 - 0.5).abs() < 1e-5);
        // Thick mid-cut, tapering symmetrically to the hairline tips.
        let th: Vec<f32> = cut.iter().map(|r| r[3]).collect();
        assert!(th[CUT_STEPS / 2] > th[2] && th[2] > th[0] - 1e-7);
        assert!((th[1] - th[CUT_STEPS - 2]).abs() < 1e-6);
        let rise = 0.5 * 0.52_f32.sin() / CUT_STEPS as f32;
        assert!(th.iter().all(|t| *t >= 0.001 + rise - 1e-6 && *t <= 0.01 + rise + 1e-6));
        // Neighbouring steps touch and overlap vertically (no gaps along the slash).
        assert!(cut.windows(2).all(|p| p[0][0] + p[0][2] >= p[1][0] && p[1][1] + p[1][3] >= p[0][1]));
    }

    #[test]
    fn chrome_is_ink_steel_and_blood_not_grey_plates() {
        // Linear tints: ink stays near-black on the sRGB swapchain.
        assert!(luma(INK) < 0.01 && INK[3] >= 0.99, "opaque ink plate");
        assert!(
            luma(STEEL_EDGE) > luma(INK) * 20.0,
            "steel hairline reads on ink"
        );
        assert!(
            (STEEL_EDGE[0] - STEEL_EDGE[2]).abs() < 0.05,
            "steel is neutral, not gold"
        );
        for blood in [BLOOD_INK, BLOOD_PLATE] {
            assert!(
                blood[0] > blood[1] * 10.0 && blood[0] > blood[2] * 10.0,
                "{blood:?}"
            );
            assert!(luma(blood) < 0.06, "dried blood, not candy red {blood:?}");
        }
        // The kill wash is a soft tint (was [0.25, 0, 0.005, 0.2] on every SOLID).
        assert!(KILL_WASH[3] <= 0.1 && KILL_WASH[0] < 0.25 && KILL_WASH[0] > KILL_WASH[1] * 10.0);
    }
}
