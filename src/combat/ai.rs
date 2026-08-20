//! Enemy turn policy.

use super::*;
use crate::catalog::{self};
use crate::hex::{hex_distance, hex_eq, Axial};

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

pub fn run_ai(state: &mut CombatState) {
    let mut guard = 0;
    while state.over.is_none()
        && current_unit(state)
            .map(|u| u.side == Side::Enemy)
            .unwrap_or(false)
        && guard < 24
    {
        guard += 1;
        let Some(u) = current_unit(state).cloned() else {
            break;
        };
        let foes = living(state, Some(Side::Player));
        if foes.is_empty() {
            break;
        }
        let from = core_hex(&u);
        let mut foes_owned: Vec<_> = foes.into_iter().cloned().collect();
        foes_owned.sort_by_key(|a| hex_distance(from, core_hex(a)));
        let nearest = foes_owned[0].clone();
        if let Some(skill) = pick_ai_skill(state, &u) {
            if u.trans < skill.trans && !u.raised_trans {
                act(state, PlayerAction::Raise, false);
                continue;
            }
            let targets = legal_targets(state, &u.id, skill.id);
            let foe_cells = live_cells(&nearest);
            let hit = targets.iter().find(|t| {
                zone_for(state, &u, skill, **t)
                    .iter()
                    .any(|h| foe_cells.iter().any(|c| hex_eq(*c, *h)))
            });
            if let Some(hit) = hit {
                act(
                    state,
                    PlayerAction::Skill {
                        id: skill.id.into(),
                        hex: *hit,
                    },
                    false,
                );
                continue;
            }
        }
        let moves = legal_moves(state, &u.id);
        if let Some(step) = moves
            .into_iter()
            .min_by_key(|h| hex_distance(*h, core_hex(&nearest)))
        {
            if hex_distance(step, core_hex(&nearest)) < hex_distance(from, core_hex(&nearest)) {
                act(state, PlayerAction::Move(step), false);
                continue;
            }
        }
        act(state, PlayerAction::Wait, false);
    }
}
