//! Combat UI actions, juice, and result handoff.

use super::*;
use crate::audio;
use crate::catalog;
use crate::combat::{
    act, ai_beat, core_hex, current_unit, legal_moves, legal_targets, peek_enemy_tell, zone_for,
    AiBeat, IntendedStrike, PlayerAction, Side,
};
use super::beats;
use crate::dialog::{self, SceneId, SceneState};
use crate::hud;
use crate::world::apply_victory;
use crate::hex::{hex_eq, Axial};


impl Game {
    pub(super) fn click_combat(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
        let bar = hud::combat_bar();
        let open = self.combat_input_open();
        if ny > bar.wait.y {
            if bar.forfeit.contains(nx, ny) {
                // FALL is a retreat, not a turn action: it stays live during
                // enemy beats, like Esc-Esc flee.
                audio::error();
                self.finish_combat(false);
            } else if !open {
                // Turn actions are dimmed while the yoma act.
                if bar.wait.contains(nx, ny)
                    || bar.raise.contains(nx, ny)
                    || bar.guard.contains(nx, ny)
                    || bar.cut.contains(nx, ny)
                    || bar.slot.contains(nx, ny)
                {
                    audio::error();
                }
            } else if bar.wait.contains(nx, ny) {
                audio::click();
                self.combat_act(PlayerAction::Wait);
            } else if bar.raise.contains(nx, ny) {
                self.combat_act(PlayerAction::Raise);
            } else if bar.guard.contains(nx, ny) {
                self.select_skill("guard");
            } else if bar.cut.contains(nx, ny) {
                self.select_skill("cut");
            } else if bar.slot.contains(nx, ny) {
                audio::click();
                self.pick_skill_slot(4);
            }
            return;
        }
        if !open {
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

    pub(super) fn pick_skill_slot(&mut self, nth: usize) {
        if !self.combat_input_open() {
            return;
        }
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

    /// True when the combat bar accepts turn actions (WAIT / RAISE / GUARD /
    /// CUT / skill slot and their keys). False during enemy beats and claw
    /// windups; the HUD dims the same buttons from this predicate.
    pub fn combat_input_open(&self) -> bool {
        self.mode == Mode::Combat
            && combat_buttons_enabled(self.combat.as_ref(), self.enemy_tell.is_some())
    }

    /// Select a skill from the bar or a hotkey (no-op while enemies act).
    pub(super) fn select_skill(&mut self, id: &str) {
        if !self.combat_input_open() {
            return;
        }
        audio::click();
        self.ui.selected_skill = Some(id.into());
    }

    pub fn combat_act(&mut self, action: PlayerAction) {
        if !self.combat_input_open() {
            return;
        }
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

        {
            let Some(combat) = self.combat.as_mut() else {
                return;
            };
            act(combat, action.clone(), true);
        }
        let landed = self.combat.as_ref().map(|c| c.log.len() > log_len).unwrap_or(false);
        let flash_by = actor.as_ref().map(|a| a.0.clone());
        self.juice_from_act(&before, actor, &action, log_len);
        if landed {
            self.arm_flash_cutin(&action, flash_by);
        }
        // Enemies take one action per beat from here (see `tick_enemy_tell`).
        self.schedule_enemy_beat();
        if !self.combat_input_open() {
            // A half-picked skill must not linger over the enemy phase.
            self.ui.selected_skill = None;
        }
        if let Some(win) = self.combat.as_ref().and_then(|c| c.over) {
            let id = self.combat.as_ref().unwrap().id.clone();
            self.finish_combat_id(win, &id);
        } else {
            self.persist();
        }
    }

    /// Blood-tint hexes the next yoma will claw (pending windup or player-turn peek).
    pub fn threat_zone(&self) -> Vec<Axial> {
        let Some(combat) = self.combat.as_ref() else {
            return vec![];
        };
        let strike = if let Some(tell) = self.enemy_tell.as_ref() {
            Some(IntendedStrike {
                unit_id: tell.unit_id.clone(),
                skill_id: tell.skill_id.clone(),
                hex: tell.hex,
            })
        } else if current_unit(combat)
            .map(|u| u.side == Side::Player)
            .unwrap_or(false)
        {
            peek_enemy_tell(combat)
        } else {
            None
        };
        let Some(strike) = strike else {
            return vec![];
        };
        let Some(u) = combat.units.iter().find(|u| u.id == strike.unit_id) else {
            return vec![];
        };
        let Some(skill) = catalog::skill(&strike.skill_id) else {
            return vec![];
        };
        zone_for(combat, u, skill, strike.hex)
    }

    /// FLASH (Quicksword) cut-in: presentational only. The strike already
    /// resolved, input stays open, and beat timing is untouched.
    fn arm_flash_cutin(&mut self, action: &PlayerAction, striker: Option<String>) {
        let (PlayerAction::Skill { id, .. }, Some(unit_id)) = (action, striker) else {
            return;
        };
        if id != "flash" {
            return;
        }
        self.cutin = Some(beats::flash_cutin(&unit_id, self.fx.time));
        self.fx.emit_hint(0.50, 0.72, beats::FLASH_CAPTION);
    }

    pub(super) fn arm_enemy_tell(&mut self, strike: IntendedStrike) {
        self.enemy_tell = Some(super::EnemyTell {
            unit_id: strike.unit_id.clone(),
            skill_id: strike.skill_id.clone(),
            hex: strike.hex,
            resolve_at: self.fx.time + beats::ENEMY_WINDUP,
        });
        self.enemy_beat_at = None;
        self.cutin = Some(beats::rearm_cutin(
            self.cutin.as_ref(),
            &strike.unit_id,
            self.fx.time,
            beats::ENEMY_WINDUP,
        ));
        self.fx.play_clip(&strike.unit_id, crate::fx::FightClip::Slash);
        if !self.enemy_tell_announced {
            self.fx.emit_hint(0.50, 0.72, "YOMA REACHES");
            self.enemy_tell_announced = true;
        }
    }

    fn enemy_turn(&self) -> bool {
        self.combat
            .as_ref()
            .filter(|c| c.over.is_none())
            .and_then(current_unit)
            .map(|u| u.side == Side::Enemy)
            .unwrap_or(false)
    }

    /// Queue the next paced enemy action if an enemy owns the turn; otherwise
    /// close the enemy phase.
    pub(super) fn schedule_enemy_beat(&mut self) {
        if self.enemy_tell.is_some() {
            return;
        }
        if self.enemy_turn() {
            if self.enemy_beat_at.is_none() {
                self.enemy_beat_at = Some(beats::next_beat_at(self.fx.time));
            }
        } else {
            self.enemy_beat_at = None;
            self.enemy_phase_beats = 0;
        }
    }

    /// Fire one enemy beat: exactly one Raise / Move / Wait, or arm a claw windup.
    fn run_enemy_beat(&mut self) {
        self.enemy_beat_at = None;
        let Some(combat) = self.combat.as_ref() else {
            return;
        };
        let before: Vec<(String, i32, bool, Axial, i32, i32)> = combat
            .units
            .iter()
            .map(|u| (u.id.clone(), u.hp, u.dead, u.origin, u.trans, u.yoki))
            .collect();
        let actor = current_unit(combat)
            .filter(|u| !u.dead)
            .map(|u| (u.id.clone(), core_hex(u)));
        let log_len = combat.log.len();
        self.enemy_phase_beats += 1;
        let beat = if self.enemy_phase_beats > beats::ENEMY_PHASE_MAX_BEATS {
            // Runaway guard: pass the turn rather than stall the fight.
            let combat = self.combat.as_mut().unwrap();
            act(combat, PlayerAction::Wait, false);
            AiBeat::Acted(PlayerAction::Wait)
        } else {
            ai_beat(self.combat.as_mut().unwrap())
        };
        match beat {
            AiBeat::Acted(action) if actor.is_some() => {
                self.juice_from_act(&before, actor, &action, log_len);
            }
            AiBeat::Acted(_) => {}
            AiBeat::Strike(strike) => {
                self.arm_enemy_tell(strike);
            }
            AiBeat::Idle => {}
        }
        if let Some(win) = self.combat.as_ref().and_then(|c| c.over) {
            self.finish_combat(win);
            return;
        }
        self.schedule_enemy_beat();
        self.persist();
    }

    pub(super) fn tick_enemy_tell(&mut self) {
        // Cut-in outlives the windup by its ease-out, then retires.
        if self
            .cutin
            .as_ref()
            .map(|c| self.fx.time >= c.end())
            .unwrap_or(false)
        {
            self.cutin = None;
        }
        // Soft stretch while the player can still Guard against a peeked claw.
        if self.enemy_tell.is_none() {
            // Saves contain combat state, not presentation timers. Resume an
            // enemy phase after Continue (or after discarding a stale tell).
            if self.enemy_turn() {
                if self.enemy_beat_at.is_none() {
                    self.schedule_enemy_beat();
                }
                if beats::beat_due(self.fx.time, self.enemy_beat_at) {
                    self.run_enemy_beat();
                }
                return;
            }
            self.enemy_beat_at = None;
            self.enemy_phase_beats = 0;
            if let Some(combat) = self.combat.as_ref() {
                if current_unit(combat)
                    .map(|u| u.side == Side::Player)
                    .unwrap_or(false)
                {
                    if let Some(strike) = peek_enemy_tell(combat) {
                        let id = strike.unit_id.clone();
                        if self.fx.clip_of(&id).0 == crate::fx::FightClip::Idle {
                            self.fx.play_clip(&id, crate::fx::FightClip::Slash);
                        }
                        if !self.enemy_tell_announced {
                            self.fx.emit_hint(0.50, 0.72, "YOMA REACHES");
                            self.enemy_tell_announced = true;
                        }
                    }
                }
            }
            return;
        }
        let unit_id = self.enemy_tell.as_ref().unwrap().unit_id.clone();
        if self.fx.clip_of(&unit_id).0 == crate::fx::FightClip::Idle {
            self.fx.play_clip(&unit_id, crate::fx::FightClip::Slash);
        }
        let ready = self
            .enemy_tell
            .as_ref()
            .map(|t| self.fx.time >= t.resolve_at)
            .unwrap_or(false);
        if !ready {
            return;
        }
        let Some(tell) = self.enemy_tell.take() else {
            return;
        };
        self.resolve_enemy_tell(tell);
    }

    fn resolve_enemy_tell(&mut self, tell: super::EnemyTell) {
        let Some(combat) = self.combat.as_ref() else {
            return;
        };
        // The AI entry point bypasses the player-side gate. Never apply a
        // presentation callback to a different actor after a state change.
        if !current_unit(combat)
            .map(|u| u.side == Side::Enemy && u.id == tell.unit_id)
            .unwrap_or(false)
        {
            // Stale tell: ease any cut-in out from now instead of snapping.
            if let Some(cut) = self.cutin.as_mut() {
                cut.strike_at = cut.strike_at.min(self.fx.time);
            }
            return;
        }
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
        let actor_hex = combat
            .units
            .iter()
            .find(|u| u.id == tell.unit_id)
            .map(|u| core_hex(u));
        let actor = actor_hex.map(|h| (tell.unit_id.clone(), h));
        let log_len = combat.log.len();
        let action = PlayerAction::Skill {
            id: tell.skill_id.clone(),
            hex: tell.hex,
        };
        {
            let Some(combat) = self.combat.as_mut() else {
                return;
            };
            act(combat, action.clone(), false);
        }
        self.juice_from_act(&before, actor, &action, log_len);
        // The claw landed: the next enemy action (if any) waits one beat.
        self.schedule_enemy_beat();
        if let Some(win) = self.combat.as_ref().and_then(|c| c.over) {
            let id = self.combat.as_ref().unwrap().id.clone();
            self.finish_combat_id(win, &id);
        } else {
            self.persist();
        }
    }

    pub(super) fn juice_from_act(
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
        if let Some((id, _)) = actor.as_ref() {
            match action {
                PlayerAction::Move(_) => self.fx.play_clip(id, crate::fx::FightClip::Lunge),
                PlayerAction::Raise => self.fx.play_clip(id, crate::fx::FightClip::Raise),
                PlayerAction::Wait => self.fx.play_clip(id, crate::fx::FightClip::Wait),
                PlayerAction::Skill { id: skill, .. } if skill == "guard" => {
                    self.fx.play_clip(id, crate::fx::FightClip::Guard)
                }
                PlayerAction::Skill { id: skill, .. } if skill == "flash" => {
                    self.fx.play_clip(id, crate::fx::FightClip::Flash)
                }
                PlayerAction::Skill { .. } => self.fx.play_clip(id, crate::fx::FightClip::Slash),
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
                    } else if id == "flash" {
                        self.fx.emit_flash_burst(q[0], q[1]);
                    } else {
                        self.fx.emit_hit(q[0], q[1], 0, "windup");
                    }
                }
                PlayerAction::Wait => {}
            }
        }
        #[derive(Clone)]
        struct Ev {
            id: String,
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
                            id: u.id.clone(),
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
                self.fx.play_clip(&ev.id, crate::fx::FightClip::Hurt);
            }
            if ev.heal > 0 {
                self.fx.emit_heal(p[0], p[1], ev.heal);
            }
            if ev.trans_up {
                self.fx.emit_raise(p[0], p[1]);
            }
        }
        let kinds: Vec<(String, String)> = self
            .combat
            .as_ref()
            .map(|c| {
                let n = c.log.len().saturating_sub(log_len).min(8);
                c.log
                    .iter()
                    .rev()
                    .take(n)
                    .map(|l| (l.kind.clone(), l.text.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for (kind, text) in kinds.into_iter().rev() {
            let dmg = parse_dealt(&text);
            let def_hex = self
                .combat
                .as_ref()
                .and_then(|c| find_defender_hex(c, &text));
            let pos = def_hex
                .map(|h| self.hex_screen(h))
                .unwrap_or([0.50, 0.42]);
            match kind.as_str() {
                "miss" => {
                    audio::play("miss");
                    self.fx.emit_hit(pos[0], pos[1], 0, "miss");
                    self.fx.emit_outcome(pos[0], pos[1] - 0.02, "miss", 0);
                }
                "glance" => {
                    self.fx.emit_outcome(pos[0], pos[1] - 0.02, "glance", dmg);
                }
                "blocked" => {
                    audio::play("block");
                    self.fx.emit_outcome(pos[0], pos[1] - 0.02, "blocked", dmg);
                }
                "solid" => {
                    self.fx.emit_outcome(pos[0], pos[1] - 0.02, "solid", dmg);
                }
                "sever" => audio::play("chop"),
                _ => {}
            }
        }
        // One plated floater when the skill-slot gate first opens (Raise→Flash payoff).
        if let Some(combat) = self.combat.as_ref() {
            for u in &combat.units {
                let Some(prev) = before.iter().find(|b| b.0 == u.id) else {
                    continue;
                };
                let Some(slot_id) = u.skills.get(4) else {
                    continue;
                };
                let Some(skill) = catalog::skill(slot_id) else {
                    continue;
                };
                if prev.4 < skill.trans && u.trans >= skill.trans {
                    let label = format!("{} READY", slot_id.to_ascii_uppercase());
                    self.fx.emit_hint(0.58, 0.78, label);
                    break;
                }
            }
        }
    }

    pub(super) fn finish_combat(&mut self, win: bool) {
        let id = self
            .combat
            .as_ref()
            .map(|c| c.id.clone())
            .unwrap_or_default();
        self.finish_combat_id(win, &id);
    }

    pub(super) fn finish_combat_id(&mut self, win: bool, id: &str) {
        if win {
            apply_victory(&mut self.world, id);
            self.result_title = dialog::RESULT_WIN_TITLE.into();
            self.result_body = dialog::RESULT_WIN_BODY.into();
            self.fx.emit_win();
            audio::confirm();
            let raki_fell = self
                .combat
                .as_ref()
                .and_then(|c| c.units.iter().find(|u| u.template_id == "raki"))
                .map(|u| u.dead)
                .unwrap_or(false);
            if raki_fell {
                self.world.flags.insert("raki-dead".into(), true);
            }
            // Nest path already told the late story: the boy is gone.
            self.scene = match id {
                "doga-yoma" if !self.world.raki && !raki_fell => {
                    Some(SceneState::new(SceneId::RakiJoin))
                }
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
            audio::music_defeat();
            self.scene = None;
        }
        self.result_win = Some(win);
        self.mode = Mode::Result;
        self.combat = None;
        self.reset_enemy_pacing();
        self.persist();
    }
}


fn parse_dealt(text: &str) -> i32 {
    text.rsplit_once(" for ")
        .and_then(|(_, rest)| rest.trim().trim_end_matches('.').parse().ok())
        .unwrap_or(0)
}

fn find_defender_hex(combat: &crate::combat::CombatState, text: &str) -> Option<Axial> {
    let mut best: Option<(usize, Axial)> = None;
    for u in &combat.units {
        if let Some(pos) = text.find(&u.name) {
            if pos == 0 {
                continue; // leading attacker name
            }
            let len = u.name.len();
            if best.map(|(l, _)| len > l).unwrap_or(true) {
                best = Some((len, core_hex(u)));
            }
        }
    }
    best.map(|(_, hex)| hex)
}

/// Combat-bar turn actions are live only on a player's turn in a running
/// fight with no claw windup armed. Pure so the HUD and input share it.
pub fn combat_buttons_enabled(combat: Option<&crate::combat::CombatState>, tell_armed: bool) -> bool {
    let Some(combat) = combat else {
        return false;
    };
    combat.over.is_none()
        && !tell_armed
        && current_unit(combat).map(|u| u.side == Side::Player).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{create_battle, Terrain};

    fn adjacent_yoma(raki: bool) -> Game {
        let mut game = Game::new();
        game.world = world::new_world();
        game.world.raki = raki;
        game.mode = Mode::Combat;
        game.enemy_tell = None;
        let mut combat = create_battle(
            catalog::encounter("paburo-nest").unwrap(),
            &["clare".into()],
            7,
        );
        combat.units.retain(|u| u.id == "clare" || u.id == "yoma-0");
        combat.order = vec!["clare".into(), "yoma-0".into()];
        combat.turn = 0;
        combat.support_raki = raki;
        for (_, terrain) in &mut combat.terrain {
            *terrain = Terrain::Grass;
        }
        for unit in &mut combat.units {
            unit.origin = if unit.side == Side::Player {
                Axial::new(3, 3)
            } else {
                Axial::new(4, 3)
            };
            unit.ap = 1;
            unit.max_ap = 1;
        }
        game.combat = Some(combat);
        game
    }

    /// Jump to the scheduled enemy beat and fire it.
    fn fire_beat(game: &mut Game) {
        game.fx.time = game.enemy_beat_at.expect("enemy beat scheduled");
        game.tick_enemy_tell();
    }

    fn waiting_for_yoma(raki: bool) -> Game {
        let mut game = adjacent_yoma(raki);
        game.combat_act(PlayerAction::Wait);
        assert_eq!(
            current_unit(game.combat.as_ref().unwrap()).unwrap().id,
            "yoma-0"
        );
        // The claw is not armed in the player's frame: the yoma gets a beat first.
        assert!(game.enemy_tell.is_none());
        fire_beat(&mut game);
        assert_eq!(game.enemy_tell.as_ref().unwrap().skill_id, "claw");
        game
    }

    fn resolve_pending(game: &mut Game) {
        game.fx.time = game.enemy_tell.as_ref().unwrap().resolve_at;
        game.tick_enemy_tell();
    }

    #[test]
    fn enemy_tell_resolves_with_or_without_recruited_raki() {
        for raki in [false, true] {
            let mut game = waiting_for_yoma(raki);
            resolve_pending(&mut game);
            let combat = game.combat.as_ref().unwrap();
            assert_eq!(
                current_unit(combat).unwrap().side,
                Side::Player,
                "enemy must spend its AP and return the turn; Raki={raki}"
            );
            assert!(game.enemy_tell.is_none());
            assert!(combat.log.iter().any(|entry| entry.text == "Yoma uses Rend."));
        }
    }

    #[test]
    fn player_input_cannot_control_or_postpone_enemy_turn() {
        for raki in [false, true] {
            for action in [PlayerAction::Wait, PlayerAction::Raise] {
                let mut game = waiting_for_yoma(raki);
                let before = serde_json::to_string(game.combat.as_ref().unwrap()).unwrap();
                let deadline = game.enemy_tell.as_ref().unwrap().resolve_at;
                game.fx.time += 0.2;
                game.combat_act(action);
                assert!(
                    serde_json::to_string(game.combat.as_ref().unwrap()).unwrap() == before,
                    "player input changed an enemy turn; Raki={raki}"
                );
                assert_eq!(game.enemy_tell.as_ref().unwrap().resolve_at, deadline);
                resolve_pending(&mut game);
                assert_eq!(
                    current_unit(game.combat.as_ref().unwrap()).unwrap().side,
                    Side::Player
                );
            }
        }
    }

    #[test]
    fn continue_resumes_enemy_turn_and_discards_old_tell() {
        for raki in [false, true] {
            for stale_tell in [false, true] {
                let original = waiting_for_yoma(raki);
                original.persist();
                let mut loaded = Game::new();
                if stale_tell {
                    loaded.enemy_tell = Some(EnemyTell {
                        unit_id: "old-battle-enemy".into(),
                        skill_id: "claw".into(),
                        hex: Axial::new(0, 0),
                        resolve_at: 1000.0,
                    });
                    loaded.enemy_tell_announced = true;
                }
                loaded.continue_hunt();
                assert_eq!(loaded.mode, Mode::Combat);
                assert!(loaded.enemy_tell.is_none());
                assert!(!loaded.enemy_tell_announced);
                loaded.tick_enemy_tell();
                fire_beat(&mut loaded);
                assert_eq!(loaded.enemy_tell.as_ref().unwrap().unit_id, "yoma-0");
                resolve_pending(&mut loaded);
                assert_eq!(
                    current_unit(loaded.combat.as_ref().unwrap()).unwrap().side,
                    Side::Player
                );
            }
        }
    }

    #[test]
    fn enemy_turn_without_tell_can_move_and_end() {
        let mut game = adjacent_yoma(true);
        let combat = game.combat.as_mut().unwrap();
        combat.turn = 1;
        combat.units.iter_mut().find(|u| u.id == "yoma-0").unwrap().origin =
            Axial::new(8, 3);
        game.tick_enemy_tell();
        for _ in 0..8 {
            if game.enemy_beat_at.is_none() {
                break;
            }
            fire_beat(&mut game);
        }
        assert!(game.enemy_tell.is_none());
        let combat = game.combat.as_ref().unwrap();
        assert_eq!(current_unit(combat).unwrap().side, Side::Player);
        assert!(combat.log.iter().any(|entry| entry.kind == "move"));
    }

    #[test]
    fn stale_tell_never_executes_for_another_actor() {
        for enemy_turn in [false, true] {
            let mut game = adjacent_yoma(true);
            if enemy_turn {
                game.combat.as_mut().unwrap().turn = 1;
            }
            let before = serde_json::to_string(game.combat.as_ref().unwrap()).unwrap();
            game.enemy_tell = Some(EnemyTell {
                unit_id: "old-battle-enemy".into(),
                skill_id: "claw".into(),
                hex: Axial::new(4, 3),
                resolve_at: game.fx.time,
            });
            game.tick_enemy_tell();
            assert!(game.enemy_tell.is_none());
            assert!(serde_json::to_string(game.combat.as_ref().unwrap()).unwrap() == before);
            if enemy_turn {
                game.tick_enemy_tell();
                fire_beat(&mut game);
                assert_eq!(game.enemy_tell.as_ref().unwrap().unit_id, "yoma-0");
            }
        }
    }

    /// Clare plus two far yoma, 2 AP each (two paced steps apiece).
    fn far_pack() -> Game {
        let mut game = adjacent_yoma(true);
        let mut combat = create_battle(
            catalog::encounter("paburo-nest").unwrap(),
            &["clare".into()],
            7,
        );
        combat.units.retain(|u| u.id == "clare" || u.id == "yoma-0" || u.id == "yoma-1");
        combat.order = vec!["clare".into(), "yoma-0".into(), "yoma-1".into()];
        combat.turn = 0;
        combat.support_raki = true;
        for (_, terrain) in &mut combat.terrain {
            *terrain = Terrain::Grass;
        }
        for unit in &mut combat.units {
            unit.origin = match unit.id.as_str() {
                "clare" => Axial::new(0, 3),
                "yoma-0" => Axial::new(8, 1),
                _ => Axial::new(8, 5),
            };
            unit.ap = 2;
            unit.max_ap = 2;
        }
        game.combat = Some(combat);
        game
    }

    fn snapshot(game: &Game) -> (usize, usize, Vec<Axial>) {
        let c = game.combat.as_ref().unwrap();
        (c.turn, c.log.len(), c.units.iter().map(|u| u.origin).collect())
    }

    #[test]
    fn enemy_phase_plays_one_action_per_beat_and_queue_advances() {
        use crate::gpu::queue_ids;
        let mut game = far_pack();
        let t0 = game.fx.time;
        game.combat_act(PlayerAction::Wait);
        // Player's Wait handed the turn over, but no enemy acted in that frame.
        assert_eq!(current_unit(game.combat.as_ref().unwrap()).unwrap().id, "yoma-0");
        assert!(!game.combat.as_ref().unwrap().log.iter().any(|l| l.text.starts_with("Yoma")));
        assert!((game.enemy_beat_at.unwrap() - (t0 + ENEMY_BEAT)).abs() < 1e-5);

        // Before the beat: ticking changes nothing.
        let before = snapshot(&game);
        game.fx.time = t0 + ENEMY_BEAT * 0.9;
        game.tick_enemy_tell();
        assert_eq!(snapshot(&game), before);

        let mut queue_heads = vec![];
        let mut beat_times = vec![];
        let mut prev = snapshot(&game);
        for _ in 0..12 {
            let Some(at) = game.enemy_beat_at else { break };
            beat_times.push(at);
            fire_beat(&mut game);
            let now = snapshot(&game);
            // Exactly one action per beat: at most one new log line, at most one
            // unit moved, turn index advances by at most one slot.
            assert!(now.1 - prev.1 <= 1, "beat applied several actions");
            let moved = now.2.iter().zip(&prev.2).filter(|(a, b)| a != b).count();
            assert!(moved <= 1);
            assert!(now.0 == prev.0 || now.0 == (prev.0 + 1) % 3);
            let c = game.combat.as_ref().unwrap();
            queue_heads.push(queue_ids(&c.order, c.turn, &c.units)[0].clone());
            prev = now;
        }
        // Two paced steps per yoma (2 AP), then Clare: the head chip holds on
        // each yoma for its beats and advances in initiative order.
        assert_eq!(queue_heads, ["yoma-0", "yoma-1", "yoma-1", "clare"]);
        for pair in beat_times.windows(2) {
            assert!((pair[1] - pair[0] - ENEMY_BEAT).abs() < 1e-5, "{beat_times:?}");
        }
        assert!(game.enemy_beat_at.is_none());
        assert_eq!(game.enemy_phase_beats, 0, "phase counter resets on the party turn");
    }

    #[test]
    fn player_input_is_ignored_between_enemy_beats() {
        let mut game = far_pack();
        game.combat_act(PlayerAction::Wait);
        fire_beat(&mut game);
        let before = serde_json::to_string(game.combat.as_ref().unwrap()).unwrap();
        let at = game.enemy_beat_at;
        for action in [PlayerAction::Wait, PlayerAction::Raise, PlayerAction::Move(Axial::new(1, 3))] {
            game.combat_act(action);
        }
        assert_eq!(serde_json::to_string(game.combat.as_ref().unwrap()).unwrap(), before);
        assert_eq!(game.enemy_beat_at, at, "input must not reschedule the beat");
    }

    #[test]
    fn continue_mid_enemy_phase_resumes_on_the_next_beat() {
        let mut game = far_pack();
        game.combat_act(PlayerAction::Wait);
        fire_beat(&mut game); // yoma-0 moves; persisted
        let saved = snapshot(&game);
        let mut loaded = Game::new();
        loaded.continue_hunt();
        assert_eq!(snapshot(&loaded), saved);
        assert!(loaded.enemy_beat_at.is_none());
        loaded.tick_enemy_tell();
        assert!(loaded.enemy_beat_at.is_some(), "Continue rebuilds the pending beat");
        for _ in 0..8 {
            if loaded.enemy_beat_at.is_none() {
                break;
            }
            fire_beat(&mut loaded);
        }
        assert_eq!(current_unit(loaded.combat.as_ref().unwrap()).unwrap().id, "clare");
    }

    /// Streaks of the live cut-in stay off every hex top and the caption plate.
    fn assert_streaks_clear(mut game: Game, caption_text: &str, yaw: u8) {
        let cut = game.cutin.clone().unwrap();
        let caption = game
            .fx
            .floaters
            .iter()
            .find(|f| f.plate && f.text == caption_text)
            .expect("cut-in shows its caption")
            .plate_rect();
        let combat = game.combat.clone().unwrap();
        let [w, h] = game.ui.screen;
        let (rin, _) = cut.kind.ramps();
        let mut seen = 0;
        for i in 0..24 {
            game.fx.time = cut.start + rin + i as f32 * 0.021;
            let k = cut.strength(game.fx.time);
            let (pan, zoom) = game.view_camera();
            let tile = crate::iso::board_size(combat.cols, combat.rows, w, h) * zoom;
            let focus = game.hex_screen(core_hex(
                combat.units.iter().find(|u| u.id == cut.unit_id).unwrap(),
            ));
            let keep = game.cutin_keep_out();
            for l in super::beats::speed_lines(k, game.fx.time, focus, w / h, &keep) {
                seen += 1;
                let (y0, y1) = (l[1], l[1] + l[3]);
                // Caption plate (independent of the keep-out list).
                let cap_hit = l[0] < caption[0] + caption[2]
                    && l[0] + l[2] > caption[0]
                    && y0 < caption[1] + caption[3]
                    && y1 > caption[1];
                assert!(!cap_hit, "yaw {yaw}: streak {l:?} crosses caption {caption:?}");
                // Every hex top: no streak within a hex radius of a centre.
                for (hex, _) in &combat.terrain {
                    let c = crate::iso::hex_to_screen(
                        *hex, w, h, combat.cols, combat.rows, pan, zoom, 0.0, yaw,
                    );
                    let (rx, ry) = (tile * 0.9 / w, tile * 0.5 / h);
                    let hit = l[0] < c[0] + rx && l[0] + l[2] > c[0] - rx && y0 < c[1] + ry && y1 > c[1] - ry;
                    assert!(!hit, "yaw {yaw}: streak {l:?} crosses hex {hex:?} at {c:?}");
                }
            }
        }
        assert!(seen > 24 * 4, "yaw {yaw}: streaks still play in the margins ({seen})");
    }

    #[test]
    fn cutin_streaks_stay_off_the_board_and_the_caption() {
        for (yaw, max_zoom) in [(0u8, false), (1, false), (2, true), (3, true)] {
            let mut game = waiting_for_yoma(true);
            game.ui.yaw = yaw;
            if max_zoom {
                game.set_combat_zoom(10.0);
            }
            assert_streaks_clear(game, "YOMA REACHES", yaw);
        }
    }

    /// Clare adjacent to a yoma with the FLASH gate open (trans 50) and AP to spare.
    fn clare_ready_to_flash(trans: i32) -> (Game, Axial) {
        let mut game = adjacent_yoma(true);
        let combat = game.combat.as_mut().unwrap();
        let clare = combat.units.iter_mut().find(|u| u.id == "clare").unwrap();
        clare.trans = trans;
        clare.yoki = 30;
        clare.ap = 3;
        clare.max_ap = 3;
        let prey = core_hex(combat.units.iter().find(|u| u.id == "yoma-0").unwrap());
        (game, prey)
    }

    fn flash_at(hex: Axial) -> PlayerAction {
        PlayerAction::Skill { id: "flash".into(), hex }
    }

    #[test]
    fn flash_arms_a_presentational_cutin_on_clare() {
        let (mut game, prey) = clare_ready_to_flash(50);
        assert!(legal_targets(game.combat.as_ref().unwrap(), "clare", "flash").contains(&prey));
        // Same outcome as the bare rules call: the cut-in changes nothing.
        let mut control = game.combat.clone().unwrap();
        act(&mut control, flash_at(prey), true);
        game.combat_act(flash_at(prey));
        assert_eq!(
            serde_json::to_string(game.combat.as_ref().unwrap()).unwrap(),
            serde_json::to_string(&control).unwrap(),
        );
        let cut = game.cutin.clone().expect("FLASH arms a cut-in");
        assert_eq!((cut.unit_id.as_str(), cut.kind), ("clare", beats::CutInKind::Flash));
        assert!(game.fx.floaters.iter().any(|f| f.plate && f.text == beats::FLASH_CAPTION));
        // Non-blocking: Clare keeps her turn and input stays open; no enemy beat.
        assert!(game.combat_input_open());
        assert!(game.enemy_beat_at.is_none() && game.enemy_tell.is_none());
        // Camera pushes toward Clare, then eases back and the cut-in retires.
        game.ui.zoom = 0.9;
        game.fx.time = cut.start + beats::FLASH_IN;
        assert!(game.view_camera().1 > game.ui.zoom * 1.05);
        let clare = game.combat.as_ref().unwrap().units.iter().find(|u| u.id == "clare").unwrap().origin;
        let p = game.hex_screen(clare);
        assert!(p[0] > 0.05 && p[0] < 0.95 && p[1] > 0.15 && p[1] < 0.85, "{p:?}");
        game.fx.time = cut.end();
        game.tick_enemy_tell();
        assert!(game.cutin.is_none());
        assert_eq!(game.view_camera(), (game.ui.pan, game.ui.zoom));
    }

    #[test]
    fn flash_cutin_needs_a_landed_flash() {
        // Below the trans 40 gate the rules refuse FLASH: no cut-in.
        let (mut game, prey) = clare_ready_to_flash(39);
        game.combat_act(flash_at(prey));
        assert!(game.cutin.is_none());
        // Other skills never arm it.
        let (mut game, prey) = clare_ready_to_flash(50);
        game.combat_act(PlayerAction::Skill { id: "cut".into(), hex: prey });
        assert!(game.cutin.is_none());
    }

    #[test]
    fn flash_ending_the_turn_keeps_beat_timing() {
        let (mut game, prey) = clare_ready_to_flash(50);
        game.combat.as_mut().unwrap().units.iter_mut().find(|u| u.id == "clare").unwrap().ap = 1;
        let now = game.fx.time;
        game.combat_act(flash_at(prey));
        let c = game.combat.as_ref().unwrap();
        if c.over.is_none() && current_unit(c).map(|u| u.side) == Some(Side::Enemy) {
            assert_eq!(game.enemy_beat_at, Some(beats::next_beat_at(now)));
        }
        assert_eq!(game.cutin.as_ref().unwrap().kind, beats::CutInKind::Flash);
    }

    #[test]
    fn flash_streaks_stay_off_the_board_units_and_caption() {
        for (yaw, max_zoom) in [(0u8, false), (1, false), (2, true), (3, true)] {
            let (mut game, prey) = clare_ready_to_flash(50);
            game.ui.yaw = yaw;
            if max_zoom {
                game.set_combat_zoom(10.0);
            }
            game.combat_act(flash_at(prey));
            if game.combat.is_none() {
                continue; // a one-shot kill ends the fight
            }
            assert_streaks_clear(game, beats::FLASH_CAPTION, yaw);
        }
    }

    #[test]
    fn claw_windup_arms_cutin_push_in_then_eases_back() {
        let mut game = waiting_for_yoma(true);
        game.ui.zoom = 0.9; // room under the HUD fit cap
        let base = game.view_camera();
        let cut = game.cutin.clone().expect("windup arms a cut-in");
        assert_eq!(cut.unit_id, "yoma-0");
        let tell = game.enemy_tell.as_ref().unwrap().resolve_at;
        assert!((cut.strike_at - tell).abs() < 1e-6, "cut-in matches the windup");
        assert!((tell - cut.start - ENEMY_WINDUP).abs() < 1e-5);
        assert_eq!(base.1, game.ui.zoom, "push starts from the player's zoom");

        game.fx.time = cut.start + super::beats::CUTIN_IN;
        let (_, pushed) = game.view_camera();
        assert!(pushed > game.ui.zoom * 1.05, "camera pushes in: {pushed}");
        let (cols, rows) = {
            let c = game.combat.as_ref().unwrap();
            (c.cols, c.rows)
        };
        let [sw, sh] = game.ui.screen;
        let cap = crate::iso::board_zoom_max(cols, rows, sw, sh, game.ui.yaw);
        assert!(pushed <= cap.max(game.ui.zoom) * (1.0 + super::beats::CUTIN_CAP_SLACK) + 1e-4);
        // The clawing yoma stays roughly put on screen while the board grows.
        let y = game.combat.as_ref().unwrap().units.iter().find(|u| u.id == "yoma-0").unwrap().origin;
        let p = game.hex_screen(y);
        assert!(p[0] > 0.05 && p[0] < 0.95 && p[1] > 0.15 && p[1] < 0.85, "{p:?}");

        resolve_pending(&mut game);
        assert!(game.enemy_tell.is_none());
        assert!(game.cutin.is_some(), "cut-in eases out after the strike");
        game.fx.time = cut.end() - 0.01;
        assert!(game.view_camera().1 > game.ui.zoom);
        game.fx.time = cut.end();
        game.tick_enemy_tell();
        assert!(game.cutin.is_none());
        assert_eq!(game.view_camera(), (game.ui.pan, game.ui.zoom));
    }

    fn center(r: hud::Rect) -> (f32, f32) {
        (r.x + r.w * 0.5, r.y + r.h * 0.5)
    }

    #[test]
    fn buttons_enabled_only_on_a_live_player_turn() {
        let game = adjacent_yoma(false);
        let combat = game.combat.as_ref().unwrap();
        assert!(combat_buttons_enabled(Some(combat), false), "Clare's turn");
        assert!(!combat_buttons_enabled(Some(combat), true), "claw windup armed");
        assert!(!combat_buttons_enabled(None, false), "no fight");
        let mut enemy = combat.clone();
        enemy.turn = 1;
        assert_eq!(current_unit(&enemy).unwrap().side, Side::Enemy);
        assert!(!combat_buttons_enabled(Some(&enemy), false), "yoma beat");
        let mut over = combat.clone();
        over.over = Some(true);
        assert!(!combat_buttons_enabled(Some(&over), false), "fight resolved");
        // The Game predicate also requires combat mode.
        assert!(game.combat_input_open());
        let mut world = adjacent_yoma(false);
        world.mode = Mode::World;
        assert!(!world.combat_input_open());
    }

    #[test]
    fn buttons_and_hotkeys_stay_blocked_through_beats_and_windup() {
        use winit::keyboard::KeyCode;
        for raki in [false, true] {
            let mut game = adjacent_yoma(raki);
            game.ui.selected_skill = Some("cut".into());
            game.combat_act(PlayerAction::Wait);
            assert!(!game.combat_input_open(), "enemy beat pending");
            assert!(game.ui.selected_skill.is_none(), "half-picked skill cleared");
            let bar = hud::combat_bar();
            let screen = game.ui.screen;
            for phase in ["beat", "windup"] {
                if phase == "windup" {
                    fire_beat(&mut game);
                    assert!(game.enemy_tell.is_some());
                    assert!(!game.combat_input_open(), "claw windup");
                }
                for key in [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::KeyG] {
                    game.key(key, true);
                    game.key(key, false);
                }
                for r in [bar.guard, bar.cut, bar.slot, bar.wait, bar.raise] {
                    let (x, y) = center(r);
                    game.click_combat(x, y, screen);
                }
                assert!(game.ui.selected_skill.is_none(), "{phase}: no skill picked");
                assert_eq!(game.mode, Mode::Combat);
            }
            resolve_pending(&mut game);
            assert!(game.combat_input_open(), "turn back to Clare; Raki={raki}");
            game.key(KeyCode::Digit1, true);
            assert_eq!(game.ui.selected_skill.as_deref(), Some("cut"));
        }
    }

    #[test]
    fn fall_stays_live_during_enemy_beats() {
        let mut game = adjacent_yoma(false);
        game.combat_act(PlayerAction::Wait);
        assert!(!game.combat_input_open());
        let (x, y) = center(hud::combat_bar().forfeit);
        let screen = game.ui.screen;
        game.click_combat(x, y, screen);
        assert_ne!(game.mode, Mode::Combat, "FALL forfeits even mid enemy phase");
    }

    #[test]
    fn holding_shift_runs_beats_windup_and_cutin_twice_as_fast() {
        use winit::keyboard::KeyCode;
        let mut game = adjacent_yoma(false);
        // Not an enemy phase: holding does nothing.
        game.keys.push(KeyCode::ShiftLeft);
        assert!(!game.fast_forwarding());
        game.keys.clear();
        game.combat_act(PlayerAction::Wait);
        let at = game.enemy_beat_at.unwrap();
        game.fast_forward_enemy(0.05);
        assert_eq!(game.enemy_beat_at, Some(at), "real-time when not held");
        game.keys.push(KeyCode::ShiftLeft);
        assert!(game.fast_forwarding());
        let dt = 0.05;
        game.fast_forward_enemy(dt);
        let skip = dt * (beats::FAST_FORWARD - 1.0);
        assert!((game.enemy_beat_at.unwrap() - (at - skip)).abs() < 1e-6);
        // Windup and its cut-in shift together, keeping the envelope intact.
        fire_beat(&mut game);
        let tell = game.enemy_tell.as_ref().unwrap().resolve_at;
        let cut = game.cutin.clone().unwrap();
        game.fast_forward_enemy(dt);
        assert!((game.enemy_tell.as_ref().unwrap().resolve_at - (tell - skip)).abs() < 1e-6);
        let c2 = game.cutin.as_ref().unwrap();
        assert!((c2.start - (cut.start - skip)).abs() < 1e-6);
        assert!((c2.strike_at - (cut.strike_at - skip)).abs() < 1e-6);
        // Space still held after WAIT also counts.
        game.keys.clear();
        game.keys.push(KeyCode::Space);
        assert!(game.fast_forwarding());
        resolve_pending(&mut game);
        assert!(!game.fast_forwarding(), "player turn: no x2");
    }

    #[test]
    fn focus_loss_drops_held_keys_fast_forward_and_drag() {
        use winit::keyboard::KeyCode;
        let mut game = adjacent_yoma(false);
        game.combat_act(PlayerAction::Wait);
        game.key(KeyCode::ShiftLeft, true);
        game.key(KeyCode::KeyD, true);
        game.ui.dragging = true;
        assert!(game.fast_forwarding());
        // Alt-tab: the Shift key-up goes to another window and never arrives.
        game.focus_lost();
        assert!(game.keys.is_empty());
        assert!(!game.fast_forward_held());
        assert!(!game.fast_forwarding(), "x2 must not stick after blur");
        assert!(!game.ui.dragging);
        // Pressing Shift again after refocus is a fresh press.
        game.key(KeyCode::ShiftLeft, true);
        assert!(game.fast_forwarding());
    }
}
