//! Enemy turn policy + readable strike tells.

use super::*;
use crate::catalog::{self};
use crate::hex::{hex_distance, hex_eq, Axial};

/// What an enemy would swing right now (no move / Raise). Pure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntendedStrike {
    pub unit_id: String,
    pub skill_id: String,
    pub hex: Axial,
}

pub(crate) fn pick_ai_skill(_state: &CombatState, u: &Unit) -> Option<&'static SkillDef> {
    let mut usable: Vec<_> = u
        .skills
        .iter()
        .filter_map(|id| catalog::skill(id))
        .filter(|s| can_use(u, s, false) && s.power > 0)
        .collect();
    usable.sort_by_key(|s| -s.power);
    usable.first().copied()
}

fn nearest_foe(state: &CombatState, u: &Unit) -> Option<Unit> {
    let foes = living(state, Some(Side::Player));
    if foes.is_empty() {
        return None;
    }
    let from = core_hex(u);
    let mut foes_owned: Vec<_> = foes.into_iter().cloned().collect();
    foes_owned.sort_by_key(|a| {
        let lured = a.statuses.iter().any(|s| s.id == "lured");
        let boy = a.template_id == "raki";
        let d = hex_distance(from, core_hex(a));
        // Lured first, then warriors, then the boy if he is in biting range.
        (!lured, boy && d > 1, d)
    });
    foes_owned.into_iter().next()
}

/// Peek the skill + aim hex this enemy would fire from its current cell.
/// `None` when out of reach, no living foe, or they still need Raise.
pub fn intended_strike(state: &CombatState, u: &Unit) -> Option<IntendedStrike> {
    if u.dead || u.side != Side::Enemy {
        return None;
    }
    let nearest = nearest_foe(state, u)?;
    let skill = pick_ai_skill(state, u)?;
    if u.trans < skill.trans && !u.raised_trans {
        return None;
    }
    if !can_use(u, skill, false) {
        return None;
    }
    let targets = legal_targets(state, &u.id, skill.id);
    let foe_cells = live_cells(&nearest);
    let hit = targets.iter().find(|t| {
        zone_for(state, u, skill, **t)
            .iter()
            .any(|h| foe_cells.iter().any(|c| hex_eq(*c, *h)))
    })?;
    Some(IntendedStrike {
        unit_id: u.id.clone(),
        skill_id: skill.id.into(),
        hex: *hit,
    })
}

/// Next living enemy that will act: current if enemy, else the next in `order`.
pub fn next_enemy_unit(state: &CombatState) -> Option<&Unit> {
    if let Some(u) = current_unit(state) {
        if u.side == Side::Enemy && !u.dead {
            return Some(u);
        }
    }
    let n = state.order.len();
    if n == 0 {
        return None;
    }
    for i in 1..=n {
        let idx = (state.turn + i) % n;
        let id = &state.order[idx];
        if let Some(u) = state
            .units
            .iter()
            .find(|u| u.id == *id && !u.dead && u.side == Side::Enemy)
        {
            return Some(u);
        }
    }
    None
}

/// Peek the next yoma's claw while the player still owns the turn (or mid-windup).
pub fn peek_enemy_tell(state: &CombatState) -> Option<IntendedStrike> {
    let u = next_enemy_unit(state)?;
    intended_strike(state, u)
}

/// One enemy beat: what the current enemy did (or is about to strike with).
#[derive(Clone, Debug)]
pub enum AiBeat {
    /// Applied exactly one Raise / Move / Wait (the turn may have advanced).
    Acted(PlayerAction),
    /// The current enemy is ready to strike; nothing was applied yet.
    Strike(IntendedStrike),
    /// Not an enemy turn (or the fight is over); nothing to do.
    Idle,
}

/// Apply at most one Raise / Move / Wait for the current enemy, or return the
/// strike it is ready to make without resolving it. Callers pace these beats.
pub fn ai_beat(state: &mut CombatState) -> AiBeat {
    if state.over.is_some() {
        return AiBeat::Idle;
    }
    let Some(u) = current_unit(state).cloned() else {
        return AiBeat::Idle;
    };
    if u.side != Side::Enemy {
        return AiBeat::Idle;
    }
    let Some(nearest) = nearest_foe(state, &u) else {
        return AiBeat::Idle;
    };
    if let Some(strike) = intended_strike(state, &u) {
        return AiBeat::Strike(strike);
    }
    if let Some(skill) = pick_ai_skill(state, &u) {
        if u.trans < skill.trans && !u.raised_trans {
            act(state, PlayerAction::Raise, false);
            return AiBeat::Acted(PlayerAction::Raise);
        }
    }
    let from = core_hex(&u);
    let moves = legal_moves(state, &u.id);
    if let Some(step) = moves
        .into_iter()
        .min_by_key(|h| hex_distance(*h, core_hex(&nearest)))
    {
        if hex_distance(step, core_hex(&nearest)) < hex_distance(from, core_hex(&nearest)) {
            act(state, PlayerAction::Move(step), false);
            return AiBeat::Acted(PlayerAction::Move(step));
        }
    }
    act(state, PlayerAction::Wait, false);
    AiBeat::Acted(PlayerAction::Wait)
}

/// Raise / Move / Wait until the current enemy is ready to strike, or the enemy
/// phase ends. Returns the pending strike without resolving it. Instant (tests,
/// headless); the game paces the same beats with [`ai_beat`].
pub fn run_ai_prep(state: &mut CombatState) -> Option<IntendedStrike> {
    for _ in 0..24 {
        match ai_beat(state) {
            AiBeat::Acted(_) => continue,
            AiBeat::Strike(strike) => return Some(strike),
            AiBeat::Idle => return None,
        }
    }
    None
}

pub fn run_ai(state: &mut CombatState) {
    while let Some(strike) = run_ai_prep(state) {
        act(
            state,
            PlayerAction::Skill {
                id: strike.skill_id,
                hex: strike.hex,
            },
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::Axial;

    #[test]
    fn intended_strike_claws_clare_in_range() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 7);
        let clare_hex = s
            .units
            .iter()
            .find(|u| u.id == "clare")
            .map(|u| u.origin)
            .expect("clare");
        // Park the first yoma on a neighbor of Clare so Rend is legal.
        let neighbor = Axial::new(clare_hex.q + 1, clare_hex.r);
        {
            let yoma = s
                .units
                .iter_mut()
                .find(|u| u.template_id == "yoma")
                .expect("yoma");
            yoma.origin = neighbor;
        }
        let yoma = s
            .units
            .iter()
            .find(|u| u.template_id == "yoma")
            .expect("yoma")
            .clone();
        let strike = intended_strike(&s, &yoma).expect("claw in range");
        assert_eq!(strike.skill_id, "claw");
        assert_eq!(strike.unit_id, yoma.id);
        // Aim hex is Clare's cell (Single shape).
        assert_eq!(strike.hex, clare_hex);
    }

    #[test]
    fn intended_strike_none_when_far_or_no_foe() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 7);
        let yoma = s
            .units
            .iter()
            .find(|u| u.template_id == "yoma")
            .expect("yoma")
            .clone();
        // Opening Doga spawn is out of claw range.
        assert!(
            intended_strike(&s, &yoma).is_none(),
            "far yoma must not telegraph a claw"
        );

        for u in &mut s.units {
            if u.side == Side::Player {
                u.dead = true;
                u.hp = 0;
            }
        }
        assert!(
            intended_strike(&s, &yoma).is_none(),
            "no living foe → no tell"
        );
    }

    #[test]
    fn peek_enemy_tell_reads_next_yoma_on_player_turn() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 7);
        // Ensure Clare is current.
        if let Some(i) = s.order.iter().position(|id| id == "clare") {
            s.turn = i;
        }
        assert_eq!(current_unit(&s).map(|u| u.id.as_str()), Some("clare"));
        let clare_hex = s.units.iter().find(|u| u.id == "clare").unwrap().origin;
        let neighbor = Axial::new(clare_hex.q + 1, clare_hex.r);
        let yoma_id = {
            let y = s
                .units
                .iter_mut()
                .find(|u| u.template_id == "yoma")
                .unwrap();
            y.origin = neighbor;
            y.id.clone()
        };
        let tell = peek_enemy_tell(&s).expect("next yoma in claw range");
        assert_eq!(tell.unit_id, yoma_id);
        assert_eq!(tell.skill_id, "claw");
        assert_eq!(tell.hex, clare_hex);
    }

    fn far_yoma_battle() -> CombatState {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 7);
        let first = s
            .units
            .iter()
            .find(|u| u.template_id == "yoma")
            .unwrap()
            .id
            .clone();
        s.turn = s.order.iter().position(|id| *id == first).unwrap();
        s
    }

    #[test]
    fn ai_beat_applies_exactly_one_action() {
        let mut s = far_yoma_battle();
        let actor = current_unit(&s).unwrap().id.clone();
        let log_before = s.log.len();
        let beat = ai_beat(&mut s);
        assert!(matches!(beat, AiBeat::Acted(_)), "far yoma should act, got {beat:?}");
        // One action → at most one log line from the actor and no second enemy moved.
        assert!(s.log.len() - log_before <= 1, "{:?}", &s.log[log_before..]);
        let moved: Vec<_> = s
            .units
            .iter()
            .filter(|u| u.side == Side::Enemy && u.id != actor)
            .filter(|u| {
                far_yoma_battle()
                    .units
                    .iter()
                    .find(|o| o.id == u.id)
                    .map(|o| !hex_eq(o.origin, u.origin))
                    .unwrap_or(false)
            })
            .collect();
        assert!(moved.is_empty(), "another enemy acted in the same beat");
    }

    #[test]
    fn ai_beat_idle_on_player_turn_and_strike_is_not_applied() {
        let mut s = far_yoma_battle();
        let i = s.order.iter().position(|id| id == "clare").unwrap();
        s.turn = i;
        let before = serde_json::to_string(&s).unwrap();
        assert!(matches!(ai_beat(&mut s), AiBeat::Idle));
        assert_eq!(serde_json::to_string(&s).unwrap(), before);

        // Adjacent yoma: the beat reports the strike and leaves state untouched.
        let mut s = far_yoma_battle();
        let clare = s.units.iter().find(|u| u.id == "clare").unwrap().origin;
        let actor = current_unit(&s).unwrap().id.clone();
        s.units.iter_mut().find(|u| u.id == actor).unwrap().origin =
            Axial::new(clare.q + 1, clare.r);
        let before = serde_json::to_string(&s).unwrap();
        match ai_beat(&mut s) {
            AiBeat::Strike(st) => assert_eq!(st.unit_id, actor),
            other => panic!("expected strike, got {other:?}"),
        }
        assert_eq!(serde_json::to_string(&s).unwrap(), before);
    }

    #[test]
    fn run_ai_prep_matches_repeated_beats() {
        let mut a = far_yoma_battle();
        let mut b = a.clone();
        let strike = run_ai_prep(&mut a);
        let mut beats = 0;
        let stepped = loop {
            match ai_beat(&mut b) {
                AiBeat::Acted(_) => beats += 1,
                AiBeat::Strike(s) => break Some(s),
                AiBeat::Idle => break None,
            }
            assert!(beats < 24);
        };
        assert!(beats >= 1);
        assert_eq!(strike, stepped);
        assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
    }
}
