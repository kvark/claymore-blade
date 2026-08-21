use super::*;

impl Renderer {
    pub(super) fn draw_title(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/title.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0, 1.0],
        );
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.48]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/rune/brand.png"),
            [0.78, 0.08, 0.16, 0.22],
            [0.55, 0.18, 0.16, 0.85],
        );
        self.text(&mut rc, "CLAYMORE", 0.08, 0.12, 0.028);
        self.text(&mut rc, "NO. 47", 0.08, 0.20, 0.016);
        self.text(&mut rc, game.title_flavor(), 0.08, 0.28, 0.012);
        self.kenney_btn(&mut rc, "kenney/ui/button.png", hud::title_new(), "NEW HUNT", false);
        if game.has_save {
            self.kenney_btn(
                &mut rc,
                "kenney/ui/button-grey.png",
                hud::title_continue(),
                "CONTINUE",
                false,
            );
        }
        self.draw_fx(&mut rc, game);
    }

    pub(super) fn draw_intro(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.blit(
            &mut rc,
            self.tex("art/title.jpg"),
            [0.0, 0.0, 1.0, 1.0],
            [0.4, 0.4, 0.4, 1.0],
        );
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.03, 0.02, 0.02, 0.55]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.07, 0.16, 0.72, 0.58],
            [0.55, 0.46, 0.36, 0.55],
        );
        let mut y = 0.22;
        for line in game.intro_text().lines() {
            self.text(&mut rc, line, 0.10, y, 0.014);
            y += 0.045;
        }
        self.prompt(&mut rc, "kenney/prompt/space.png", 0.10, 0.84, 0.05);
        self.text(&mut rc, "WALK THE ISLAND", 0.17, 0.855, 0.014);
        self.draw_fx(&mut rc, game);
    }

}
