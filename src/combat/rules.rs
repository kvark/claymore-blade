//! Move / target legality.

use super::*;
use crate::catalog::{self, derived};
use crate::hex::{
    facing_toward, hex_cone, hex_disc, hex_distance, hex_eq, hex_line, hex_neighbors, hex_ring,
    hex_sweep, Axial,
};

pub fn raki_can_help(state: &CombatState, actor_id: &str) -> bool {
    match state.units.iter().find(|u| u.template_id == "raki") {
        Some(raki) if !raki.dead => {
            let Some(actor) = state.units.iter().find(|u| u.id == actor_id) else {
                return false;
            };
            hex_distance(core_hex(actor), core_hex(raki)) <= 3
        }
        Some(_) => false,
        None => state.support_raki,
    }
}

pub fn can_use(u: &Unit, skill: &SkillDef, has_raki: bool) -> bool {
    if u.ap < skill.ap || u.trans < skill.trans || u.yoki < skill.yoki {
        return false;
    }
    if skill.id == "drop" && !has_raki {
        return false;
    }
    if skill.id == "lure" && u.template_id != "raki" {
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

pub(crate) fn move_cost(state: &CombatState, hex: Axial) -> i32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_needs_the_boy_close() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let mut s = create_battle(enc, &["clare".into()], 3);
        assert!(s.units.iter().any(|u| u.template_id == "raki"));
        if let Some(r) = s.units.iter_mut().find(|u| u.template_id == "raki") {
            r.origin = Axial::new(s.cols - 1, 0);
        }
        if let Some(c) = s.units.iter_mut().find(|u| u.id == "clare") {
            c.origin = Axial::new(0, s.rows - 1);
        }
        assert!(!raki_can_help(&s, "clare"));
        let clare = s.units.iter().find(|u| u.id == "clare").cloned().unwrap();
        let drop = catalog::skill("drop").unwrap();
        assert!(!can_use(&clare, drop, false));
        assert!(can_use(&clare, drop, true));
    }

    #[test]
    fn lure_is_only_the_boy() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let s = create_battle(enc, &["clare".into()], 3);
        let lure = catalog::skill("lure").unwrap();
        let clare = s.units.iter().find(|u| u.id == "clare").unwrap();
        let raki = s.units.iter().find(|u| u.template_id == "raki").unwrap();
        assert!(!can_use(clare, lure, true));
        assert!(can_use(raki, lure, true));
    }
}
