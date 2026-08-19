//! Mode machine: title → intro → island → town → hunt.

use crate::catalog::{self, INTRO};
use crate::combat::{
    act, create_battle, current_unit, legal_moves, legal_targets, run_ai, zone_for, CombatState,
    PlayerAction, Side,
};
use crate::hex::{hex_eq, Axial};
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
}

#[derive(Clone, Debug)]
pub struct Ui {
    pub selected_skill: Option<String>,
    pub hover: Option<Axial>,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub dragging: bool,
    pub last_mouse: [f32; 2],
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
}

impl Game {
    pub fn new() -> Self {
        let save = load_save();
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
            write_save(&s);
        }
    }

    pub fn new_hunt(&mut self) {
        self.mode = Mode::Intro;
        self.world = new_world();
        self.combat = None;
        self.result_win = None;
        self.ui = Ui::default();
        self.persist();
    }

    pub fn continue_hunt(&mut self) {
        if let Some(p) = load_save() {
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
        }
    }

    pub fn tick(&mut self, dt: f32) {
        if self.mode != Mode::World {
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
            return;
        }
        let len = (dx * dx + dy * dy).sqrt();
        dx /= len;
        dy /= len;
        let speed = 0.18;
        self.world.party_x = (self.world.party_x + dx * speed * dt).clamp(0.08, 0.92);
        self.world.party_y = (self.world.party_y + dy * speed * dt).clamp(0.10, 0.88);
        world::tick_hours(&mut self.world, dt * 2.4);
    }

    pub fn click(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        match self.mode {
            Mode::Title => self.click_title(nx, ny),
            Mode::Intro => self.mode = Mode::World,
            Mode::World => self.click_world(nx, ny),
            Mode::Town => self.click_town(nx, ny),
            Mode::Combat => self.click_combat(nx, ny, screen),
            Mode::Result => {
                self.mode = Mode::World;
                self.combat = None;
                self.result_win = None;
                self.persist();
            }
            Mode::Codex => self.mode = Mode::World,
        }
    }

    fn click_title(&mut self, nx: f32, ny: f32) {
        if nx > 0.08 && nx < 0.42 && ny > 0.72 && ny < 0.82 {
            self.new_hunt();
        } else if self.has_save && nx > 0.08 && nx < 0.42 && ny > 0.84 && ny < 0.93 {
            self.continue_hunt();
        }
    }

    fn click_world(&mut self, nx: f32, ny: f32) {
        if ny > 0.92 && nx > 0.82 {
            self.mode = Mode::Codex;
            return;
        }
        if let Some(loc) = world::nearest_location(nx, ny, 0.035) {
            let st = self.world.locations.get(loc.id).map(|s| s.status);
            if st == Some(world::WorldStatus::Locked) {
                return;
            }
            self.world.party_x = loc.x;
            self.world.party_y = loc.y;
            self.world.last_town = Some(loc.id.into());
            self.mode = Mode::Town;
            self.persist();
        }
    }

    fn click_town(&mut self, nx: f32, ny: f32) {
        let id = self
            .world
            .last_town
            .clone()
            .unwrap_or_else(|| "doga".into());
        let loc = catalog::location(&id);
        if nx > 0.08 && nx < 0.32 && ny > 0.78 && ny < 0.88 {
            if let Some(enc_id) = loc.and_then(|l| l.encounter) {
                let st = self.world.locations.get(&id).map(|s| s.status);
                if st == Some(world::WorldStatus::Cleared) || st == Some(world::WorldStatus::Locked)
                {
                    return;
                }
                self.start_encounter(enc_id);
            }
        } else if nx > 0.36 && nx < 0.56 && ny > 0.78 && ny < 0.88 {
            self.world.hours += 8.0;
            self.persist();
        } else if nx > 0.60 && nx < 0.80 && ny > 0.78 && ny < 0.88 {
            self.mode = Mode::World;
        }
    }

    pub fn start_encounter(&mut self, id: &str) {
        let Some(enc) = catalog::encounter(id) else {
            return;
        };
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
        self.ui = Ui::default();
        self.persist();
    }

    fn click_combat(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        // HUD buttons along the bottom.
        if ny > 0.88 {
            if nx < 0.14 {
                self.combat_act(PlayerAction::Wait);
            } else if nx < 0.28 {
                self.combat_act(PlayerAction::Raise);
            } else if nx < 0.42 {
                self.ui.selected_skill = Some("guard".into());
            } else if nx < 0.56 {
                self.ui.selected_skill = Some("cut".into());
            } else if nx < 0.70 {
                self.pick_skill_slot(4);
            } else if nx > 0.88 {
                // forfeit
                self.finish_combat(false);
            }
            return;
        }
        let Some(hex) = self.pick_hex(nx, ny, screen) else {
            return;
        };
        let Some(combat) = self.combat.as_ref() else {
            return;
        };
        let Some(u) = current_unit(combat) else {
            return;
        };
        if u.side != Side::Player {
            return;
        }
        if let Some(skill_id) = self.ui.selected_skill.clone() {
            let targets = legal_targets(combat, &u.id, &skill_id);
            if targets.iter().any(|h| hex_eq(*h, hex)) {
                self.combat_act(PlayerAction::Skill { id: skill_id, hex });
                self.ui.selected_skill = None;
            }
            return;
        }
        let moves = legal_moves(combat, &u.id);
        if moves.iter().any(|h| hex_eq(*h, hex)) {
            self.combat_act(PlayerAction::Move(hex));
        } else {
            self.ui.selected_skill = Some("cut".into());
            let combat = self.combat.as_ref().unwrap();
            let u = current_unit(combat).unwrap();
            let targets = legal_targets(combat, &u.id, "cut");
            if targets.iter().any(|h| hex_eq(*h, hex)) {
                self.combat_act(PlayerAction::Skill {
                    id: "cut".into(),
                    hex,
                });
                self.ui.selected_skill = None;
            }
        }
    }

    fn pick_skill_slot(&mut self, nth: usize) {
        let Some(combat) = self.combat.as_ref() else {
            return;
        };
        let Some(u) = current_unit(combat) else {
            return;
        };
        if let Some(id) = u.skills.get(nth) {
            self.ui.selected_skill = Some(id.clone());
        }
    }

    pub fn combat_act(&mut self, action: PlayerAction) {
        let raki = self.world.raki;
        let Some(combat) = self.combat.as_mut() else {
            return;
        };
        act(combat, action, raki);
        if current_unit(combat)
            .map(|u| u.side == Side::Enemy)
            .unwrap_or(false)
        {
            run_ai(combat);
        }
        if let Some(win) = combat.over {
            let id = combat.id.clone();
            self.finish_combat_id(win, &id);
        } else {
            self.persist();
        }
    }

    fn finish_combat(&mut self, win: bool) {
        let id = self
            .combat
            .as_ref()
            .map(|c| c.id.clone())
            .unwrap_or_default();
        self.finish_combat_id(win, &id);
    }

    fn finish_combat_id(&mut self, win: bool, id: &str) {
        if win {
            apply_victory(&mut self.world, id);
            self.result_title = "The board is quiet.".into();
            self.result_body =
                "You walk back with blood on the silver. The beacon goes dark.".into();
        } else {
            self.result_title = "You fall.".into();
            self.result_body = "The Organization will send another number.".into();
        }
        self.result_win = Some(win);
        self.mode = Mode::Result;
        self.combat = None;
        self.persist();
    }

    pub fn hover_hex(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
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

    pub fn intro_text(&self) -> &'static str {
        INTRO
    }

    pub fn key(&mut self, code: winit::keyboard::KeyCode, down: bool) {
        if down {
            if !self.keys.contains(&code) {
                self.keys.push(code);
            }
            match code {
                winit::keyboard::KeyCode::Escape => {
                    if self.mode == Mode::Combat {
                        self.mode = Mode::World;
                        self.combat = None;
                    } else if self.mode != Mode::Title {
                        self.mode = Mode::Title;
                    }
                }
                winit::keyboard::KeyCode::Digit1 => self.ui.selected_skill = Some("cut".into()),
                winit::keyboard::KeyCode::Digit2 => self.ui.selected_skill = Some("guard".into()),
                winit::keyboard::KeyCode::Digit3 => self.ui.selected_skill = Some("aimed".into()),
                winit::keyboard::KeyCode::KeyG => self.ui.selected_skill = Some("guard".into()),
                winit::keyboard::KeyCode::KeyT => self.combat_act(PlayerAction::Raise),
                winit::keyboard::KeyCode::Space => {
                    if self.mode == Mode::Intro {
                        self.mode = Mode::World;
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

fn load_save() -> Option<Persist> {
    let raw = read_save()?;
    serde_json::from_str(&raw).ok()
}

fn read_save() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item("claymore.save.v1").ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string("claymore.save.json").ok()
    }
}

fn write_save(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("claymore.save.v1", s);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write("claymore.save.json", s);
    }
}
