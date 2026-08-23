//! Battle setup, turn flow, and helpers.

use super::*;
use crate::catalog::{self, derived, EncounterDef};
use crate::hex::{hex_eq, hex_ring, place_footprint, Axial};
use crate::rng::Rng;

pub fn effect_scale(pa: i32, pd: i32) -> f32 {
    let d = (pa - pd).clamp(-4, 8);
    1.0 + 0.25 * d as f32
}

fn max_ap(stats: &Stats) -> i32 {
    (2 + stats.a / 4).clamp(2, 5)
}

fn find_template(id: &str) -> Option<&'static UnitTemplate> {
    catalog::warrior(id).or_else(|| catalog::enemy(id))
}

pub(crate) fn in_bounds(h: Axial, cols: i32, rows: i32) -> bool {
    h.q >= 0 && h.r >= 0 && h.q < cols && h.r < rows
}

pub fn live_cells(u: &Unit) -> Vec<Axial> {
    let placed = place_footprint(&u.footprint, u.origin, u.facing);
    placed
        .into_iter()
        .enumerate()
        .filter(|(i, _)| {
            u.parts
                .iter()
                .find(|p| p.hex_index == *i)
                .map(|p| p.hp > 0)
                .unwrap_or(true)
        })
        .map(|(_, h)| h)
        .collect()
}

pub fn core_hex(u: &Unit) -> Axial {
    let placed = place_footprint(&u.footprint, u.origin, u.facing);
    placed.get(u.core_index).copied().unwrap_or(u.origin)
}

pub(crate) fn terrain_at(state: &CombatState, h: Axial) -> Terrain {
    state
        .terrain
        .iter()
        .find(|(a, _)| hex_eq(*a, h))
        .map(|(_, t)| *t)
        .unwrap_or(Terrain::Grass)
}

pub(crate) fn occupied(state: &CombatState, ignore: Option<&str>) -> Vec<(Axial, String)> {
    let mut m = Vec::new();
    for u in &state.units {
        if u.dead || ignore == Some(u.id.as_str()) {
            continue;
        }
        for c in live_cells(u) {
            m.push((c, u.id.clone()));
        }
    }
    m
}

pub(crate) fn spawn(t: &UnitTemplate, id: String, origin: Axial, facing: i32) -> Unit {
    let d = derived(&t.stats);
    let ap = max_ap(&t.stats);
    let parts: Vec<Part> = if t.parts.is_empty() {
        vec![Part {
            id: "body".into(),
            name: "Body".into(),
            hex_index: 0,
            hp: t.body_hp,
            max_hp: t.body_hp,
            zone: None,
        }]
    } else {
        t.parts
            .iter()
            .map(|(id, name, idx, hp, zone)| Part {
                id: (*id).into(),
                name: (*name).into(),
                hex_index: *idx,
                hp: *hp,
                max_hp: *hp,
                zone: zone.map(|z| z.into()),
            })
            .collect()
    };
    let hp: i32 = parts.iter().map(|p| p.hp).sum();
    Unit {
        id,
        template_id: t.id.into(),
        name: t.name.into(),
        title: t.title.into(),
        rank: t.rank,
        side: t.side,
        portrait: t.portrait.into(),
        sprite: t.sprite.into(),
        origin,
        facing,
        footprint: t.footprint.to_vec(),
        core_index: t.core_index,
        parts,
        hp,
        max_hp: hp,
        yoki: d.yoki,
        stats: t.stats,
        derived: d,
        ap,
        max_ap: ap,
        skills: t.skills.iter().map(|s| (*s).into()).collect(),
        color: t.color,
        dead: false,
        guard: false,
        trans: 0,
    }
}

pub fn living(state: &CombatState, side: Option<Side>) -> Vec<&Unit> {
    state
        .units
        .iter()
        .filter(|u| !u.dead && side.map(|s| u.side == s).unwrap_or(true))
        .collect()
}

pub fn current_unit(state: &CombatState) -> Option<&Unit> {
    let id = state.order.get(state.turn)?;
    state.units.iter().find(|u| u.id == *id && !u.dead)
}

pub(crate) fn current_unit_mut(state: &mut CombatState) -> Option<&mut Unit> {
    let id = state.order.get(state.turn)?.clone();
    state.units.iter_mut().find(|u| u.id == id && !u.dead)
}

pub(crate) fn push_log(state: &mut CombatState, kind: &str, text: String) {
    state.log.insert(
        0,
        LogLine {
            kind: kind.into(),
            text,
        },
    );
    if state.log.len() > 12 {
        state.log.truncate(12);
    }
}

pub fn create_battle(enc: &EncounterDef, party: &[String], seed: u32) -> CombatState {
    let mut rng = Rng::new(seed);
    let cols = enc.cols;
    let rows = enc.rows;
    let mut units = Vec::new();
    let mut order = Vec::new();

    for (i, id) in party.iter().enumerate() {
        let Some(t) = find_template(id) else { continue };
        let origin = enc
            .player_origins
            .get(i)
            .copied()
            .unwrap_or(Axial::new(1, rows / 2));
        let u = spawn(t, id.clone(), origin, 1);
        order.push(u.id.clone());
        units.push(u);
    }

    for (i, es) in enc.enemies.iter().enumerate() {
        let Some(t) = find_template(es.template) else { continue };
        let id = format!("{}-{}", es.template, i);
        let u = spawn(t, id, es.origin, es.facing);
        order.push(u.id.clone());
        units.push(u);
    }

    order.sort_by_key(|id| {
        let u = units.iter().find(|x| x.id == *id).unwrap();
        let noise = rng.int(0, 6);
        (-u.stats.a * 10 - noise, id.clone())
    });

    let mut terrain = Vec::new();
    for _ in 0..6 {
        let q = rng.int(0, cols - 1);
        let r = rng.int(0, rows - 1);
        let h = Axial::new(q, r);
        if !terrain.iter().any(|(a, _)| hex_eq(*a, h)) {
            terrain.push((h, Terrain::Mud));
        }
    }

    let mut state = CombatState {
        id: enc.id.into(),
        title: enc.title.into(),
        seed,
        turn: 0,
        round: 1,
        order,
        units,
        terrain,
        cols,
        rows,
        zones: Vec::new(),
        log: Vec::new(),
        over: None,
        briefing: enc.briefing.into(),
    };
    begin_turn(&mut state);
    state
}

pub(crate) fn begin_turn(state: &mut CombatState) {
    let Some(id) = state.order.get(state.turn).cloned() else {
        return;
    };
    let Some(u) = state.units.iter_mut().find(|u| u.id == id) else {
        return;
    };
    u.ap = u.max_ap;
    u.guard = false;
    let name = u.name.clone();
    let trans = u.trans;
    let seed = state.seed;
    let round = state.round;
    let lost_turn = if trans >= 90 && u.side == Side::Player {
        let mut rng = Rng::new(seed + round as u32 * 17 + trans as u32);
        if rng.chance(0.25 + (trans - 90) as f32 / 80.0) {
            u.ap = 0;
            true
        } else {
            false
        }
    } else {
        false
    };
    drop(u);
    if lost_turn {
        push_log(
            state,
            "trans",
            format!("{} loses the bar. The turn is gone.", name),
        );
    }
    tick_ripples(state, &id);
}

pub(crate) fn tick_ripples(state: &mut CombatState, actor_id: &str) {
    let zones = std::mem::take(&mut state.zones);
    let mut keep = Vec::new();
    for z in zones {
        let ring: Vec<_> = hex_ring(z.center, z.radius)
            .into_iter()
            .filter(|h| in_bounds(*h, state.cols, state.rows))
            .collect();
        resolve::apply_zone(state, &ring, z.power, z.pa, Attr::A, actor_id, false, false);
        push_log(state, "info", format!("Ripple expands to {}.", z.radius));
        if z.radius < z.max_radius {
            keep.push(DelayedZone {
                radius: z.radius + 1,
                ..z
            });
        }
    }
    state.zones = keep;
}

pub(crate) fn advance_turn(state: &mut CombatState) {
    if state.over.is_some() {
        return;
    }
    check_over(state);
    if state.over.is_some() {
        return;
    }
    let mut guard = 0;
    loop {
        state.turn += 1;
        if state.turn >= state.order.len() {
            state.turn = 0;
            state.round += 1;
        }
        guard += 1;
        if current_unit(state).is_some() || guard > state.order.len() + 2 {
            break;
        }
    }
    begin_turn(state);
    check_over(state);
}

pub(crate) fn check_over(state: &mut CombatState) {
    if living(state, Some(Side::Enemy)).is_empty() {
        state.over = Some(true);
    }
    if living(state, Some(Side::Player)).is_empty() {
        state.over = Some(false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_table() {
        assert!((effect_scale(0, 0) - 1.0).abs() < 1e-5);
        assert!((effect_scale(4, 0) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn doga_starts() {
        let enc = catalog::encounter("doga-yoma").unwrap();
        let s = create_battle(enc, &["clare".into()], 7);
        assert!(s.units.len() >= 3);
        assert!(current_unit(&s).is_some());
    }
}
