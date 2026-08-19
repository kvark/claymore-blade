//! Mode machine: title → intro → island → town → hunt.

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
            write_save(&s);
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
        audio::confirm();
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

    fn tick_world(&mut self, dt: f32) {
        if self.fx.hitstop > 0.0 {
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
        self.step_acc += dt;
        if self.step_acc > 0.32 {
            self.step_acc = 0.0;
            self.fx.emit_step(self.world.party_x, self.world.party_y);
            audio::play("step");
        }
    }

    pub fn click(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        self.ui.screen = screen;
        match self.mode {
            Mode::Title => self.click_title(nx, ny),
            Mode::Intro => {
                audio::click();
                self.mode = Mode::World;
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

    fn click_result(&mut self) {
        audio::click();
        self.combat = None;
        self.result_win = None;
        if self.scene.is_some() {
            self.mode = Mode::Scene;
        } else {
            self.mode = Mode::World;
        }
        self.persist();
    }

    fn click_title(&mut self, nx: f32, ny: f32) {
        if hud::title_new().contains(nx, ny) {
            self.new_hunt();
        } else if self.has_save && hud::title_continue().contains(nx, ny) {
            self.continue_hunt();
        }
    }

    fn click_world(&mut self, nx: f32, ny: f32) {
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
            }
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
        if hud::town_hunt().contains(nx, ny) {
            if let Some(enc_id) = loc.and_then(|l| l.encounter) {
                let st = self.world.locations.get(&id).map(|s| s.status);
                if st == Some(world::WorldStatus::Cleared) || st == Some(world::WorldStatus::Locked)
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

    fn begin_battle(&mut self, enc: &catalog::EncounterDef) {
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
        self.ui = Ui {
            screen: self.ui.screen,
            ..Ui::default()
        };
        self.fx = Fx::default();
        self.persist();
    }

    fn click_combat(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        let bar = hud::combat_bar();
        if ny > 0.88 {
            if bar.wait.contains(nx, ny) {
                audio::click();
                self.combat_act(PlayerAction::Wait);
            } else if bar.raise.contains(nx, ny) {
                self.combat_act(PlayerAction::Raise);
            } else if bar.guard.contains(nx, ny) {
                audio::click();
                self.ui.selected_skill = Some("guard".into());
            } else if bar.cut.contains(nx, ny) {
                audio::click();
                self.ui.selected_skill = Some("cut".into());
            } else if bar.slot.contains(nx, ny) {
                audio::click();
                self.pick_skill_slot(4);
            } else if bar.forfeit.contains(nx, ny) {
                audio::error();
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
            } else {
                audio::error();
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
        } else {
            audio::error();
        }
    }

    pub fn combat_act(&mut self, action: PlayerAction) {
        let raki = self.world.raki;
        let Some(combat) = self.combat.as_ref() else {
            return;
        };
        let before: Vec<(String, i32, bool, Axial, i32, i32)> = combat
            .units
            .iter()
            .map(|u| {
                (
                    u.id.clone(),
                    u.hp,
                    u.dead,
                    u.origin,
                    u.trans,
                    u.yoki,
                )
            })
            .collect();
        let actor = current_unit(combat).map(|u| (u.id.clone(), core_hex(u)));
        let log_len = combat.log.len();

        let Some(combat) = self.combat.as_mut() else {
            return;
        };
        act(combat, action.clone(), raki);
        if current_unit(combat)
            .map(|u| u.side == Side::Enemy)
            .unwrap_or(false)
        {
            run_ai(combat);
        }
        self.juice_from_act(&before, actor, &action, log_len);
        if let Some(win) = self.combat.as_ref().and_then(|c| c.over) {
            let id = self.combat.as_ref().unwrap().id.clone();
            self.finish_combat_id(win, &id);
        } else {
            self.persist();
        }
    }

    fn juice_from_act(
        &mut self,
        before: &[(String, i32, bool, Axial, i32, i32)],
        actor: Option<(String, Axial)>,
        action: &PlayerAction,
        log_len: usize,
    ) {
        match action {
            PlayerAction::Move(_) => audio::play("step"),
            PlayerAction::Raise => audio::play("raise"),
            PlayerAction::Wait => audio::play("cloth"),
            PlayerAction::Skill { id, .. } => {
                if id == "guard" {
                    audio::play("block");
                } else {
                    audio::play("slash");
                }
            }
        }
        if let Some((_, hex)) = actor {
            let p = self.hex_screen(hex);
            match action {
                PlayerAction::Move(dest) => {
                    let q = self.hex_screen(*dest);
                    self.fx.emit_step(q[0], q[1]);
                }
                PlayerAction::Raise => self.fx.emit_raise(p[0], p[1]),
                PlayerAction::Skill { id, hex: target } => {
                    let q = self.hex_screen(*target);
                    if id == "guard" {
                        self.fx.emit_guard(p[0], p[1]);
                    } else {
                        self.fx.emit_hit(q[0], q[1], 0, "windup");
                    }
                }
                PlayerAction::Wait => {}
            }
        }
        #[derive(Clone)]
        struct Ev {
            hex: Axial,
            dmg: i32,
            kind: &'static str,
            moved: bool,
            trans_up: bool,
            heal: i32,
        }
        let events: Vec<Ev> = self
            .combat
            .as_ref()
            .map(|combat| {
                combat
                    .units
                    .iter()
                    .filter_map(|u| {
                        let prev = before.iter().find(|b| b.0 == u.id)?;
                        Some(Ev {
                            hex: core_hex(u),
                            dmg: (prev.1 - u.hp).max(0),
                            kind: if u.dead && !prev.2 { "death" } else { "hit" },
                            moved: u.origin.q != prev.3.q || u.origin.r != prev.3.r,
                            trans_up: u.trans > prev.4 + 4,
                            heal: (u.hp - prev.1).max(0),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        for ev in events {
            let p = self.hex_screen(ev.hex);
            if ev.moved {
                self.fx.emit_step(p[0], p[1]);
            }
            if ev.dmg > 0 {
                self.fx.emit_hit(p[0], p[1] - 0.02, ev.dmg, ev.kind);
                audio::play("hit");
            }
            if ev.heal > 0 {
                self.fx.emit_heal(p[0], p[1], ev.heal);
            }
            if ev.trans_up {
                self.fx.emit_raise(p[0], p[1]);
            }
        }
        let kinds: Vec<String> = self
            .combat
            .as_ref()
            .map(|c| {
                c.log
                    .iter()
                    .take(c.log.len().saturating_sub(log_len).min(8))
                    .map(|l| format!("{}|{}", l.kind, l.text))
                    .collect()
            })
            .unwrap_or_default();
        for line in kinds {
            if line.starts_with("miss|") {
                audio::play("miss");
            } else if line.contains("catches") {
                audio::play("block");
            } else if line.starts_with("sever|") {
                audio::play("chop");
            }
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
            self.result_title = dialog::RESULT_WIN_TITLE.into();
            self.result_body = dialog::RESULT_WIN_BODY.into();
            self.fx.emit_win();
            audio::confirm();
            self.scene = match id {
                "doga-yoma" if !self.world.raki => Some(SceneState::new(SceneId::RakiJoin)),
                "paburo-nest"
                    if !self.world.party.iter().any(|p| p == "miria" || p == "helen") =>
                {
                    Some(SceneState::new(SceneId::RecruitPaburo))
                }
                "pieta-worm" if !self.world.party.iter().any(|p| p == "deneve") => {
                    Some(SceneState::new(SceneId::RecruitPieta))
                }
                _ => None,
            };
        } else {
            self.result_title = dialog::RESULT_LOSE_TITLE.into();
            self.result_body = dialog::RESULT_LOSE_BODY.into();
            audio::error();
            self.scene = None;
        }
        self.result_win = Some(win);
        self.mode = Mode::Result;
        self.combat = None;
        self.persist();
    }

    fn click_scene(&mut self, nx: f32, ny: f32) {
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
            // single-choice scenes also advance on any click below the panel
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

    fn resolve_scene(&mut self, yes: bool) {
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
                    // He follows anyway next town — flag soft refuse.
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
        }
        self.persist();
    }

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
        dialog::INTRO
    }

    pub fn title_flavor(&self) -> &'static str {
        let i = (self.world.hours as usize) % dialog::TITLE_FLAVOR.len();
        dialog::TITLE_FLAVOR[i]
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
