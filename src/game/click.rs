//! Pointer handlers: title, world, town, result, encounter start.

use super::*;
use crate::audio;
use crate::catalog::{self};
use crate::combat::{
    act, core_hex, create_battle, current_unit, legal_moves, legal_targets, run_ai, zone_for,
    CombatState, PlayerAction, Side,
};
use crate::dialog::{self, SceneId, SceneState};
use crate::hud;
use crate::world::{self, apply_victory};
use crate::hex::{hex_eq, Axial};

impl Game {
    pub fn click(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        self.ui.screen = screen;
        match self.mode {
            Mode::Title => self.click_title(nx, ny),
            Mode::Intro => {
                audio::click();
                self.mode = Mode::World;
                audio::music_island();
            }
            Mode::World => self.click_world(nx, ny),
            Mode::Town => self.click_town(nx, ny),
            Mode::Combat => self.click_combat(nx, ny, screen),
            Mode::Result => self.click_result(),
            Mode::Codex => {
                audio::play("close");
                self.mode = Mode::World;
            }
            Mode::Scene => self.click_scene(nx, ny),
        }
    }

    pub(super) fn click_result(&mut self) {
        audio::click();
        self.combat = None;
        self.result_win = None;
        if self.scene.is_some() {
            self.mode = Mode::Scene;
        } else {
            self.mode = Mode::World;
            audio::music_island();
        }
        self.persist();
    }

    pub(super) fn click_title(&mut self, nx: f32, ny: f32) {
        if hud::title_new().contains(nx, ny) {
            self.new_hunt();
        } else if self.has_save && hud::title_continue().contains(nx, ny) {
            self.continue_hunt();
        }
    }

    pub(super) fn click_world(&mut self, nx: f32, ny: f32) {
        if hud::world_codex().contains(nx, ny) {
            audio::click();
            self.mode = Mode::Codex;
            return;
        }
        if let Some(loc) = world::nearest_location(nx, ny, 0.04) {
            let st = self.world.locations.get(loc.id).map(|s| s.status);
            if st == Some(world::WorldStatus::Locked) {
                audio::error();
                return;
            }
            if st == Some(world::WorldStatus::Dead) {
                audio::error();
                self.result_title = dialog::RESULT_LATE_TITLE.into();
                self.result_body = dialog::RESULT_LATE.into();
                audio::music_defeat();
                self.result_win = Some(false);
                self.scene = None;
                self.mode = Mode::Result;
                self.persist();
                return;
            }
            audio::confirm();
            self.world.party_x = loc.x;
            self.world.party_y = loc.y;
            self.world.last_town = Some(loc.id.into());
            if self.world.flags.get("raki-refused") == Some(&true) && !self.world.raki {
                self.world.raki = true;
                self.world.flags.remove("raki-refused");
                self.world.flags.insert("raki-followed".into(), true);
            }
            if loc.id == "doga"
                && self.world.flags.get("doga-talked") != Some(&true)
                && self.world.locations.get("doga").map(|s| s.status)
                    != Some(world::WorldStatus::Cleared)
            {
                self.scene = Some(SceneState::new(SceneId::TownDoga));
                self.mode = Mode::Scene;
            } else {
                self.mode = Mode::Town;
                audio::music_town();
            }
            self.persist();
        }
    }

    pub(super) fn click_town(&mut self, nx: f32, ny: f32) {
        let id = self
            .world
            .last_town
            .clone()
            .unwrap_or_else(|| "doga".into());
        let loc = catalog::location(&id);
        if hud::town_hunt().contains(nx, ny) {
            if let Some(enc_id) = loc.and_then(|l| l.encounter) {
                let st = self.world.locations.get(&id).map(|s| s.status);
                if st == Some(world::WorldStatus::Cleared)
                    || st == Some(world::WorldStatus::Locked)
                    || st == Some(world::WorldStatus::Dead)
                {
                    audio::error();
                    return;
                }
                audio::play("draw");
                self.start_encounter(enc_id);
            } else {
                audio::error();
            }
        } else if hud::town_rest().contains(nx, ny) {
            audio::play("cloth");
            self.world.hours += 8.0;
            self.persist();
        } else if hud::town_leave().contains(nx, ny) {
            audio::play("close");
            self.mode = Mode::World;
            audio::music_island();
        }
    }

    pub fn start_encounter(&mut self, id: &str) {
        let Some(enc) = catalog::encounter(id) else {
            return;
        };
        if id == "gonal-ripple" && self.world.flags.get("ophelia-spoken") != Some(&true) {
            self.pending_encounter = Some(id.into());
            self.scene = Some(SceneState::new(SceneId::OpheliaIntro));
            self.mode = Mode::Scene;
            self.persist();
            return;
        }
        self.begin_battle(enc);
    }

    pub(super) fn begin_battle(&mut self, enc: &catalog::EncounterDef) {
        let seed = (self.world.hours as u32).wrapping_mul(997) + 13;
        let mut combat = create_battle(enc, &self.world.party, seed.max(1));
        if current_unit(&combat)
            .map(|u| u.side == Side::Enemy)
            .unwrap_or(false)
        {
            run_ai(&mut combat);
        }
        self.combat = Some(combat);
        self.mode = Mode::Combat;
        audio::music_hunt();
        self.ui = Ui {
            screen: self.ui.screen,
            ..Ui::default()
        };
        self.fx = crate::fx::Fx::default();
        self.persist();
    }
}
