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

impl Renderer {
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
        let glyph_h = 0.024;
        let glyph_w = glyph_h * 0.7;
        let n = label.chars().count() as f32;
        let text_w = if n > 0.0 {
            glyph_w * ((n - 1.0) * 0.85 + 1.0)
        } else {
            0.0
        };
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
        if f == 0.0 {
            // Impact frame: the whole scene takes a blood wash.
            self.rect(rc, [0.0, 0.0, 1.0, 1.0], [0.25, 0.0, 0.005, 0.2]);
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn luma(c: [f32; 4]) -> f32 {
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
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
    }
}
