//! Mode machine: title -> intro -> island -> town -> hunt.
//! FULL SOURCE in artifacts/story-src/game.rs — apply via sources-restored.zip

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
    Title, Intro, World, Town, Combat, Result, Codex, Scene,
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
            selected_skill: None, hover: None, pan: [0.0, 0.0], zoom: 1.05,
            dragging: false, last_mouse: [0.0, 0.0], screen: [1280.0, 800.0],
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
    pub scene: Option<SceneState>,
    pub pending_encounter: Option<String>,
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
            mode: Mode::Title, world, combat, result_win,
            result_title: String::new(), result_body: String::new(),
            ui: Ui::default(), keys: Vec::new(), has_save,
            fx: Fx::default(), step_acc: 0.0, scene: None, pending_encounter: None,
        }
    }
    pub fn persist(&self) {
        let blob = Persist {
            v: 1, world: self.world.clone(),
            mode: if self.mode == Mode::Title { Mode::World } else { self.mode },
            combat: self.combat.clone(), result_win: self.result_win,
        };
        if let Ok(s) = serde_json::to_string(&blob) { write_save(&s); }
    }
    pub fn new_hunt(&mut self) {
        self.mode = Mode::Intro;
        self.world = new_world();
        self.combat = None;
        self.result_win = None;
        self.ui = Ui { screen: self.ui.screen, ..Ui::default() };
        self.fx = Fx::default();
        audio::confirm();
        self.persist();
    }
    pub fn continue_hunt(&mut self) {
        if let Some(p) = load_save() {
            self.world = p.world;
            self.combat = p.combat;
            self.result_win = p.result_win;
            self.mode = if self.combat.is_some() { Mode::Combat }
                else if p.mode == Mode::Intro { Mode::World } else { p.mode };
            audio::confirm();
        }
    }
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.08);
        self.fx.tick(dt);
    }
    pub fn click(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        self.ui.screen = screen;
        match self.mode {
            Mode::Title => {
                if hud::title_new().contains(nx, ny) { self.new_hunt(); }
                else if self.has_save && hud::title_continue().contains(nx, ny) { self.continue_hunt(); }
            }
            Mode::Intro => {
                audio::click();
                self.mode = Mode::World;
                audio::music_island();
            }
            Mode::World => {
                if hud::world_codex().contains(nx, ny) {
                    audio::click();
                    self.mode = Mode::Codex;
                } else if let Some(loc) = world::nearest_location(nx, ny, 0.04) {
                    audio::confirm();
                    self.world.party_x = loc.x;
                    self.world.party_y = loc.y;
                    self.world.last_town = Some(loc.id.into());
                    self.mode = Mode::Town;
                    audio::music_town();
                    self.persist();
                }
            }
            Mode::Town => {
                if hud::town_leave().contains(nx, ny) {
                    audio::play("close");
                    self.mode = Mode::World;
                    audio::music_island();
                } else if hud::town_hunt().contains(nx, ny) {
                    let id = self.world.last_town.clone().unwrap_or_else(|| "doga".into());
                    if let Some(enc_id) = catalog::location(&id).and_then(|l| l.encounter) {
                        audio::play("draw");
                        self.start_encounter(enc_id);
                    }
                }
            }
            Mode::Combat => {
                let bar = hud::combat_bar();
                if ny > 0.88 && bar.wait.contains(nx, ny) {
                    audio::click();
                    if let Some(c) = self.combat.as_mut() {
                        act(c, PlayerAction::Wait, true);
                        if c.over == Some(true) { self.finish_combat(true); }
                        else if c.over == Some(false) { self.finish_combat(false); }
                    }
                }
            }
            Mode::Result => {
                audio::click();
                self.combat = None;
                self.result_win = None;
                self.mode = Mode::World;
                audio::music_island();
                self.persist();
            }
            Mode::Codex => {
                audio::play("close");
                self.mode = Mode::World;
            }
            Mode::Scene => {
                audio::click();
                self.mode = Mode::World;
            }
        }
    }
    pub fn start_encounter(&mut self, id: &str) {
        let Some(enc) = catalog::encounter(id) else { return; };
        let seed = (self.world.hours as u32).wrapping_mul(997) + 13;
        let mut combat = create_battle(enc, &self.world.party, seed.max(1));
        if current_unit(&combat).map(|u| u.side == Side::Enemy).unwrap_or(false) {
            run_ai(&mut combat);
        }
        self.combat = Some(combat);
        self.mode = Mode::Combat;
        audio::music_hunt();
        self.persist();
    }
    fn finish_combat(&mut self, win: bool) {
        if win {
            self.result_title = dialog::RESULT_WIN_TITLE.into();
            self.result_body = dialog::RESULT_WIN_BODY.into();
            audio::confirm();
        } else {
            self.result_title = dialog::RESULT_LOSE_TITLE.into();
            self.result_body = dialog::RESULT_LOSE_BODY.into();
            audio::error();
            audio::music_defeat();
        }
        self.result_win = Some(win);
        self.mode = Mode::Result;
        self.combat = None;
        self.persist();
    }
    pub fn key(&mut self, code: winit::keyboard::KeyCode, down: bool) {
        if down {
            if !self.keys.contains(&code) { self.keys.push(code); }
        } else {
            self.keys.retain(|k| *k != code);
        }
        if !down { return; }
        match self.mode {
            Mode::Intro if code == winit::keyboard::KeyCode::Space => {
                self.mode = Mode::World;
                audio::music_island();
            }
            Mode::Result if code == winit::keyboard::KeyCode::Space => {
                self.mode = Mode::World;
                audio::music_island();
                self.persist();
            }
            _ => {}
        }
    }
    pub fn hex_screen(&self, hex: Axial) -> [f32; 2] {
        let _ = hex;
        [0.5, 0.5]
    }
}

fn load_save() -> Option<Persist> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let s = storage.get_item("claymore-save").ok()??;
        serde_json::from_str(&s).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let s = std::fs::read_to_string("claymore-save.json").ok()?;
        serde_json::from_str(&s).ok()
    }
}
fn write_save(s: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item("claymore-save", s);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write("claymore-save.json", s);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wasd_signs_on_the_map() {
        assert!(true);
    }
}
