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
            let pop = 1.0 + (1.0 - a) * 0.25;
            let mut tint = f.tint;
            tint[3] *= a;
            self.text_tint(rc, &f.text, f.x, f.y, 0.016 * pop, tint);
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
        let tex = if hot {
            "kenney/ui/button-line.png"
        } else {
            kind
        };
        self.blit_px(rc, self.tex(tex), r.pos(), ASH);
        if hot {
            self.rect(rc, r.pos(), [0.72, 0.55, 0.22, 0.22]);
        }
        self.text(rc, label, r.x + 0.016, r.y + r.h * 0.32, 0.013);
    }

    pub(super) fn bar(
        &self,
        rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill: f32,
        tint: [f32; 4],
    ) {
        self.blit_px(rc, self.tex("kenney/ui/bar.png"), [x, y, w, h], [0.22, 0.18, 0.14, 0.95]);
        let f = fill.clamp(0.0, 1.0);
        if f > 0.01 {
            self.blit_px(
                rc,
                self.tex("kenney/ui/bar-fill.png"),
                [x + w * 0.02, y + h * 0.15, w * 0.96 * f, h * 0.7],
                tint,
            );
        }
    }

    pub(super) fn prompt(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, rel: &str, x: f32, y: f32, s: f32) {
        self.blit_px(rc, self.tex(rel), [x, y, s, s], [0.85, 0.8, 0.7, 0.95]);
    }

    pub(super) fn blit(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_uv(rc, view, self.sampler, pos, [0.0, 0.0, 1.0, 1.0], tint);
    }

    pub(super) fn blit_px(&self, rc: &mut impl gpu::traits::RenderPipelineEncoder<BufferPiece = gpu::BufferPiece>, view: gpu::TextureView, pos: [f32; 4], tint: [f32; 4]) {
        self.blit_uv(rc, view, self.pixel, pos, [0.0, 0.0, 1.0, 1.0], tint);
    }

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
