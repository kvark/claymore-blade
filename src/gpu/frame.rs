use super::*;
use super::util::make_depth;

impl Renderer {
    pub fn resize(&mut self, context: &gpu::Context, screen: gpu::Extent) {
        if screen == self.screen {
            return;
        }
        context.destroy_texture_view(self.depth_view);
        context.destroy_texture(self.depth);
        let (d, v) = make_depth(context, screen);
        self.depth = d;
        self.depth_view = v;
        self.screen = screen;
        let _ = self.format;
    }

    pub fn depth_texture(&self) -> gpu::Texture {
        self.depth
    }

    pub fn is_zero_screen(&self) -> bool {
        self.screen.width == 0 || self.screen.height == 0
    }

    pub fn render(
        &mut self,
        encoder: &mut gpu::CommandEncoder,
        target: gpu::TextureView,
        game: &Game,
    ) {
        let w = self.screen.width.max(1) as f32;
        let h = self.screen.height.max(1) as f32;
        if let mut pass = encoder.render(
            "hunt",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: target,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack),
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: Some(gpu::RenderTarget {
                    view: self.depth_view,
                    init_op: gpu::InitOp::Clear(gpu::TextureColor::White),
                    finish_op: gpu::FinishOp::Discard,
                }),
            },
        ) {
            if game.mode == Mode::Combat {
                if let Some(combat) = game.combat.as_ref() {
                    self.draw_board(&mut pass, combat, game, w, h);
                }
            }
        }
        if let mut pass = encoder.render(
            "flat",
            gpu::RenderTargetSet {
                colors: &[gpu::RenderTarget {
                    view: target,
                    init_op: if game.mode == Mode::Combat {
                        gpu::InitOp::Load
                    } else {
                        gpu::InitOp::Clear(gpu::TextureColor::OpaqueBlack)
                    },
                    finish_op: gpu::FinishOp::Store,
                }],
                depth_stencil: None,
            },
        ) {
            match game.mode {
                Mode::Title => self.draw_title(&mut pass, game, w, h),
                Mode::Intro => self.draw_intro(&mut pass, game, w, h),
                Mode::World => self.draw_world(&mut pass, game, w, h),
                Mode::Town => self.draw_town(&mut pass, game, w, h),
                Mode::Combat => self.draw_combat_overlay(&mut pass, game, w, h),
                Mode::Result => self.draw_result(&mut pass, game, w, h),
                Mode::Codex => self.draw_codex(&mut pass, game, w, h),
                Mode::Scene => self.draw_scene(&mut pass, game, w, h),
            }
        }
    }

    pub fn destroy(&mut self, context: &gpu::Context) {
        context.destroy_texture_view(self.depth_view);
        context.destroy_texture(self.depth);
        context.destroy_texture_view(self.white.view);
        context.destroy_texture(self.white.texture);
        context.destroy_texture_view(self.font.view);
        context.destroy_texture(self.font.texture);
        for t in self.images.values() {
            context.destroy_texture_view(t.view);
            context.destroy_texture(t.texture);
        }
        context.destroy_sampler(self.sampler);
        context.destroy_sampler(self.pixel);
        context.destroy_buffer(self.prism);
        context.destroy_buffer(self.quad);
        context.destroy_render_pipeline(&mut self.hunt);
        context.destroy_render_pipeline(&mut self.flat);
    }

    pub(super) fn pan(game: &Game, w: f32, h: f32) -> [f32; 2] {
        let s = game.fx.shake(w, h);
        [game.ui.pan[0] + s[0], game.ui.pan[1] + s[1]]
    }

    pub(super) fn tex(&self, rel: &str) -> gpu::TextureView {
        self.images
            .get(rel)
            .map(|t| t.view)
            .unwrap_or(self.white.view)
    }

}
