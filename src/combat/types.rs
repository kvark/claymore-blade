//! Combat types.

use crate::hex::Axial;
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
