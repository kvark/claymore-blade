use super::*;

impl Renderer {
    pub(super) fn draw_fx(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, game: &Game) {
        for p in &game.fx.particles {
            let a = (p.life / p.max).clamp(0.0, 1.0);
            let mut tint = p.tint;
            tint[3] *= a;
            self.blit_px(
                rc,
                self.tex(p.sprite),
                [p.x - p.size * 0.5, p.y - p.size * 0.5, p.size, p.size],
                tint,
            );
        }
        for b in &game.fx.bursts {
            let t = 1.0 - (b.life / b.max);
            let s = b.size * (0.55 + t * 0.8);
            let a = (b.life / b.max).clamp(0.0, 1.0);
            let mut tint = b.tint;
            tint[3] *= a;
            self.blit(
                rc,
                self.tex(b.sprite),
                [b.x - s * 0.5, b.y - s * 0.5, s, s],
                tint,
            );
        }
        for f in &game.fx.floaters {
            let a = (f.life / f.max).clamp(0.0, 1.0);
            let glyph_h = f.glyph_h();
            let mut tint = f.tint;
            tint[3] *= a;
            if f.plate {
                self.rect(rc, f.plate_rect(), [0.05, 0.04, 0.03, 0.78 * a]);
            }
            self.text_tint(rc, &f.text, f.x, f.y, glyph_h, tint);
        }
        if game.fx.flash > 0.02 {
            self.rect(rc, [0.0, 0.0, 1.0, 1.0], [0.92, 0.82, 0.72, game.fx.flash * 0.4]);
        }
    }

    pub(super) fn kenney_btn(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        kind: &str,
        r: hud::Rect,
        label: &str,
        hot: bool,
    ) {
        self.kenney_btn_ex(rc, kind, r, label, hot, 0.022);
    }

    pub(super) fn kenney_btn_ex(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        kind: &str,
        r: hud::Rect,
        label: &str,
        hot: bool,
        glyph_h: f32,
    ) {
        self.kenney_btn_state(rc, kind, r, label, hot, glyph_h, true);
    }

    /// Button with an enabled state: a disabled plate is veiled in ink and
    /// its label drops to dim ash, so blocked input reads at a glance.
    pub(super) fn kenney_btn_state(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        kind: &str,
        r: hud::Rect,
        label: &str,
        hot: bool,
        glyph_h: f32,
        enabled: bool,
    ) {
        let hot = hot && enabled;
        let tex = if hot {
            "kenney/ui/button-line.png"
        } else {
            kind
        };
        // Dark ink plates; danger chips (button-red) keep a blood tint.
        let plate = if kind.contains("button-red") {
            BLOOD
        } else {
            INK_PLATE
        };
        self.blit_px(rc, self.tex(tex), r.pos(), plate);
        if hot {
            let mut steel = STEEL;
            steel[3] = 0.22;
            self.rect(rc, r.pos(), steel);
        }
        let glyph_w = glyph_h * 0.7;
        let n = label.chars().filter(|c| *c != '\n').count() as f32;
        let text_w = if n > 0.0 {
            glyph_w * ((n - 1.0) * 0.85 + 1.0)
        } else {
            0.0
        };
        let tx = r.x + (r.w - text_w) * 0.5;
        let ty = r.y + (r.h - glyph_h) * 0.5;
        if !enabled {
            self.rect(rc, r.pos(), super::combat_ui::DISABLED_VEIL);
        }
        let ink = if enabled { ASH_TYPE } else { super::combat_ui::DISABLED_TYPE };
        self.text_tint(rc, label, tx, ty, glyph_h, ink);
    }

    pub(super) fn prompt(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, rel: &str, x: f32, y: f32, s: f32) {
        self.prompt_state(rc, rel, x, y, s, true);
    }

    /// Key-hint chip; dimmed with its button while turn input is blocked.
    pub(super) fn prompt_state(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, rel: &str, x: f32, y: f32, s: f32, enabled: bool) {
        let tint = if enabled {
            [0.85, 0.8, 0.7, 0.95]
        } else {
            super::combat_ui::DISABLED_PROMPT
        };
        self.blit_px(rc, self.tex(rel), [x, y, s, s], tint);
    }

    pub(super) fn blit(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_uv(rc, view, self.sampler, pos, [0.0, 0.0, 1.0, 1.0], tint);
    }

    pub(super) fn blit_px(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_uv(rc, view, self.pixel, pos, [0.0, 0.0, 1.0, 1.0], tint);
    }

    #[allow(dead_code)]
    pub(super) fn blit_s(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        view: gpu::TextureView,
        sampler: gpu::Sampler,
        pos: [f32; 4],
        tint: [f32; 4],
    ) {
        self.blit_uv(rc, view, sampler, pos, [0.0, 0.0, 1.0, 1.0], tint);
    }

    /// Blit with a custom UV rect. Use `[1, 0, -1, 1]` to flip horizontally.
    pub(super) fn blit_uv(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        view: gpu::TextureView,
        sampler: gpu::Sampler,
        pos: [f32; 4],
        uv: [f32; 4],
        tint: [f32; 4],
    ) {
        rc.bind(
            0,
            &FlatFrame {
                globals: FlatGlobals { pad: [0.0; 4] },
                sprite_texture: view,
                sprite_sampler: sampler,
            },
        );
        rc.bind(
            1,
            &FlatDraw {
                locals: FlatLocal {
                    pos_size: pos,
                    uv_rect: uv,
                    tint,
                },
            },
        );
        rc.bind_vertex(0, self.quad.into());
        rc.draw(0, 4, 0, 1);
    }

    pub(super) fn rect(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, pos: [f32; 4], tint: [f32; 4]) {
        self.blit(rc, self.white.view, pos, tint);
    }

    pub(super) fn text(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, s: &str, x: f32, y: f32, glyph_h: f32) {
        self.text_tint(rc, s, x, y, glyph_h, [0.92, 0.88, 0.78, 1.0]);
    }

    pub(super) fn text_tint(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, s: &str, x: f32, y: f32, glyph_h: f32, tint: [f32; 4]) {
        let glyph_w = glyph_h * 0.7;
        rc.bind(
            0,
            &FlatFrame {
                globals: FlatGlobals { pad: [0.0; 4] },
                sprite_texture: self.font.view,
                sprite_sampler: self.sampler,
            },
        );
        let mut cx = x;
        for ch in s.chars() {
            if ch == '\n' {
                continue;
            }
            let (u, v, du, dv) = font::glyph_uv(if ch == '·' {
                '-'
            } else {
                ch.to_ascii_uppercase()
            });
            rc.bind(
                1,
                &FlatDraw {
                    locals: FlatLocal {
                        pos_size: [cx, y, glyph_w, glyph_h],
                        uv_rect: [u, v, du, dv],
                        tint,
                    },
                },
            );
            rc.bind_vertex(0, self.quad.into());
            rc.draw(0, 4, 0, 1);
            cx += glyph_w * 0.85;
        }
    }
}
