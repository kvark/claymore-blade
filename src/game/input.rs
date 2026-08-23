//! Keyboard, hex pick, and zone preview.

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
    pub fn hover_hex(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        self.ui.screen = screen;
        if self.mode != Mode::Combat || ny > 0.88 {
            self.ui.hover = None;
            return;
        }
        self.ui.hover = self.pick_hex(nx, ny, screen);
    }

    pub fn pick_hex(&self, nx: f32, ny: f32, screen: [f32; 2]) -> Option<Axial> {
        let combat = self.combat.as_ref()?;
        crate::iso::pick_screen(
            nx * screen[0],
            ny * screen[1],
            screen[0],
            screen[1],
            combat.cols,
            combat.rows,
            self.ui.pan,
            self.ui.zoom,
            self.ui.yaw,
        )
    }

    pub fn hex_screen(&self, hex: Axial) -> [f32; 2] {
        let Some(combat) = self.combat.as_ref() else {
            return [0.5, 0.5];
        };
        let w = self.ui.screen[0].max(1.0);
        let h = self.ui.screen[1].max(1.0);
        crate::iso::hex_to_screen(
            hex,
            w,
            h,
            combat.cols,
            combat.rows,
            self.ui.pan,
            self.ui.zoom,
            8.0,
            self.ui.yaw,
        )
    }

    pub fn preview_zone(&self) -> Vec<Axial> {
        let Some(combat) = self.combat.as_ref() else {
            return vec![];
        };
        let Some(u) = current_unit(combat) else {
            return vec![];
        };
        if u.side != Side::Player {
            return vec![];
        }
        if let Some(skill_id) = &self.ui.selected_skill {
            if let Some(hover) = self.ui.hover {
                if let Some(skill) = catalog::skill(skill_id) {
                    return zone_for(combat, u, skill, hover);
                }
            }
            return legal_targets(combat, &u.id, skill_id);
        }
        legal_moves(combat, &u.id)
    }

    pub fn key(&mut self, code: winit::keyboard::KeyCode, down: bool) {
        if down {
            if !self.keys.contains(&code) {
                self.keys.push(code);
            }
            match code {
                winit::keyboard::KeyCode::Escape => {
                    audio::play("close");
                    if self.mode == Mode::Combat {
                        self.mode = Mode::World;
                        self.combat = None;
                    } else if self.mode == Mode::Scene {
                        // Decline / skip choice scenes; Ophelia still proceeds to fight.
                        if let Some(scene) = self.scene.as_ref() {
                            if scene.at_end() {
                                self.resolve_scene(false);
                            } else if let Some(s) = self.scene.as_mut() {
                                // jump to last line so player can still choose
                                while !s.at_end() {
                                    s.advance();
                                }
                            }
                        }
                    } else if self.mode != Mode::Title {
                        self.mode = Mode::Title;
                    }
                }
                winit::keyboard::KeyCode::Digit1 => {
                    audio::click();
                    self.ui.selected_skill = Some("cut".into());
                }
                winit::keyboard::KeyCode::Digit2 => {
                    audio::click();
                    self.ui.selected_skill = Some("guard".into());
                }
                winit::keyboard::KeyCode::Digit3 => {
                    audio::click();
                    self.ui.selected_skill = Some("aimed".into());
                }
                winit::keyboard::KeyCode::KeyG => {
                    audio::click();
                    self.ui.selected_skill = Some("guard".into());
                }
                winit::keyboard::KeyCode::KeyT => self.combat_act(PlayerAction::Raise),
                winit::keyboard::KeyCode::KeyQ => {
                    if self.mode == Mode::Combat {
                        self.ui.yaw = (self.ui.yaw + 3) % 4; // 90° CCW
                        audio::click();
                    }
                }
                winit::keyboard::KeyCode::KeyE => {
                    if self.mode == Mode::Combat {
                        self.ui.yaw = (self.ui.yaw + 1) % 4; // 90° CW
                        audio::click();
                    }
                }
                winit::keyboard::KeyCode::Space => {
                    if self.mode == Mode::Intro {
                        audio::click();
                        self.mode = Mode::World;
                    } else if self.mode == Mode::Scene {
                        if let Some(scene) = self.scene.as_ref() {
                            if scene.at_end() {
                                // default to the affirmative / only choice
                                self.resolve_scene(true);
                            } else {
                                audio::click();
                                if let Some(s) = self.scene.as_mut() {
                                    s.advance();
                                }
                            }
                        }
                    } else if self.mode == Mode::Combat {
                        self.combat_act(PlayerAction::Wait);
                    }
                }
                _ => {}
            }
        } else {
            self.keys.retain(|k| *k != code);
        }
    }
}
