//! Turn-based hex combat. Native and wasm run this same code.

use crate::catalog::{self, derived, EncounterDef};
use crate::hex::{
    facing_toward, hex_cone, hex_disc, hex_distance, hex_eq, hex_line, hex_neighbors, hex_ring,
    hex_sweep, place_footprint, Axial,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attr {
    S,
    A,
    C,
    P,
    W,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeKind {
    SelfCast,
    Single,
    Line,
    Cone,
    Blast,
    Ring,
    Sweep,
    Ripple,
    Leap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitKind {
    Miss,
    Glance,
    Blocked,
    Solid,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub s: i32,
    pub a: i32,
    pub c: i32,
    pub p: i32,
    pub w: i32,
}

impl Stats {
    pub fn get(self, a: Attr) -> i32 {
        match a {
            Attr::S => self.s,
            Attr::A => self.a,
            Attr::C => self.c,
            Attr::P => self.p,
            Attr::W => self.w,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub blurb: &'static str,
    pub ap: i32,
    pub trans: i32,
    pub yoki: i32,
    pub shape: ShapeKind,
    pub range: i32,
    pub length: i32,
    pub pa: Attr,
    pub pd: Attr,
    pub power: i32,
    pub aimed: bool,
    pub self_cast: bool,
    pub heal: i32,
    pub trans_delta: i32,
    pub r#move: i32,
    pub guard: i32,
    pub telegraph: bool,
    pub afterimage: bool,
    pub unblockable: bool,
    pub strikes: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct UnitTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub rank: i32,
    pub side: Side,
    pub portrait: &'static str,
    pub sprite: &'static str,
    pub stats: Stats,
    pub skills: &'static [&'static str],
    pub trans: i32,
    pub footprint: &'static [Axial],
    pub core_index: usize,
    pub parts: &'static [(&'static str, &'static str, usize, i32, Option<&'static str>)],
    pub body_hp: i32,
    pub color: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    pub name: String,
    pub hex_index: usize,
    pub hp: i32,
    pub max_hp: i32,
    pub zone: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub id: String,
    pub name: String,
    pub turns: i32,
    pub guard: i32,
    pub telegraph: bool,
    pub afterimage: Option<Axial>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub template_id: String,
    pub name: String,
    pub title: String,
    pub rank: i32,
    pub side: Side,
    pub portrait: String,
    pub sprite: String,
    pub origin: Axial,
    pub facing: i32,
    pub footprint: Vec<Axial>,
    pub core_index: usize,
    pub parts: Vec<Part>,
    pub hp: i32,
    pub max_hp: i32,
    pub yoki: i32,
    pub max_yoki: i32,
    pub trans: i32,
    pub ap: i32,
    pub max_ap: i32,
    pub stats: Stats,
    pub skills: Vec<String>,
    pub statuses: Vec<Status>,
    pub raised_trans: bool,
    pub next_hint: Option<String>,
    pub color: [f32; 3],
    pub dead: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DelayedZone {
    pub source_id: String,
    pub center: Axial,
    pub radius: i32,
    pub max_radius: i32,
    pub power: i32,
    pub pa: Attr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatLog {
    pub text: String,
    pub kind: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Mud,
    Ruin,
    Water,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatState {
    pub id: String,
    pub title: String,
    pub seed: u32,
    pub turn: usize,
    pub round: i32,
    pub order: Vec<String>,
    pub units: Vec<Unit>,
    pub terrain: Vec<(Axial, Terrain)>,
    pub cols: i32,
    pub rows: i32,
    pub zones: Vec<DelayedZone>,
    pub log: Vec<CombatLog>,
    pub over: Option<bool>,
    pub briefing: String,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    Move(Axial),
    Skill { id: String, hex: Axial },
    Raise,
    Wait,
}

pub fn effect_scale(pa: i32, pd: i32) -> f32 {
    let d = (pa - pd).clamp(-4, 8);
    1.0 + 0.25 * d as f32
}

fn in_bounds(h: Axial, cols: i32, rows: i32) -> bool {
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

fn terrain_at(state: &CombatState, h: Axial) -> Terrain {
    state
        .terrain
        .iter()
        .find(|(a, _)| hex_eq(*a, h))
        .map(|(_, t)| *t)
        .unwrap_or(Terrain::Grass)
}

fn occupied(state: &CombatState, ignore: Option<&str>) -> Vec<(Axial, String)> {
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

fn spawn(t: &UnitTemplate, id: String, origin: Axial, facing: i32) -> Unit {
    let d = derived(&t.stats);
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
        max_yoki: d.yoki,
        trans: t.trans,
        ap: 2,
        max_ap: 2,
        stats: t.stats,
        skills: t.skills.iter().map(|s| (*s).into()).collect(),
        statuses: Vec::new(),
        raised_trans: false,
        next_hint: None,
        color: t.color,
        dead: false,
    }
}

fn push_log(state: &mut CombatState, kind: &str, text: String) {
    state.log.insert(
        0,
        CombatLog {
            text,
            kind: kind.into(),
        },
    );
    if state.log.len() > 40 {
        state.log.pop();
    }
}

pub fn create_battle(enc: &EncounterDef, party: &[String], seed: u32) -> CombatState {
    let mut units = Vec::new();
    for (i, pid) in party.iter().take(3).enumerate() {
        if let Some(t) = catalog::warrior(pid) {
            let origin = *enc.player_origins.get(i).unwrap_or(&enc.player_origins[0]);
            units.push(spawn(t, format!("p-{}", t.id), origin, 0));
        }
    }
    for (i, e) in enc.enemies.iter().enumerate() {
        if let Some(t) = catalog::enemy(e.template) {
            units.push(spawn(t, format!("e-{}-{i}", t.id), e.origin, e.facing));
        }
    }
    let mut rng = Rng::new(seed);
    let mut terrain = Vec::new();
    for q in 0..enc.cols {
        for r in 0..enc.rows {
            let roll = rng.next_f32();
            let t = if roll > 0.92 {
                Terrain::Ruin
            } else if roll > 0.84 {
                Terrain::Mud
            } else if roll < 0.04 {
                Terrain::Water
            } else {
                Terrain::Grass
            };
            terrain.push((Axial::new(q, r), t));
        }
    }
    for u in &units {
        for c in live_cells(u) {
            if let Some((_, t)) = terrain.iter_mut().find(|(h, _)| hex_eq(*h, c)) {
                *t = Terrain::Grass;
            }
        }
    }
    let mut keyed: Vec<_> = units
        .iter()
        .map(|u| (u.stats.a + rng.int(0, 9), u.id.clone()))
        .collect();
    keyed.sort_by(|a, b| b.0.cmp(&a.0));
    let order: Vec<String> = keyed.into_iter().map(|(_, id)| id).collect();
    let mut state = CombatState {
        id: enc.id.into(),
        title: enc.title.into(),
        seed,
        turn: 0,
        round: 1,
        order,
        units,
        terrain,
        cols: enc.cols,
        rows: enc.rows,
        zones: Vec::new(),
        log: Vec::new(),
        over: None,
        briefing: enc.briefing.into(),
    };
    let enemies = state.units.iter().filter(|u| u.side == Side::Enemy).count();
    push_log(
        &mut state,
        "info",
        format!("{}. {enemies} on the board.", enc.title),
    );
    begin_turn(&mut state);
    state
}

pub fn current_unit(state: &CombatState) -> Option<&Unit> {
    let id = state.order.get(state.turn)?;
    state.units.iter().find(|u| u.id == *id && !u.dead)
}

fn current_unit_mut(state: &mut CombatState) -> Option<&mut Unit> {
    let id = state.order.get(state.turn)?.clone();
    state.units.iter_mut().find(|u| u.id == id && !u.dead)
}

pub fn living<'a>(state: &'a CombatState, side: Option<Side>) -> Vec<&'a Unit> {
    state
        .units
        .iter()
        .filter(|u| !u.dead && side.map(|s| u.side == s).unwrap_or(true))
        .collect()
}

fn begin_turn(state: &mut CombatState) {
    let Some(id) = state.order.get(state.turn).cloned() else {
        advance_turn(state);
        return;
    };
    let round = state.round;
    let seed = state.seed;
    let trans;
    {
        let Some(u) = state.units.iter_mut().find(|u| u.id == id && !u.dead) else {
            advance_turn(state);
            return;
        };
        u.ap = u.max_ap;
        u.raised_trans = false;
        u.yoki = (u.yoki + 2).min(u.max_yoki);
        u.statuses.retain_mut(|s| {
            s.turns -= 1;
            s.turns > 0
        });
        trans = u.trans;
        if trans >= 90 && u.side == Side::Player {
            let mut rng = Rng::new(seed + round as u32 * 17 + trans as u32);
            if rng.chance(0.25 + (trans - 90) as f32 / 80.0) {
                u.ap = 0;
            }
        }
    }
    if trans >= 90 && current_unit(state).map(|u| u.ap == 0).unwrap_or(false) {
        if let Some(u) = current_unit(state) {
            push_log(
                state,
                "trans",
                format!("{} loses the bar. The turn is gone.", u.name),
            );
        }
    }
    tick_ripples(state, &id);
}

fn tick_ripples(state: &mut CombatState, actor_id: &str) {
    let zones = std::mem::take(&mut state.zones);
    let mut keep = Vec::new();
    for z in zones {
        let ring: Vec<_> = hex_ring(z.center, z.radius)
            .into_iter()
            .filter(|h| in_bounds(*h, state.cols, state.rows))
            .collect();
        apply_zone(state, &ring, z.power, z.pa, Attr::A, actor_id, false, false);
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

fn advance_turn(state: &mut CombatState) {
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

fn check_over(state: &mut CombatState) {
    if living(state, Some(Side::Enemy)).is_empty() {
        state.over = Some(true);
    }
    if living(state, Some(Side::Player)).is_empty() {
        state.over = Some(false);
    }
}

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

fn move_cost(state: &CombatState, hex: Axial) -> i32 {
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

fn resolve_hit(
    rng: &mut Rng,
    atk: &Unit,
    def: &Unit,
    skill: &SkillDef,
    cover: bool,
) -> (HitKind, f32) {
    let pa = atk.stats.get(skill.pa);
    let pd = def.stats.get(skill.pd);
    let scale = effect_scale(pa, pd);
    let hit = derived(&atk.stats).hit + if skill.aimed { -4 } else { 0 };
    let dodge = derived(&def.stats).dodge;
    let chance = 0.55 + (hit - dodge) as f32 * 0.03;
    let roll = rng.next_f32();
    if skill.unblockable {
        return (HitKind::Solid, scale);
    }
    if roll > chance + 0.15 {
        return (HitKind::Miss, scale);
    }
    if roll > chance {
        return (HitKind::Glance, scale);
    }
    let guarded = def.statuses.iter().any(|s| s.guard > 0) || cover;
    if guarded && rng.chance(0.55) {
        return (HitKind::Blocked, scale);
    }
    (HitKind::Solid, scale)
}

fn apply_damage(
    state: &mut CombatState,
    target_id: &str,
    amount: i32,
    zone: &[Axial],
    aimed: bool,
) {
    let Some(target) = state.units.iter_mut().find(|u| u.id == target_id) else {
        return;
    };
    let placed = place_footprint(&target.footprint, target.origin, target.facing);
    let mut hit_idx = Vec::new();
    for (i, h) in placed.iter().enumerate() {
        let alive = target
            .parts
            .iter()
            .find(|p| p.hex_index == i)
            .map(|p| p.hp > 0)
            .unwrap_or(true);
        if alive && zone.iter().any(|z| hex_eq(*z, *h)) {
            hit_idx.push(i);
        }
    }
    if hit_idx.is_empty() {
        return;
    }
    let core_hit = hit_idx.contains(&target.core_index);
    let mut dmg = ((amount as f32) * if core_hit { 1.0 } else { 0.5 }).round() as i32;
    if dmg < 1 {
        dmg = 1;
    }
    let focus = if aimed {
        if hit_idx.contains(&target.core_index) {
            Some(target.core_index)
        } else {
            hit_idx.first().copied()
        }
    } else {
        None
    };
    let nparts = target.parts.len();
    let tname = target.name.clone();
    let mut severed: Vec<String> = Vec::new();
    if let Some(focus) = focus {
        if let Some(part) = target.parts.iter_mut().find(|p| p.hex_index == focus) {
            part.hp = (part.hp - dmg).max(0);
            if part.hp == 0 && nparts > 1 {
                severed.push(format!("{}'s {} is carved off.", tname, part.name));
                if let Some(z) = part.zone.clone() {
                    target.skills.retain(|s| s != &z);
                }
            }
        }
    } else {
        let share = (dmg / hit_idx.len() as i32).max(1);
        for i in hit_idx {
            if let Some(part) = target.parts.iter_mut().find(|p| p.hex_index == i) {
                part.hp = (part.hp - share).max(0);
                if part.hp == 0 && nparts > 1 {
                    severed.push(format!("{}'s {} is carved off.", tname, part.name));
                }
            }
        }
    }
    target.hp = target.parts.iter().map(|p| p.hp).sum();
    let died = target.hp <= 0;
    let name = target.name.clone();
    if died {
        target.dead = true;
        target.hp = 0;
    }
    for s in severed {
        push_log(state, "sever", s);
    }
    if died {
        push_log(state, "death", format!("{name} falls."));
    }
}

fn apply_zone(
    state: &mut CombatState,
    zone: &[Axial],
    power: i32,
    pa: Attr,
    pd: Attr,
    attacker_id: &str,
    aimed: bool,
    unblockable: bool,
) {
    let atk = match state.units.iter().find(|u| u.id == attacker_id).cloned() {
        Some(u) => u,
        None => return,
    };
    let mut rng =
        Rng::new(state.seed + state.round as u32 * 31 + state.turn as u32 * 7 + power as u32);
    let dummy = SkillDef {
        id: "hit",
        name: "Hit",
        blurb: "",
        ap: 0,
        trans: 0,
        yoki: 0,
        shape: ShapeKind::Single,
        range: 1,
        length: 1,
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
        strikes: false,
    };
    let mut hit_ids = Vec::new();
    for h in zone {
        for u in &state.units {
            if u.dead || u.id == attacker_id {
                continue;
            }
            if live_cells(u).iter().any(|c| hex_eq(*c, *h)) && !hit_ids.contains(&u.id) {
                hit_ids.push(u.id.clone());
            }
        }
    }
    for id in hit_ids {
        let def = match state.units.iter().find(|u| u.id == id).cloned() {
            Some(u) => u,
            None => continue,
        };
        let cover = live_cells(&def)
            .iter()
            .any(|c| zone.iter().any(|z| hex_eq(*z, *c)) && terrain_at(state, *c) == Terrain::Ruin);
        let (kind, scale) = resolve_hit(&mut rng, &atk, &def, &dummy, cover);
        let trans_mul = 1.0 + atk.trans as f32 / 200.0;
        let base = power as f32 * scale * trans_mul;
        let dmg = match kind {
            HitKind::Miss => 0,
            HitKind::Glance => (base * 0.2).round() as i32,
            HitKind::Blocked => (base * rng.next_f32() * 0.3).round() as i32,
            HitKind::Solid => base.round() as i32,
        };
        match kind {
            HitKind::Miss => push_log(state, "miss", format!("{} misses {}.", atk.name, def.name)),
            HitKind::Blocked => push_log(
                state,
                "hit",
                format!("{} catches {}'s blow.", def.name, atk.name),
            ),
            _ => push_log(
                state,
                "hit",
                format!("{} → {}: {:?} {dmg}", atk.name, def.name, kind),
            ),
        }
        if dmg > 0 {
            apply_damage(state, &id, dmg, zone, aimed);
        }
    }
}

pub fn act(state: &mut CombatState, action: PlayerAction, has_raki: bool) {
    if state.over.is_some() {
        return;
    }
    let Some(actor_id) = current_unit(state).map(|u| u.id.clone()) else {
        return;
    };
    match action {
        PlayerAction::Raise => {
            let Some(actor) = current_unit_mut(state) else {
                return;
            };
            if actor.raised_trans {
                return;
            }
            actor.trans = (actor.trans + 16).min(100);
            actor.raised_trans = true;
            let name = actor.name.clone();
            let trans = actor.trans;
            push_log(state, "trans", format!("{name} opens the bar ({trans})."));
            if trans >= 100 {
                push_log(state, "trans", format!("{name} is at the edge."));
            }
        }
        PlayerAction::Wait => {
            let Some(actor) = current_unit_mut(state) else {
                return;
            };
            actor.ap = 0;
            actor.trans = (actor.trans - 4).max(0);
            let name = actor.name.clone();
            push_log(state, "info", format!("{name} waits."));
            advance_turn(state);
        }
        PlayerAction::Move(hex) => {
            let moves = legal_moves(state, &actor_id);
            if !moves.iter().any(|h| hex_eq(*h, hex)) {
                return;
            }
            let Some(actor) = current_unit_mut(state) else {
                return;
            };
            let from = core_hex(actor);
            let cost = actor.ap.min(hex_distance(from, hex).max(1));
            actor.facing = facing_toward(from, hex);
            actor.origin = Axial::new(
                actor.origin.q + hex.q - from.q,
                actor.origin.r + hex.r - from.r,
            );
            actor.ap = (actor.ap - cost.max(1)).max(0);
            let name = actor.name.clone();
            let ap = actor.ap;
            push_log(state, "info", format!("{name} steps."));
            if ap <= 0 {
                advance_turn(state);
            }
        }
        PlayerAction::Skill { id, hex } => {
            let Some(skill) = catalog::skill(&id).cloned() else {
                return;
            };
            {
                let Some(actor) = current_unit(state) else {
                    return;
                };
                if !can_use(actor, &skill, has_raki) {
                    return;
                }
                if !legal_targets(state, &actor_id, skill.id)
                    .iter()
                    .any(|h| hex_eq(*h, hex))
                {
                    return;
                }
            }
            let zone = {
                let actor = current_unit(state).unwrap();
                zone_for(state, actor, &skill, hex)
            };
            let (from, name, strikes, leap_to) = {
                let actor = current_unit_mut(state).unwrap();
                let from = core_hex(actor);
                actor.facing = facing_toward(from, hex);
                actor.ap -= skill.ap;
                actor.yoki -= skill.yoki;
                if skill.trans_delta != 0 {
                    actor.trans = (actor.trans + skill.trans_delta).clamp(0, 100);
                }
                if skill.heal > 0 {
                    let n = actor.parts.len().max(1) as i32;
                    let share = skill.heal / n;
                    for p in &mut actor.parts {
                        if p.hp > 0 {
                            p.hp = (p.hp + share).min(p.max_hp);
                        } else if skill.id == "regen" {
                            p.hp = (p.max_hp as f32 * 0.4).round() as i32;
                        }
                    }
                    actor.hp = actor.parts.iter().map(|p| p.hp).sum();
                }
                if skill.guard > 0 {
                    actor.statuses.push(Status {
                        id: "guard".into(),
                        name: "Guard".into(),
                        turns: 2,
                        guard: skill.guard,
                        telegraph: false,
                        afterimage: None,
                    });
                }
                if skill.telegraph {
                    actor.statuses.push(Status {
                        id: "read".into(),
                        name: "Read Energy".into(),
                        turns: 3,
                        guard: 0,
                        telegraph: true,
                        afterimage: None,
                    });
                }
                let name = actor.name.clone();
                let actor_id_copy = actor.id.clone();
                let strikes = if skill.strikes {
                    derived(&actor.stats).strikes
                } else {
                    1
                };
                let do_leap = skill.shape == ShapeKind::Leap && skill.r#move > 0;
                let after = skill.afterimage;
                (
                    from,
                    name,
                    strikes,
                    do_leap.then_some((actor_id_copy, hex, after)),
                )
            };
            if let Some((id, dest, after)) = leap_to {
                let occ = occupied(state, Some(&id));
                if !occ.iter().any(|(h, _)| hex_eq(*h, dest)) {
                    if let Some(actor) = state.units.iter_mut().find(|u| u.id == id) {
                        if after {
                            actor.statuses.push(Status {
                                id: "after".into(),
                                name: "Afterimage".into(),
                                turns: 2,
                                guard: 0,
                                telegraph: false,
                                afterimage: Some(from),
                            });
                        }
                        actor.origin = dest;
                    }
                }
            }
            if skill.heal > 0 {
                push_log(state, "info", format!("{name} knits flesh."));
            }
            if skill.telegraph {
                push_log(state, "info", format!("{name} reads the field."));
            }
            if skill.shape == ShapeKind::Leap {
                push_log(state, "info", format!("{name} is already gone."));
            }
            if skill.shape == ShapeKind::Ripple {
                state.zones.push(DelayedZone {
                    source_id: actor_id.clone(),
                    center: from,
                    radius: 1,
                    max_radius: 3,
                    power: skill.power,
                    pa: skill.pa,
                });
                push_log(state, "info", format!("{name} starts a ripple."));
            } else if skill.power > 0 {
                for _ in 0..strikes {
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

fn pick_ai_skill(_state: &CombatState, u: &Unit) -> Option<&'static SkillDef> {
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
