//! Keyboard, hex pick, and zone preview.

use super::*;
use crate::audio;
use crate::catalog::{self};
use crate::combat::{
    current_unit, legal_moves, legal_targets, zone_for, PlayerAction, Side,
};
use crate::hex::Axial;
use crate::hud;


impl Game {
    pub fn hover_hex(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        self.ui.screen = screen;
        if self.mode != Mode::Combat || ny > hud::combat_bar().wait.y {
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
            // Edge-trigger actions; key-repeat must not re-fire Wait / skills / Raise.
            let fresh = !self.keys.contains(&code);
            if fresh {
                self.keys.push(code);
            }
            if !fresh {
                return;
            }
            match code {
                winit::keyboard::KeyCode::Escape => {
                    audio::play("close");
                    match self.mode {
                        Mode::Combat => {
                            if self.esc_arm > 0.0 {
                                self.mode = Mode::World;
                                self.combat = None;
                                self.esc_arm = 0.0;
                                self.ui.selected_skill = None;
                                audio::music_island();
                                self.persist();
                            } else {
                                self.esc_arm = 2.0;
                                self.fx.emit_hint(0.38, 0.78, "ESC AGAIN TO FLEE");
                            }
                        }
                        Mode::Scene => {
                            // Decline / skip choice scenes; Ophelia still proceeds to fight.
                            if let Some(scene) = self.scene.as_ref() {
                                if scene.at_end() {
                                    self.resolve_scene(false);
                                } else if let Some(s) = self.scene.as_mut() {
                                    while !s.at_end() {
                                        s.advance();
                                    }
                                }
                            }
                            self.persist();
                        }
                        Mode::Town | Mode::Codex | Mode::Result => {
                            self.mode = Mode::World;
                            self.esc_arm = 0.0;
                            audio::music_island();
                            self.persist();
                        }
                        Mode::World | Mode::Intro => {
                            // Persist first so Continue restores the island, not a stale fight.
                            self.persist();
                            self.mode = Mode::Title;
                            self.esc_arm = 0.0;
                        }
                        Mode::Title => {
                            // First Esc arms quit; app.rs exits on the second (native).
                            self.esc_arm = 2.0;
                            self.fx.emit_hint(0.38, 0.78, "ESC AGAIN TO QUIT");
                        }
                    }
                }
                winit::keyboard::KeyCode::Digit1 | winit::keyboard::KeyCode::Numpad1 => {
                    if self.mode == Mode::Combat {
                        audio::click();
                        self.ui.selected_skill = Some("cut".into());
                    }
                }
                winit::keyboard::KeyCode::Digit2 | winit::keyboard::KeyCode::Numpad2 => {
                    if self.mode == Mode::Combat {
                        audio::click();
                        self.ui.selected_skill = Some("guard".into());
                    }
                }
                winit::keyboard::KeyCode::Digit3 | winit::keyboard::KeyCode::Numpad3 => {
                    if self.mode == Mode::Combat {
                        audio::click();
                        self.pick_skill_slot(4);
                    }
                }
                winit::keyboard::KeyCode::KeyG => {
                    if self.mode == Mode::Combat {
                        audio::click();
                        self.ui.selected_skill = Some("guard".into());
                    }
                }
                winit::keyboard::KeyCode::KeyT => {
                    if self.mode == Mode::Combat {
                        self.combat_act(PlayerAction::Raise);
                    }
                }
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
