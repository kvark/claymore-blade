//! Mode machine: title -> intro -> island -> town -> hunt.

pub mod beats;
mod click;
mod combat_flow;
mod input;
mod save;
mod scene;

use crate::audio;
use crate::combat::{
    core_hex,
    CombatState,
};
use crate::dialog::{self, SceneState};
use crate::fx::Fx;
use crate::hex::Axial;
use crate::world::{self, new_world, WorldState};
use serde::{Deserialize, Serialize};

pub use beats::{CutIn, ENEMY_BEAT, ENEMY_WINDUP};

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
    /// Combat camera yaw in 90° steps (0..=3). Q / E cycle.
    pub yaw: u8,
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
            yaw: 0,
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
    /// Pending story scene (Raki join, recruits). Saves before this field
    /// existed load as `None`; see [`resume_mode`].
    #[serde(default)]
    pub scene: Option<SceneState>,
    /// Encounter queued behind a scene (Ophelia intro -> Gonal ripple).
    #[serde(default)]
    pub pending_encounter: Option<String>,
}

/// Mode to land in after CONTINUE. `Scene` / `Result` are only resumable when
/// the scene they lead into was saved; older saves (and Result screens, whose
/// text is not persisted) fall back to the island instead of a blank backdrop.
pub fn resume_mode(saved: Mode, has_combat: bool, has_scene: bool) -> Mode {
    if has_combat {
        return Mode::Combat;
    }
    match saved {
        Mode::Scene | Mode::Result if has_scene => Mode::Scene,
        Mode::Scene | Mode::Result | Mode::Intro | Mode::Title | Mode::Combat => Mode::World,
        other => other,
    }
}


/// Combat wheel-zoom floor (playtest: 0.55 overshoots into tiny tiles).
pub const COMBAT_ZOOM_MIN: f32 = 0.7;
/// Combat wheel-zoom ceiling (playtest: 2.2 clips huge tiles).
pub const COMBAT_ZOOM_MAX: f32 = 1.55;

pub fn clamp_combat_zoom(zoom: f32) -> f32 {
    zoom.clamp(COMBAT_ZOOM_MIN, COMBAT_ZOOM_MAX)
}

/// Clamp wheel zoom so the framed board stays between the HUD strips.
pub fn clamp_board_zoom(zoom: f32, cols: i32, rows: i32, w: f32, h: f32, yaw: u8) -> f32 {
    let cap = crate::iso::board_zoom_max(cols, rows, w, h, yaw).min(COMBAT_ZOOM_MAX);
    zoom.clamp(COMBAT_ZOOM_MIN, cap)
}

/// Share of the HUD-fit zoom cap the default combat view fills; the rest is
/// headroom for the claw cut-in push-in.
pub const COMBAT_FRAME_FILL: f32 = 0.95;

/// Default combat camera `(yaw, zoom)` for a board on this screen: the 90°
/// yaw with the most HUD-fit headroom (landscape: yaw 1, the board's long
/// axis across the screen; portrait: yaw 0), filled to `COMBAT_FRAME_FILL`.
pub fn default_combat_view(cols: i32, rows: i32, w: f32, h: f32) -> (u8, f32) {
    let mut best = (0u8, 0.0_f32);
    for yaw in [1u8, 0, 3, 2] {
        let cap = crate::iso::board_zoom_max(cols, rows, w, h, yaw).min(COMBAT_ZOOM_MAX);
        if cap > best.1 + 1e-3 {
            best = (yaw, cap);
        }
    }
    let (yaw, cap) = best;
    (yaw, (cap * COMBAT_FRAME_FILL).max(1.05_f32.min(cap)).max(COMBAT_ZOOM_MIN))
}

/// Island walk speed in normalized map units per second (~8–15s Doga→Pieta).
pub const WORLD_WALK_SPEED: f32 = 0.055;
/// World-hour advance per real second while walking.
pub const WORLD_HOUR_RATE: f32 = 1.0;
/// Footfall / bob cycle length; dust + audio share this cadence.
pub const WORLD_STEP_PERIOD: f32 = 0.32;


/// Presentation-only windup before an enemy skill resolves.
#[derive(Clone, Debug)]
pub struct EnemyTell {
    pub unit_id: String,
    pub skill_id: String,
    pub hex: Axial,
    pub resolve_at: f32,
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
    /// Vertical facing sense: -1 north / up, +1 south / down (banner lean).
    pub facing_y: f32,
    /// True while Clare is moving on the island.
    pub walking: bool,
    pub scene: Option<SceneState>,
    pub pending_encounter: Option<String>,
    /// Seconds left to confirm Combat Esc flee (0 = not armed).
    pub esc_arm: f32,
    /// Armed yoma claw windup (hex + stretch) before HP drops.
    pub enemy_tell: Option<EnemyTell>,
    /// Plated "YOMA REACHES" shown once per fight.
    pub enemy_tell_announced: bool,
    /// When the next paced enemy action fires (None = not scheduled).
    pub enemy_beat_at: Option<f32>,
    /// Paced actions spent in the current enemy phase (runaway guard).
    pub enemy_phase_beats: u32,
    /// Claw-windup cut-in (speed lines + camera push-in), presentation only.
    pub cutin: Option<CutIn>,
}


impl Game {
    pub fn new() -> Self {
        let save = save::load_save();
        let has_save = save.is_some();
        let (mut world, combat, result_win) = if let Some(p) = save {
            (p.world, p.combat, p.result_win)
        } else {
            (new_world(), None, None)
        };
        world::ensure_explored(&mut world);
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
            facing_y: 0.0,
            walking: false,
            scene: None,
            pending_encounter: None,
            esc_arm: 0.0,
            enemy_tell: None,
            enemy_tell_announced: false,
            enemy_beat_at: None,
            enemy_phase_beats: 0,
            cutin: None,
        }
    }

    /// Drop every enemy-phase presentation timer (new fight, Continue, result).
    pub(crate) fn reset_enemy_pacing(&mut self) {
        self.enemy_tell = None;
        self.enemy_tell_announced = false;
        self.enemy_beat_at = None;
        self.enemy_phase_beats = 0;
        self.cutin = None;
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
            scene: self.scene.clone(),
            pending_encounter: self.pending_encounter.clone(),
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
        self.facing_y = 0.0;
        self.walking = false;
        self.step_acc = 0.0;
        self.reset_enemy_pacing();
        audio::confirm();
        self.persist();
    }

    pub fn continue_hunt(&mut self) {
        let Some(p) = save::load_save() else {
            // Unreadable / vanished save: stay on the title with NEW HUNT only.
            self.has_save = false;
            return;
        };
        self.world = p.world;
        world::ensure_explored(&mut self.world);
        self.combat = p.combat;
        // Windups belong to the previous in-memory battle. The next combat
        // tick reconstructs any pending enemy action from the loaded turn.
        self.reset_enemy_pacing();
        self.result_win = p.result_win;
        self.scene = p.scene;
        self.pending_encounter = p.pending_encounter;
        self.mode = resume_mode(p.mode, self.combat.is_some(), self.scene.is_some());
        if self.mode != Mode::Scene {
            self.scene = None;
        }
        if self.mode == Mode::World {
            self.result_win = None;
        }
        self.frame_combat_view();
        audio::confirm();
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.08);
        self.fx.tick(dt);
        if self.esc_arm > 0.0 {
            self.esc_arm = (self.esc_arm - dt).max(0.0);
        }
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
                self.fast_forward_enemy(dt);
                self.tick_enemy_tell();
            }
            _ => {}
        }
    }

    /// Fast-forward key held (Shift, or Space still down after WAIT).
    pub fn fast_forward_held(&self) -> bool {
        use winit::keyboard::KeyCode;
        self.keys
            .iter()
            .any(|k| matches!(k, KeyCode::ShiftLeft | KeyCode::ShiftRight | KeyCode::Space))
    }

    /// True while enemy beats are paced and the fast-forward key is held.
    pub fn fast_forwarding(&self) -> bool {
        self.mode == Mode::Combat
            && self.combat.as_ref().map(|c| c.over.is_none()).unwrap_or(false)
            && !self.combat_input_open()
            && self.fast_forward_held()
    }

    /// Pull every pending enemy-phase timer earlier so beats, the claw
    /// windup and its cut-in envelope all play `FAST_FORWARD`x.
    pub(super) fn fast_forward_enemy(&mut self, dt: f32) {
        let skip = beats::fast_forward_skip(self.fast_forward_held(), self.fast_forwarding(), dt);
        if skip <= 0.0 {
            return;
        }
        if let Some(at) = self.enemy_beat_at.as_mut() {
            *at -= skip;
        }
        if let Some(tell) = self.enemy_tell.as_mut() {
            tell.resolve_at -= skip;
        }
        if let Some(cut) = self.cutin.as_mut() {
            cut.start -= skip;
            cut.strike_at -= skip;
            if let Some(h) = cut.handoff.as_mut() {
                h.at -= skip;
            }
        }
    }

    pub(super) fn tick_world(&mut self, dt: f32) {
        // FoW: stamp vision every frame on the island (even while idle).
        let (px, py) = (self.world.party_x, self.world.party_y);
        world::stamp_explored(&mut self.world, px, py, world::VISION_RADIUS);
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
        // Facing follows movement: L/R flip + N/S lean sense.
        if dx.abs() > 0.01 {
            self.facing = if dx > 0.0 { 1.0 } else { -1.0 };
        }
        if dy.abs() > 0.01 {
            self.facing_y = if dy > 0.0 { 1.0 } else { -1.0 };
        } else {
            self.facing_y = 0.0;
        }
        self.walking = true;
        let speed = WORLD_WALK_SPEED;
        self.world.party_x = (self.world.party_x + dx * speed * dt).clamp(0.08, 0.92);
        self.world.party_y = (self.world.party_y + dy * speed * dt).clamp(0.10, 0.88);
        world::tick_hours(&mut self.world, dt * WORLD_HOUR_RATE);
        self.step_acc += dt;
        if self.step_acc >= WORLD_STEP_PERIOD {
            self.step_acc -= WORLD_STEP_PERIOD;
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

    /// Reset pan and pick the default yaw / zoom for the loaded board.
    pub fn frame_combat_view(&mut self) {
        if let Some(c) = self.combat.as_ref() {
            let [w, h] = self.ui.screen;
            let (yaw, zoom) = default_combat_view(c.cols, c.rows, w.max(1.0), h.max(1.0));
            self.ui.pan = [0.0, 0.0];
            self.ui.yaw = yaw;
            self.ui.zoom = zoom;
        }
    }

    /// Keep combat zoom inside the HUD gap for the current board, yaw, and screen.
    pub fn clamp_ui_zoom(&mut self) {
        self.ui.zoom = self.clamped_combat_zoom(self.ui.zoom);
    }

    pub fn set_combat_zoom(&mut self, zoom: f32) {
        self.ui.zoom = self.clamped_combat_zoom(zoom);
    }

    /// Screen regions the cut-in speed streaks must not cross: every board
    /// cell, every standing unit and any plated caption ("YOMA REACHES"),
    /// under the camera actually drawn this frame (push-in included).
    pub fn cutin_keep_out(&self) -> Vec<beats::Rect> {
        let mut out = Vec::new();
        if let Some(combat) = self.combat.as_ref() {
            let [w, h] = [self.ui.screen[0].max(1.0), self.ui.screen[1].max(1.0)];
            let (pan, zoom) = self.view_camera();
            let at = |hex: Axial| {
                crate::iso::hex_to_screen(hex, w, h, combat.cols, combat.rows, pan, zoom, 0.0, self.ui.yaw)
            };
            let cells: Vec<[f32; 2]> = combat.terrain.iter().map(|(hex, _)| at(*hex)).collect();
            let units: Vec<[f32; 2]> = combat
                .units
                .iter()
                .filter(|u| !u.dead)
                .map(|u| at(core_hex(u)))
                .collect();
            let tile = crate::iso::board_size(combat.cols, combat.rows, w, h) * zoom;
            out = beats::board_keep_out(&cells, &units, [tile / w, tile / h]);
        }
        out.extend(
            self.fx
                .floaters
                .iter()
                .filter(|f| f.plate)
                .map(|f| beats::caption_keep_out(f.plate_rect())),
        );
        out
    }

    /// Camera actually drawn this frame: the player's pan/zoom plus any claw
    /// cut-in push-in toward the clawing yoma. Returns (pan px, zoom).
    pub fn view_camera(&self) -> ([f32; 2], f32) {
        let (pan, zoom) = (self.ui.pan, self.ui.zoom);
        let (Some(combat), Some(cut)) = (self.combat.as_ref(), self.cutin.as_ref()) else {
            return (pan, zoom);
        };
        let k = cut.strength(self.fx.time);
        if k <= 0.0 {
            return (pan, zoom);
        }
        let hex_of = |id: &str| combat.units.iter().find(|u| u.id == id).map(core_hex);
        let Some(focus) = hex_of(&cut.unit_id) else {
            return (pan, zoom);
        };
        let [w, h] = [self.ui.screen[0].max(1.0), self.ui.screen[1].max(1.0)];
        let cap = crate::iso::board_zoom_max(combat.cols, combat.rows, w, h, self.ui.yaw)
            .min(COMBAT_ZOOM_MAX);
        let z1 = beats::cutin_zoom(zoom, cap, k);
        let center = crate::iso::board_viewport_center(w, h);
        let drift = beats::CUTIN_DRIFT * k;
        // Zoom about the focus, then drift it a little toward the board center.
        let push = |focus: Axial| {
            let at = |z: f32| {
                let p = crate::iso::hex_to_screen(
                    focus, w, h, combat.cols, combat.rows, pan, z, 8.0, self.ui.yaw,
                );
                [p[0] * w, p[1] * h]
            };
            let (x0, x1) = (at(zoom), at(z1));
            [
                pan[0] + (x0[0] - x1[0]) + (center[0] - x0[0]) * drift,
                pan[1] + (x0[1] - x1[1]) + (center[1] - x0[1]) * drift,
            ]
        };
        let mut p = push(focus);
        // A claw taking over a fading FLASH glides from Clare to the yoma.
        if let Some(prev) = cut.handoff.as_ref().and_then(|h| hex_of(&h.unit_id)) {
            let (a, t) = (push(prev), cut.takeover(self.fx.time));
            p = [a[0] + (p[0] - a[0]) * t, a[1] + (p[1] - a[1]) * t];
        }
        (p, z1)
    }

    pub fn clamped_combat_zoom(&self, zoom: f32) -> f32 {
        if let Some(combat) = self.combat.as_ref() {
            let [w, h] = self.ui.screen;
            clamp_board_zoom(zoom, combat.cols, combat.rows, w.max(1.0), h.max(1.0), self.ui.yaw)
        } else {
            clamp_combat_zoom(zoom)
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_combat_zoom_keeps_defaults() {
        assert!(Ui::default().zoom >= COMBAT_ZOOM_MIN);
        assert!(Ui::default().zoom <= COMBAT_ZOOM_MAX);
        assert!((clamp_combat_zoom(0.4) - COMBAT_ZOOM_MIN).abs() < 1e-6);
        assert!((clamp_combat_zoom(3.0) - COMBAT_ZOOM_MAX).abs() < 1e-6);
        assert!((clamp_combat_zoom(1.18) - 1.18).abs() < 1e-6);
        // Default Continue zoom stays legal; the old 1.55 ceiling clips the board.
        let cap = clamp_board_zoom(3.0, 11, 8, 1280.0, 800.0, 0);
        assert!(cap + 1e-3 >= 1.05);
        assert!(cap < COMBAT_ZOOM_MAX);
        assert!((clamp_board_zoom(1.05, 11, 8, 1280.0, 800.0, 3) - 1.05).abs() < 1e-6);
    }

    #[test]
    fn default_combat_view_fills_the_frame_inside_the_hud_cap() {
        for e in catalog::ENCOUNTERS.iter() {
            for (w, h) in [(1280.0, 800.0), (1920.0, 1080.0), (1024.0, 768.0), (390.0, 844.0)] {
                let (yaw, zoom) = default_combat_view(e.cols, e.rows, w, h);
                let cap = crate::iso::board_zoom_max(e.cols, e.rows, w, h, yaw).min(COMBAT_ZOOM_MAX);
                assert!(zoom <= cap + 1e-4, "{} {w}x{h}: {zoom} > cap {cap}", e.id);
                assert!(zoom >= 1.05_f32.min(cap) - 1e-4, "never tighter-than-old default");
                if w > h {
                    // Landscape: the long board axis across the screen, figures ~1.4x.
                    assert_eq!(yaw, 1, "{} {w}x{h}", e.id);
                    assert!(zoom >= 1.25, "{} {w}x{h}: {zoom}", e.id);
                } else {
                    assert_eq!(yaw, 0, "portrait keeps the tall yaw");
                }
                // Room left for the claw cut-in push-in.
                assert!(zoom < cap || cap <= 1.06 + 1e-3);
            }
        }
    }
    use crate::catalog;

    #[test]
    fn wasd_signs_on_the_map() {
        let mut g = Game::new();
        g.mode = Mode::World;
        g.world.party_x = 0.5;
        g.world.party_y = 0.5;
        g.keys = vec![winit::keyboard::KeyCode::KeyA];
        // tick() clamps dt to 0.08 — assert against that.
        g.tick(0.08);
        assert!(g.world.party_x < 0.5, "A walks left on the island");
        assert!((0.5 - g.world.party_x - WORLD_WALK_SPEED * 0.08).abs() < 1e-4);
        g.keys = vec![winit::keyboard::KeyCode::KeyD];
        let x = g.world.party_x;
        g.tick(0.08);
        assert!(g.world.party_x > x, "D walks right on the island");
        g.keys = vec![winit::keyboard::KeyCode::KeyW];
        let y = g.world.party_y;
        g.tick(0.08);
        assert!(g.world.party_y < y, "W walks up the painted map");
        assert_eq!(g.facing_y, -1.0, "north updates vertical facing");
    }

    #[test]
    fn walking_stamps_explored_cells() {
        let mut g = Game::new();
        g.mode = Mode::World;
        g.world.explored = world::empty_explored();
        g.world.party_x = 0.70;
        g.world.party_y = 0.30;
        g.keys = vec![winit::keyboard::KeyCode::KeyD];
        g.tick(0.05);
        assert!(world::map_explored_at(&g.world, g.world.party_x, g.world.party_y));
    }

    #[test]
    fn combat_esc_needs_confirm_then_persists_cleared() {
        let mut g = Game::new();
        g.world = world::new_world();
        let enc = catalog::encounter("doga-yoma").expect("doga-yoma");
        g.begin_battle(enc);
        assert_eq!(g.mode, Mode::Combat);
        assert!(g.combat.is_some());
        g.key(winit::keyboard::KeyCode::Escape, true);
        g.key(winit::keyboard::KeyCode::Escape, false);
        assert_eq!(g.mode, Mode::Combat, "first Esc only arms flee");
        assert!(g.esc_arm > 0.0);
        assert!(g.combat.is_some());
        g.key(winit::keyboard::KeyCode::Escape, true);
        assert_eq!(g.mode, Mode::World);
        assert!(g.combat.is_none());
        let p = save::load_save().expect("persist after flee");
        assert!(p.combat.is_none(), "save must not restore abandoned fight");
        assert_ne!(p.mode, Mode::Combat);
    }

    #[test]
    fn world_esc_persists_before_title() {
        let mut g = Game::new();
        g.mode = Mode::World;
        g.world = world::new_world();
        g.world.party_x = 0.71;
        g.world.party_y = 0.33;
        g.key(winit::keyboard::KeyCode::Escape, true);
        assert_eq!(g.mode, Mode::Title);
        let p = save::load_save().expect("persist on world Esc");
        assert!((p.world.party_x - 0.71).abs() < 1e-4);
        assert!((p.world.party_y - 0.33).abs() < 1e-4);
    }

    #[test]
    fn title_esc_arms_quit_confirm() {
        let mut g = Game::new();
        assert_eq!(g.mode, Mode::Title);
        g.key(winit::keyboard::KeyCode::Escape, true);
        g.key(winit::keyboard::KeyCode::Escape, false);
        assert_eq!(g.mode, Mode::Title, "first Esc only arms quit");
        assert!(g.esc_arm > 0.0);
        assert!(g.fx.floaters.iter().any(|f| f.text.contains("QUIT") && f.plate));
    }

    #[test]
    fn town_esc_returns_to_world_not_title() {
        let mut g = Game::new();
        g.mode = Mode::Town;
        g.world = world::new_world();
        g.key(winit::keyboard::KeyCode::Escape, true);
        assert_eq!(g.mode, Mode::World);
    }

    #[test]
    fn digit3_selects_slot_skill_flash_for_clare() {
        let mut g = Game::new();
        g.world = world::new_world();
        let enc = catalog::encounter("doga-yoma").expect("doga-yoma");
        g.begin_battle(enc);
        assert_eq!(g.mode, Mode::Combat);
        let slot = g
            .combat
            .as_ref()
            .and_then(|c| c.units.iter().find(|u| u.id == "clare"))
            .and_then(|u| u.skills.get(4).map(|s| s.as_str()));
        assert_eq!(slot, Some("flash"), "Clare skill[4] is the bar.slot chip");
        g.key(winit::keyboard::KeyCode::Digit3, true);
        assert_eq!(
            g.ui.selected_skill.as_deref(),
            Some("flash"),
            "Digit3 must pick_skill_slot(4), not hardcode aimed"
        );
    }

    #[test]
    fn flash_clip_and_outcome_floaters_on_resolve() {
        use crate::combat::{core_hex, legal_targets, PlayerAction, Side};
        use crate::hex::hex_eq;
        use crate::fx::FightClip;

        let mut g = Game::new();
        g.world = world::new_world();
        let enc = catalog::encounter("doga-yoma").expect("doga-yoma");
        g.begin_battle(enc);
        // Force Clare's turn and park her adjacent to a yoma.
        let combat = g.combat.as_mut().unwrap();
        let clare_turn = combat
            .order
            .iter()
            .position(|id| id == "clare")
            .expect("clare");
        combat.turn = clare_turn;
        let prey = combat
            .units
            .iter()
            .find(|u| u.side == Side::Enemy && !u.dead)
            .cloned()
            .expect("yoma");
        let prey_hex = core_hex(&prey);
        if let Some(c) = combat.units.iter_mut().find(|u| u.id == "clare") {
            let n = crate::hex::hex_neighbors(prey_hex)[0];
            c.origin = n;
            c.ap = 3;
            c.trans = 50; // Flash needs 40 trans
            c.yoki = 20;
        }
        let targets = legal_targets(g.combat.as_ref().unwrap(), "clare", "flash");
        assert!(
            targets.iter().any(|h| hex_eq(*h, prey_hex)),
            "flash should reach adjacent prey"
        );
        g.combat_act(PlayerAction::Skill {
            id: "flash".into(),
            hex: prey_hex,
        });
        let clip = g.fx.clip_of("clare").0;
        // AI may overwrite with Hurt after Clare's resolve; never reuse Cut Slash.
        assert_ne!(clip, FightClip::Slash, "Flash must not reuse the Cut slash clip");
        assert!(
            matches!(clip, FightClip::Flash | FightClip::Hurt | FightClip::Idle | FightClip::Guard),
            "unexpected post-flash clip {clip:?}"
        );
        // Direct play path: juice arms Flash for the skill id.
        let mut fx = crate::fx::Fx::default();
        fx.play_clip("clare", FightClip::Flash);
        assert_eq!(fx.clip_of("clare").0, FightClip::Flash);
        assert!(
            g.fx.flash > 0.0 || !g.fx.bursts.is_empty(),
            "Flash should leave a silver burst"
        );
        let words = ["MISS", "GLANCE", "BLOCKED", "SOLID"];
        let hit_readout = g.fx.floaters.iter().any(|f| {
            f.plate && words.iter().any(|w| f.text.starts_with(w))
        });
        // Cut path for comparison: force a resolve that logs an outcome.
        // Flash resolve above should already have produced miss/glance/blocked/solid
        // (or empty-air is unlikely when adjacent). If RNG missed every strike cell,
        // still accept MISS.
        assert!(
            hit_readout,
            "expected plated MISS/GLANCE/BLOCKED/SOLID floater, got {:?}",
            g.fx.floaters.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
    }

    #[test]
    fn cut_resolve_emits_plated_outcome_word() {
        use crate::combat::{core_hex, legal_targets, PlayerAction, Side};
        use crate::hex::hex_eq;

        let mut g = Game::new();
        g.world = world::new_world();
        let enc = catalog::encounter("doga-yoma").expect("doga-yoma");
        g.begin_battle(enc);
        let combat = g.combat.as_mut().unwrap();
        let clare_turn = combat.order.iter().position(|id| id == "clare").unwrap();
        combat.turn = clare_turn;
        let prey = combat
            .units
            .iter()
            .find(|u| u.side == Side::Enemy && !u.dead)
            .cloned()
            .unwrap();
        let prey_hex = core_hex(&prey);
        if let Some(c) = combat.units.iter_mut().find(|u| u.id == "clare") {
            c.origin = crate::hex::hex_neighbors(prey_hex)[0];
            c.ap = 2;
        }
        assert!(legal_targets(g.combat.as_ref().unwrap(), "clare", "cut")
            .iter()
            .any(|h| hex_eq(*h, prey_hex)));
        g.combat_act(PlayerAction::Skill {
            id: "cut".into(),
            hex: prey_hex,
        });
        let floater = g.fx.floaters.iter().find(|f| {
            f.plate
                && (f.text.starts_with("MISS")
                    || f.text.starts_with("GLANCE")
                    || f.text.starts_with("BLOCKED")
                    || f.text.starts_with("SOLID"))
        });
        assert!(
            floater.is_some(),
            "Cut resolve must plate an outcome word, got {:?}",
            g.fx.floaters.iter().map(|f| &f.text).collect::<Vec<_>>()
        );
        // No raw HP-only soup digits without a word.
        assert!(
            g.fx.floaters.iter().all(|f| {
                !f.text.chars().all(|c| c.is_ascii_digit())
            }),
            "raw digit floaters are HP soup"
        );
    }
}
