//! Combat UI actions, juice, and result handoff.

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

use crate::fx::Fx;

impl Game {
    pub(super) fn click_combat(&mut self, nx: f32, ny: f32, screen: [f32; 2]) {
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

    pub(super) fn pick_skill_slot(&mut self, nth: usize) {
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
                PlayerAction::Wait => self.fx.play_clip(id, crate::fx::FightClip::Ready),
                PlayerAction::Skill { id: skill, .. } if skill == "guard" => {
                    self.fx.play_clip(id, crate::fx::FightClip::Guard)
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
        self.persist();
    }
}
