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

fn spawn(t: &UnitTemplate, id: String, origin: Axial, facing: i32) -> Unit {
    let parts: Vec<Part> = t
        .parts
        .iter()
        .enumerate()
        .map(|(i, p)| Part {
            name: p.name.into(),
            hex_index: i,
            hp: p.hp,
            max_hp: p.hp,
        })
        .collect();
    let stats = derived(t);
    Unit {
        id,
        name: t.name.into(),
        side: if t.side == "player" {
            Side::Player
        } else {
            Side::Enemy
        },
        origin,
        facing,
        footprint: t.footprint.to_vec(),
        core_index: t.core_index,
        parts,
        stats,
        ap: max_ap(&stats),
        max_ap: max_ap(&stats),
        skills: t.skills.iter().map(|s| (*s).into()).collect(),
        color: t.color,
        portrait: t.portrait.into(),
        dead: false,
        guard: false,
        trans: 0,
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

    // initiative: higher A first, then seed noise
    order.sort_by_key(|id| {
        let u = units.iter().find(|x| x.id == *id).unwrap();
        let noise = rng.int(0, 6);
        (-u.stats.a * 10 - noise, id.clone())
    });

    let mut terrain = Vec::new();
    // Gorky-style arena: mud patches, ruin cover, stagnant water.
    let occupied = |terrain: &[(Axial, Terrain)], h: Axial| {
        terrain.iter().any(|(a, _)| hex_eq(*a, h))
    };
    let mud_n = ((cols * rows) / 5).max(8);
    for _ in 0..mud_n {
        let h = Axial::new(rng.int(0, cols - 1), rng.int(0, rows - 1));
        if !occupied(&terrain, h) {
            terrain.push((h, Terrain::Mud));
        }
    }
    let ruin_n = ((cols + rows) / 3).max(3);
    for _ in 0..ruin_n {
        let h = Axial::new(rng.int(0, cols - 1), rng.int(0, rows - 1));
        if !occupied(&terrain, h) {
            terrain.push((h, Terrain::Ruin));
        }
    }
    // Water seam near the mid-line for cover / chokepoints.
    let mid_r = rows / 2;
    for q in 0..cols {
        let h0 = Axial::new(q, mid_r);
        if rng.int(0, 100) < 35 && !occupied(&terrain, h0) {
            terrain.push((h0, Terrain::Water));
        }
        let h1 = Axial::new(q, (mid_r + 1).min(rows - 1));
        if rng.int(0, 100) < 20 && !occupied(&terrain, h1) {
            terrain.push((h1, Terrain::Water));
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
