use super::*;

impl Renderer {
    pub(super) fn draw_result(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.08, 0.28, 0.62, 0.32],
            [0.5, 0.4, 0.3, 0.85],
        );
        self.text(&mut rc, &game.result_title, 0.12, 0.34, 0.022);
        self.text(&mut rc, &game.result_body, 0.12, 0.44, 0.014);
        self.kenney_btn(
            &mut rc,
            "kenney/ui/button.png",
            hud::result_ok(),
            "RETURN",
            false,
        );
        self.draw_fx(&mut rc, game);
    }

    pub(super) fn draw_codex(&self, pass: &mut gpu::RenderCommandEncoder, _game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.05, 0.04, 0.03, 1.0]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/banner.png"),
            [0.06, 0.05, 0.4, 0.08],
            ASH,
        );
        self.text(&mut rc, "CODEX", 0.08, 0.07, 0.022);
        for (i, w) in WARRIORS.iter().take(8).enumerate() {
            let col = (i % 4) as f32;
            let row = (i / 4) as f32;
            let x = 0.08 + col * 0.22;
            let y = 0.18 + row * 0.32;
            self.blit_px(
                &mut rc,
                self.tex("kenney/ui/panel-brown.png"),
                [x, y, 0.20, 0.28],
                [0.48, 0.38, 0.28, 0.9],
            );
            self.blit(&mut rc, self.tex(w.portrait), [x + 0.03, y + 0.02, 0.14, 0.18], [1.0; 4]);
            self.text(&mut rc, w.name, x + 0.02, y + 0.21, 0.012);
            self.text(&mut rc, w.title, x + 0.02, y + 0.235, 0.010);
        }
        self.text(&mut rc, "CLICK TO CLOSE", 0.08, 0.90, 0.014);
    }


    pub(super) fn draw_scene(&self, pass: &mut gpu::RenderCommandEncoder, game: &Game, _w: f32, _h: f32) {
        let mut rc = pass.with(&self.flat);
        let art = match game.scene.as_ref().map(|s| s.id) {
            Some(crate::dialog::SceneId::OpheliaIntro) => "art/battle-gonal.jpg",
            Some(crate::dialog::SceneId::TownDoga) => "art/tavern.jpg",
            Some(crate::dialog::SceneId::RecruitPaburo) => "art/battle-paburo.jpg",
            Some(crate::dialog::SceneId::RecruitPieta) => "art/battle-pieta.jpg",
            _ => "art/title.jpg",
        };
        self.blit(&mut rc, self.tex(art), [0.0, 0.0, 1.0, 1.0], [0.45, 0.42, 0.38, 1.0]);
        self.rect(&mut rc, [0.0, 0.0, 1.0, 1.0], [0.04, 0.03, 0.02, 0.55]);
        self.blit_px(
            &mut rc,
            self.tex("kenney/ui/panel-brown.png"),
            [0.08, 0.22, 0.70, 0.42],
            [0.50, 0.40, 0.30, 0.92],
        );
        if let Some(scene) = game.scene.as_ref() {
            if let Some(line) = scene.current() {
                self.text(&mut rc, line.speaker, 0.12, 0.28, 0.016);
                // wrap-ish: single line may be long; draw once
                self.text(&mut rc, line.text, 0.12, 0.38, 0.014);
                let step = format!("{}/{}", scene.step + 1, scene.lines().len());
                self.text(&mut rc, &step, 0.12, 0.52, 0.011);
            }
            if scene.at_end() {
                let choices = scene.choices();
                if choices.len() >= 1 {
                    self.kenney_btn(
                        &mut rc,
                        "kenney/ui/button.png",
                        crate::hud::scene_yes(),
                        choices[0].label,
                        true,
                    );
                }
                if choices.len() >= 2 {
                    self.kenney_btn(
                        &mut rc,
                        "kenney/ui/button-grey.png",
                        crate::hud::scene_no(),
                        choices[1].label,
                        false,
                    );
                }
            } else {
                self.prompt(&mut rc, "kenney/prompt/space.png", 0.12, 0.84, 0.05);
                self.text(&mut rc, "CONTINUE", 0.19, 0.855, 0.014);
            }
        }
        self.draw_fx(&mut rc, game);
    }

}
