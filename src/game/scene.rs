//! Dialog scene advance and choice resolution.

use super::*;
use crate::audio;
use crate::catalog::{self};
use crate::dialog::{self, SceneId, SceneState};
use crate::hud;

impl Game {
    pub(super) fn click_scene(&mut self, nx: f32, ny: f32) {
        let Some(scene) = self.scene.as_ref() else {
            self.mode = Mode::World;
            return;
        };
        if scene.at_end() {
            let choices = scene.choices();
            if choices.len() >= 2 {
                if hud::scene_yes().contains(nx, ny) {
                    self.resolve_scene(true);
                    return;
                }
                if hud::scene_no().contains(nx, ny) {
                    self.resolve_scene(false);
                    return;
                }
                return;
            }
            if choices.len() == 1 && hud::scene_yes().contains(nx, ny) {
                self.resolve_scene(true);
                return;
            }
            if ny > 0.72 {
                self.resolve_scene(true);
            }
            return;
        }
        audio::click();
        if let Some(s) = self.scene.as_mut() {
            s.advance();
        }
    }

    pub(super) fn resolve_scene(&mut self, yes: bool) {
        let Some(scene) = self.scene.take() else {
            self.mode = Mode::World;
            return;
        };
        match scene.id {
            SceneId::RakiJoin => {
                if yes {
                    self.world.raki = true;
                    audio::confirm();
                } else {
                    self.world.flags.insert("raki-refused".into(), true);
                    audio::play("close");
                }
                self.mode = Mode::World;
            }
            SceneId::RecruitPaburo => {
                if yes {
                    for id in ["miria", "helen"] {
                        if !self.world.party.iter().any(|p| p == id) {
                            self.world.party.push(id.into());
                        }
                    }
                    audio::confirm();
                } else {
                    audio::play("close");
                }
                self.mode = Mode::World;
            }
            SceneId::RecruitPieta => {
                if yes {
                    if !self.world.party.iter().any(|p| p == "deneve") {
                        self.world.party.push("deneve".into());
                    }
                    audio::confirm();
                } else {
                    audio::play("close");
                }
                self.mode = Mode::World;
            }
            SceneId::OpheliaIntro => {
                self.world.flags.insert("ophelia-spoken".into(), true);
                audio::play("draw");
                if let Some(id) = self.pending_encounter.take() {
                    if let Some(enc) = catalog::encounter(&id) {
                        self.begin_battle(enc);
                        self.persist();
                        return;
                    }
                }
                self.mode = Mode::World;
            }
            SceneId::TownDoga => {
                self.world.flags.insert("doga-talked".into(), true);
                audio::click();
                self.mode = Mode::Town;
            }
            SceneId::TownDogaLate => {
                self.world.flags.insert("doga-late-talked".into(), true);
                audio::click();
                self.mode = Mode::Town;
            }
        }
        self.persist();
    }
}
