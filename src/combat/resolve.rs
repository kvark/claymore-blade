//! Hits, zones, and the act() entry point.

use super::*;
use crate::catalog::{self, derived};
use crate::hex::{facing_toward, hex_distance, hex_eq, place_footprint, Axial};
use crate::rng::Rng;

pub(crate) fn resolve_hit(
    rng: &mut Rng,
    atk: &Unit,
    def: &Unit,
    skill: &SkillDef,
    pa: i32,
    pd: i32,
) -> (HitKind, i32) {
    let scale = effect_scale(pa, pd);
    let power = (skill.power as f32 * scale).round() as i32;
    if skill.unblockable {
        return (HitKind::Solid, power.max(1));
    }
    let hit_chance = 0.55 + 0.05 * (pa - pd) as f32 + if skill.aimed { 0.12 } else { 0.0 };
    if !rng.chance(hit_chance.clamp(0.15, 0.95)) {
        return (HitKind::Miss, 0);
    }
    let guard = def
        .statuses
        .iter()
        .filter(|s| s.guard > 0)
        .map(|s| s.guard)
        .sum::<i32>();
    if guard > 0 && rng.chance(0.35 + guard as f32 * 0.08) {
        return (HitKind::Blocked, (power / 3).max(1));
    }
    if rng.chance(0.18) {
        return (HitKind::Glance, (power / 2).max(1));
    }
    (HitKind::Solid, power.max(1))
}

pub(crate) fn apply_zone(
    state: &mut CombatState,
    cells: &[Axial],
    power: i32,
    pa: Attr,
    pd: Attr,
    actor_id: &str,
    aimed: bool,
    unblockable: bool,
) {
    let mut rng = Rng::new(state.seed.wrapping_add(state.round as u32 * 31 + state.turn as u32));
    let Some(atk) = state.units.iter().find(|u| u.id == actor_id).cloned() else {
        return;
    };
    for &cell in cells {
        let targets: Vec<_> = state
            .units
            .iter()
            .filter(|u| !u.dead && live_cells(u).iter().any(|h| hex_eq(*h, cell)))
            .map(|u| u.id.clone())
            .collect();
        for tid in targets {
            let Some(def) = state.units.iter().find(|u| u.id == tid).cloned() else {
                continue;
            };
            if def.side == atk.side && !unblockable {
                continue;
            }
            let skill = SkillDef {
                id: "zone",
                name: "Zone",
                blurb: "",
                ap: 0,
                trans: 0,
                yoki: 0,
                shape: ShapeKind::Single,
                range: 0,
                length: 0,
                pa,
                pd,
                power,
                aimed,
                self_cast: false,
                heal: 0,
                trans_delta: 0,
                r#move: 0,
                guard: 0,
                telegraph: false,
                afterimage: false,
                unblockable,
                strikes: true,
            };
            let (kind, dmg) = resolve_hit(
                &mut rng,
                &atk,
                &def,
                &skill,
                atk.stats.get(pa),
                def.stats.get(pd),
            );
            apply_damage(state, &tid, dmg, kind, &atk.name);
        }
    }
}

pub(crate) fn apply_damage(
    state: &mut CombatState,
    unit_id: &str,
    dmg: i32,
    kind: HitKind,
    from: &str,
) {
    let Some(u) = state.units.iter_mut().find(|x| x.id == unit_id) else {
        return;
    };
    if u.dead {
        return;
    }
    let label = match kind {
        HitKind::Miss => "misses",
        HitKind::Glance => "glances",
        HitKind::Blocked => "is blocked",
        HitKind::Solid => "hits",
    };
    if dmg <= 0 {
        push_log(
            state,
            "miss",
            format!("{} {} {}.", from, label, u.name),
        );
        return;
    }
    // distribute to parts, prefer limbs
    let mut remaining = dmg;
    let mut parts: Vec<_> = u.parts.iter_mut().filter(|p| p.hp > 0).collect();
    parts.sort_by_key(|p| if p.zone.is_some() { 0 } else { 1 });
    for p in &mut parts {
        if remaining <= 0 {
            break;
        }
        let take = remaining.min(p.hp);
        p.hp -= take;
        remaining -= take;
    }
    u.hp = u.parts.iter().map(|p| p.hp).sum();
    push_log(
        state,
        "hit",
        format!("{} {} {} for {}.", from, label, u.name, dmg - remaining),
    );
    if u.hp <= 0 {
        u.dead = true;
        u.ap = 0;
        push_log(state, "kill", format!("{} falls.", u.name));
    }
}

pub fn act(state: &mut CombatState, action: PlayerAction, is_player: bool) {
    if state.over.is_some() {
        return;
    }
    let Some(actor_id) = state.order.get(state.turn).cloned() else {
        return;
    };
    let Some(u) = state.units.iter().find(|x| x.id == actor_id).cloned() else {
        return;
    };
    if u.dead {
        advance_turn(state);
        return;
    }
    if is_player && u.side != Side::Player {
        return;
    }
    match action {
        PlayerAction::Wait => {
            if let Some(uu) = current_unit_mut(state) {
                uu.ap = 0;
            }
            push_log(state, "info", format!("{} waits.", u.name));
            advance_turn(state);
        }
        PlayerAction::Raise => {
            if let Some(uu) = current_unit_mut(state) {
                if !uu.raised_trans && uu.trans < 100 {
                    uu.trans = (uu.trans + 15).min(100);
                    uu.raised_trans = true;
                    push_log(
                        state,
                        "trans",
                        format!("{} raises yoki. Trans {}.", uu.name, uu.trans),
                    );
                }
            }
        }
        PlayerAction::Move(hex) => {
            let legal = legal_moves(state, &actor_id);
            if !legal.iter().any(|h| hex_eq(*h, hex)) {
                return;
            }
            if let Some(uu) = current_unit_mut(state) {
                let cost = move_cost(state, hex);
                if uu.ap < cost {
                    return;
                }
                uu.origin = hex;
                uu.ap -= cost;
                push_log(state, "move", format!("{} advances.", uu.name));
            }
            let ap = current_unit(state).map(|u| u.ap).unwrap_or(0);
            if ap <= 0 {
                advance_turn(state);
            }
        }
        PlayerAction::Skill { id, hex } => {
            let Some(skill) = catalog::skill(&id) else {
                return;
            };
            if !can_use(&u, skill, true) {
                return;
            }
            let targets = legal_targets(state, &actor_id, &id);
            if !targets.iter().any(|h| hex_eq(*h, hex)) {
                return;
            }
            if let Some(uu) = current_unit_mut(state) {
                uu.ap -= skill.ap;
                uu.yoki = (uu.yoki - skill.yoki).max(0);
                if skill.trans_delta != 0 {
                    uu.trans = (uu.trans + skill.trans_delta).clamp(0, 100);
                }
                if skill.r#move != 0 {
                    // simple leap or step handled by zone
                }
                if skill.guard > 0 {
                    uu.statuses.push(Status {
                        id: "guard".into(),
                        name: "Guard".into(),
                        turns: 1,
                        guard: skill.guard,
                        telegraph: false,
                        afterimage: None,
                    });
                }
            }
            let zone = zone_for(state, &u, skill, hex);
            if skill.shape == ShapeKind::Ripple {
                state.zones.push(DelayedZone {
                    source_id: actor_id.clone(),
                    center: core_hex(&u),
                    radius: 1,
                    max_radius: skill.range.max(2),
                    power: skill.power,
                    pa: skill.pa,
                });
                push_log(state, "skill", format!("{} starts a ripple.", u.name));
            } else if skill.heal > 0 {
                if let Some(uu) = current_unit_mut(state) {
                    let heal = skill.heal;
                    uu.hp = (uu.hp + heal).min(uu.max_hp);
                    for p in &mut uu.parts {
                        p.hp = (p.hp + heal / uu.parts.len() as i32).min(p.max_hp);
                    }
                    push_log(state, "heal", format!("{} recovers {}.", uu.name, heal));
                }
            } else {
                push_log(
                    state,
                    "skill",
                    format!("{} uses {}.", u.name, skill.name),
                );
                if skill.strikes {
                    apply_zone(
                        state,
                        &zone,
                        skill.power,
                        skill.pa,
                        skill.pd,
                        &actor_id,
                        skill.aimed,
                        skill.unblockable,
                    );
                }
            }
            let ap = current_unit(state).map(|u| u.ap).unwrap_or(0);
            if ap <= 0 {
                advance_turn(state);
            }
            check_over(state);
        }
    }
}
