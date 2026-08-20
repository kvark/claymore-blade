//! Move / target legality.

use super::*;
use crate::catalog::{self};
use crate::hex::{
    facing_toward, hex_cone, hex_disc, hex_distance, hex_eq, hex_line, hex_neighbors, hex_ring,
    hex_sweep, Axial,
};

pub fn can_use(u: &Unit, skill: &SkillDef, has_raki: bool) -> bool {
    if u.ap < skill.ap || u.trans < skill.trans || u.yoki < skill.yoki {
        return false;
    }
    if skill.id == "drop" && !has_raki {
        return false;
    }
    if skill.id == "ripple"
        && !u
            .parts
            .iter()
            .any(|p| p.zone.as_deref() == Some("ripple") && p.hp > 0)
    {
        return false;
    }
    true
}

pub(super) fn move_cost(state: &CombatState, hex: Axial) -> i32 {
    match terrain_at(state, hex) {
        Terrain::Water => 99,
        Terrain::Mud => 2,
        _ => 1,
    }
}

pub fn legal_moves(state: &CombatState, unit_id: &str) -> Vec<Axial> {
    let Some(u) = state.units.iter().find(|x| x.id == unit_id) else {
        return vec![];
    };
    if u.dead {
        return vec![];
    }
    let occ = occupied(state, Some(&u.id));
    let start = core_hex(u);
    let budget = u.ap.min(derived(&u.stats).r#move);
    let mut out = Vec::new();
    let mut seen = vec![(start, 0)];
    let mut q = vec![(start, 0)];
    while let Some((cur, c)) = q.pop() {
        for n in hex_neighbors(cur) {
            if !in_bounds(n, state.cols, state.rows) {
                continue;
            }
            let cost = c + move_cost(state, n);
            if cost > budget {
                continue;
            }
            if seen.iter().any(|(h, sc)| hex_eq(*h, n) && *sc <= cost) {
                continue;
            }
            if occ.iter().any(|(h, _)| hex_eq(*h, n)) {
                continue;
            }
            seen.push((n, cost));
            out.push(n);
            q.push((n, cost));
        }
    }
    out
}

pub fn zone_for(state: &CombatState, u: &Unit, skill: &SkillDef, target: Axial) -> Vec<Axial> {
    let from = core_hex(u);
    let face = facing_toward(from, target);
    let clip = |v: Vec<Axial>| {
        v.into_iter()
            .filter(|h| in_bounds(*h, state.cols, state.rows))
            .collect()
    };
    match skill.shape {
        ShapeKind::SelfCast => vec![from],
        ShapeKind::Single | ShapeKind::Leap => vec![target],
        ShapeKind::Line => {
            let line = hex_line(from, target);
            clip(
                line.into_iter()
                    .skip(1)
                    .take(skill.length as usize)
                    .collect(),
            )
        }
        ShapeKind::Cone => clip(hex_cone(from, face, skill.range)),
        ShapeKind::Blast => clip(hex_disc(target, skill.range)),
        ShapeKind::Ring => clip(hex_ring(from, skill.range)),
        ShapeKind::Sweep => clip(hex_sweep(from, face)),
        ShapeKind::Ripple => clip(hex_ring(from, 1)),
    }
}

pub fn legal_targets(state: &CombatState, unit_id: &str, skill_id: &str) -> Vec<Axial> {
    let Some(u) = state.units.iter().find(|x| x.id == unit_id) else {
        return vec![];
    };
    let Some(skill) = catalog::skill(skill_id) else {
        return vec![];
    };
    let from = core_hex(u);
    if skill.self_cast || skill.shape == ShapeKind::SelfCast || skill.shape == ShapeKind::Ripple {
        return vec![from];
    }
    if skill.shape == ShapeKind::Leap {
        let occ = occupied(state, Some(&u.id));
        return hex_disc(from, skill.range)
            .into_iter()
            .filter(|h| {
                !hex_eq(*h, from)
                    && in_bounds(*h, state.cols, state.rows)
                    && terrain_at(state, *h) != Terrain::Water
                    && !occ.iter().any(|(c, _)| hex_eq(*c, *h))
            })
            .collect();
    }
    let mut cells = Vec::new();
    for q in 0..state.cols {
        for r in 0..state.rows {
            let h = Axial::new(q, r);
            let d = hex_distance(from, h);
            if d >= 1 && d <= skill.range {
                cells.push(h);
            }
        }
    }
    cells
}
