//! Mode machine: title -> intro -> island -> town -> hunt.

mod click;
mod combat_flow;
mod input;
mod save;
mod scene;

use crate::audio;
use crate::catalog::{self};
use crate::combat::{
    act, core_hex, create_battle, current_unit, legal_moves, legal_targets, run_ai, zone_for,
    CombatState, PlayerAction, Side,
};
use crate::dialog::{self, SceneId, SceneState};
use crate::fx::Fx;
use crate::hex::{hex_eq, Axial};
use crate::hud;
use crate::world::{self, apply_victory, new_world, WorldState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Title,
    Intro,
    World,
    Town,
    Combat,
    Result,
    Codex,
    Scene,
}

#[derive(Clone, Debug)]
pub struct Ui {
    pub selected_skill: Option<String>,
    pub hover: Option<Axial>,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub dragging: bool,
    pub last_mouse: [f32; 2],
    pub screen: [f32; 2],
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            selected_skill: None,
            hover: None,
            pan: [0.0, 0.0],
            zoom: 1.05,
            dragging: false,
            last_mouse: [0.0, 0.0],
            screen: [1280.0, 800.0],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persist {
    pub v: u32,
    pub world: WorldState,
    pub mode: Mode,
    pub combat: Option<CombatState>,
    pub result_win: Option<bool>,
}

pub struct Game {
    pub mode: Mode,
    pub world: WorldState,
    pub combat: Option<CombatState>,
    pub result_win: Option<bool>,
    pub result_title: String,
    pub result_body: String,
    pub ui: Ui,
    pub keys: Vec<winit::keyboard::KeyCode>,
    pub has_save: bool,
    pub fx: Fx,
    pub step_acc: f32,
    /// 1 = facing right, -1 = facing left (world map Clare).
    pub facing: f32,
    /// True while Clare is moving on the island.
    pub walking: bool,
    pub scene: Option<SceneState>,
    pub pending_encounter: Option<String>,
}


impl Game {
    pub fn new() -> Self {
        let save = save::load_save();
        let has_save = save.is_some();
        let (world, combat, result_win) = if let Some(p) = save {
            (p.world, p.combat, p.result_win)
        } else {
            (new_world(), None, None)
        };
        Self {
            mode: Mode::Title,
            world,
            combat,
            result_win,
            result_title: String::new(),
            result_body: String::new(),
            ui: Ui::default(),
            keys: Vec::new(),
            has_save,
            fx: Fx::default(),
            step_acc: 0.0,
            facing: 1.0,
            walking: false,
            scene: None,
            pending_encounter: None,
        }
    }

    pub fn persist(&self) {
        let blob = Persist {
            v: 1,
            world: self.world.clone(),
            mode: if self.mode == Mode::Title {
                Mode::World
            } else {
                self.mode
            },
            combat: self.combat.clone(),
            result_win: self.result_win,
        };
        if let Ok(s) = serde_json::to_string(&blob) {
            save::write_save(&s);
        }
    }

    pub fn new_hunt(&mut self) {
        self.mode = Mode::Intro;
        self.world = new_world();
        self.combat = None;
        self.result_win = None;
        self.ui = Ui {
            screen: self.ui.screen,
            ..Ui::default()
        };
        self.fx = Fx::default();
        self.facing = 1.0;
        self.walking = false;
        self.step_acc = 0.0;
        audio::confirm();
        self.persist();
    }

    pub fn continue_hunt(&mut self) {
        if let Some(p) = save::load_save() {
            self.world = p.world;
            self.combat = p.combat;
            self.result_win = p.result_win;
            self.mode = if self.combat.is_some() {
                Mode::Combat
            } else if p.mode == Mode::Intro {
                Mode::World
            } else {
                p.mode
            };
            audio::confirm();
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.08);
        self.fx.tick(dt);
        match self.mode {
            Mode::Title | Mode::Intro => {
                if self.fx.time % 0.45 < dt {
                    self.fx.emit_mote(
                        0.15 + (self.fx.time * 0.17).fract() * 0.7,
                        0.2 + (self.fx.time * 0.11).sin().abs() * 0.5,
                    );
                }
            }
            Mode::World => self.tick_world(dt),
            Mode::Combat => {
                let hexes: Vec<Axial> = self
                    .combat
                    .as_ref()
                    .map(|c| {
                        c.units
                            .iter()
                            .filter(|u| !u.dead)
                            .map(|u| core_hex(u))
                            .collect()
                    })
                    .unwrap_or_default();
                if self.fx.time % 0.8 < dt {
                    for hex in hexes {
                        let p = self.hex_screen(hex);
                        self.fx.emit_mote(p[0], p[1] + 0.02);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn tick_world(&mut self, dt: f32) {
        if self.fx.hitstop > 0.0 {
            self.walking = false;
            return;
        }
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        for k in &self.keys {
            match k {
                winit::keyboard::KeyCode::KeyA | winit::keyboard::KeyCode::ArrowLeft => dx -= 1.0,
                winit::keyboard::KeyCode::KeyD | winit::keyboard::KeyCode::ArrowRight => dx += 1.0,
                winit::keyboard::KeyCode::KeyW | winit::keyboard::KeyCode::ArrowUp => dy -= 1.0,
                winit::keyboard::KeyCode::KeyS | winit::keyboard::KeyCode::ArrowDown => dy += 1.0,
                _ => {}
            }
        }
        if dx == 0.0 && dy == 0.0 {
            self.walking = false;
            return;
        }
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;
        // Face the direction of travel (prefer horizontal when both axes pressed).
        if dx.abs() > 0.01 {
            self.facing = if dx > 0.0 { 1.0 } else { -1.0 };
        }
        self.walking = true;
        let speed = 0.18;
        self.world.party_x = (self.world.party_x + dx * speed * dt).clamp(0.08, 0.92);
        self.world.party_y = (self.world.party_y + dy * speed * dt).clamp(0.10, 0.88);
        world::tick_hours(&mut self.world, dt * 2.4);
        self.step_acc += dt;
        if self.step_acc > 0.32 {
            self.step_acc = 0.0;
            self.fx.emit_step(self.world.party_x, self.world.party_y);
            audio::play("step");
        }
    }

    pub fn intro_text(&self) -> &'static str {
        dialog::INTRO
    }

    pub fn title_flavor(&self) -> &'static str {
        let i = (self.world.hours as usize) % dialog::TITLE_FLAVOR.len();
        dialog::TITLE_FLAVOR[i]
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasd_signs_on_the_map() {
        let mut g = Game::new();
        g.mode = Mode::World;
        g.world.party_x = 0.5;
        g.world.party_y = 0.5;
        g.keys = vec![winit::keyboard::KeyCode::KeyA];
        g.tick(0.2);
        assert!(g.world.party_x < 0.5, "A walks left on the island");
        g.keys = vec![winit::keyboard::KeyCode::KeyD];
        let x = g.world.party_x;
        g.tick(0.2);
        assert!(g.world.party_x > x, "D walks right on the island");
        g.keys = vec![winit::keyboard::KeyCode::KeyW];
        let y = g.world.party_y;
        g.tick(0.2);
        assert!(g.world.party_y < y, "W walks up the painted map");
    }
}
