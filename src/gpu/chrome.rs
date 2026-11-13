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

    /// Ink gradient: `bands` stacked strips growing from an edge.
    pub(super) fn ink_fade(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        y: f32,
        height: f32,
        from_top: bool,
        alpha: f32,
    ) {
        const BANDS: usize = 6;
        for i in 0..BANDS {
            let d = height * (i + 1) as f32 / BANDS as f32;
            let y0 = if from_top { y } else { y - d };
            self.rect(rc, [0.0, y0, 1.0, d], [0.0, 0.0, 0.0, alpha / BANDS as f32]);
        }
    }

    /// Nest vignette: the clearing's edges sink into dark (left/right/top).
    pub(super) fn nest_vignette(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        panel_top: f32,
    ) {
        const BANDS: usize = 6;
        for i in 0..BANDS {
            let d = 0.03 * (i + 1) as f32;
            let c = [0.0, 0.0, 0.0, 0.085];
            self.rect(rc, [0.0, 0.0, d, 1.0], c);
            self.rect(rc, [1.0 - d, 0.0, d, 1.0], c);
        }
        self.ink_fade(rc, 0.0, 0.16, true, 0.55);
        self.ink_fade(rc, panel_top, 0.08, false, 0.5);
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
