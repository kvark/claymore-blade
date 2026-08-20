//! Turn-based hex combat. Native and wasm run this same code.

use crate::catalog::{self, derived, EncounterDef};
use crate::hex::{
    facing_toward, hex_cone, hex_disc, hex_distance, hex_eq, hex_line, hex_neighbors, hex_ring,
    hex_sweep, Axial,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Player,
    Enemy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    Grass,
    Mud,
    Ruin,
    Water,
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
    Single,
    Line,
    Cone,
    Blast,
    Ring,
    Sweep,
    Ripple,
    Cross,
    SelfCast,
    Leap,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub s: i32,
    pub a: i32,
    pub c: i32,
    pub p: i32,
    pub w: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub ap: i32,
    pub trans: i32,
    pub shape: ShapeKind,
    pub range: i32,
    pub power: i32,
    pub pa: Attr,
    pub pd: Attr,
    pub aimed: bool,
    pub unblockable: bool,
    pub self_cast: bool,
    pub heal: i32,
    pub telegraph: bool,
    pub r#move: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub side: Side,
    pub stats: Stats,
    pub max_hp: i32,
    pub skills: &'static [&'static str],
    pub color: [f32; 3],
    pub portrait: &'static str,
    pub sprite: &'static str,
    pub footprint: &'static [Axial],
    pub is_awakened: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Part {
    pub name: String,
    pub offset: Axial,
    pub hp: i32,
    pub max_hp: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub name: String,
    pub side: Side,
    pub origin: Axial,
    pub facing: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub ap: i32,
    pub max_ap: i32,
    pub trans: i32,
    pub yoki: i32,
    pub stats: Stats,
    pub skills: Vec<String>,
    pub color: [f32; 3],
    pub portrait: String,
    pub sprite: String,
    pub parts: Vec<Part>,
    pub raised_trans: bool,
    pub dead: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatLog {
    pub text: String,
    pub kind: String,
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
pub struct CombatState {
    pub id: String,
    pub title: String,
    pub cols: i32,
    pub rows: i32,
    pub units: Vec<Unit>,
    pub terrain: Vec<(Axial, Terrain)>,
    pub turn: usize,
    pub log: Vec<CombatLog>,
    pub zones: Vec<DelayedZone>,
    pub over: Option<bool>,
    pub seed: u32,
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    Wait,
    Raise,
    Move(Axial),
    Skill { id: String, hex: Axial },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HitKind {
    Miss,
    Glance,
    Blocked,
    Solid,
}

fn attr_of(u: &Unit, a: Attr) -> i32 {
    match a {
        Attr::S => u.stats.s,
        Attr::A => u.stats.a,
        Attr::C => u.stats.c,
        Attr::P => u.stats.p,
        Attr::W => u.stats.w,
    }
}

fn terrain_at(state: &CombatState, hex: Axial) -> Terrain {
    state
        .terrain
        .iter()
        .find(|(h, _)| hex_eq(*h, hex))
        .map(|(_, t)| *t)
        .unwrap_or(Terrain::Grass)
}

pub fn core_hex(u: &Unit) -> Axial {
    u.origin
}

pub fn live_cells(u: &Unit) -> Vec<Axial> {
    if u.parts.is_empty() {
        return vec![u.origin];
    }
    u.parts
        .iter()
        .filter(|p| p.hp > 0)
        .map(|p| Axial {
            q: u.origin.q + p.offset.q,
            r: u.origin.r + p.offset.r,
        })
        .collect()
}

pub fn current_unit(state: &CombatState) -> Option<&Unit> {
    state.units.get(state.turn % state.units.len().max(1))
}

fn current_unit_mut(state: &mut CombatState) -> Option<&mut Unit> {
    let i = state.turn % state.units.len().max(1);
    state.units.get_mut(i)
}

fn push_log(state: &mut CombatState, kind: &str, text: String) {
    let text = crate::dialog::bark(kind)
        .map(|b| b.to_string())
        .unwrap_or(text);
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

// NOTE: truncated for tool size — restoring full file in follow-up if needed
