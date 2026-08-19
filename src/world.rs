use crate::catalog::{self, LocationDef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldStatus {
    Quiet,
    Beacon,
    Dead,
    Cleared,
    Locked,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocState {
    pub status: WorldStatus,
    pub hours_left: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub demons: i32,
    pub awakened: i32,
    pub missions: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldState {
    pub hours: f32,
    pub party_x: f32,
    pub party_y: f32,
    pub party: Vec<String>,
    pub raki: bool,
    pub rank: i32,
    pub karma: i32,
    pub ledger: Ledger,
    pub locations: HashMap<String, LocState>,
    pub flags: HashMap<String, bool>,
    pub last_town: Option<String>,
}

pub fn new_world() -> WorldState {
    let mut locations = HashMap::new();
    for loc in catalog::LOCATIONS {
        let status = match loc.id {
            "doga" => WorldStatus::Beacon,
            "paburo" | "gonal" | "pieta" => WorldStatus::Locked,
            _ => WorldStatus::Quiet,
        };
        locations.insert(
            loc.id.into(),
            LocState {
                status,
                hours_left: loc.deadline as f32,
            },
        );
    }
    WorldState {
        hours: 6.0,
        party_x: 0.26,
        party_y: 0.56,
        party: vec!["clare".into()],
        raki: false,
        rank: 47,
        karma: 0,
        ledger: Ledger {
            demons: 0,
            awakened: 0,
            missions: 0,
        },
        locations,
        flags: HashMap::new(),
        last_town: None,
    }
}

pub fn dist01(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = ax - bx;
    let dy = ay - by;
    (dx * dx + dy * dy).sqrt()
}

pub fn tick_hours(world: &mut WorldState, hours: f32) {
    world.hours += hours;
    for loc in catalog::LOCATIONS {
        if let Some(st) = world.locations.get_mut(loc.id) {
            if st.status == WorldStatus::Beacon {
                st.hours_left = (st.hours_left - hours).max(0.0);
                if st.hours_left == 0.0 {
                    st.status = WorldStatus::Dead;
                    world.karma -= 12;
                }
            }
        }
    }
}

pub fn apply_victory(world: &mut WorldState, encounter_id: &str) {
    let Some(enc) = catalog::encounter(encounter_id) else {
        return;
    };
    if let Some(loc) = catalog::LOCATIONS
        .iter()
        .find(|l| l.encounter == Some(encounter_id))
    {
        world.locations.insert(
            loc.id.into(),
            LocState {
                status: WorldStatus::Cleared,
                hours_left: 0.0,
            },
        );
    }
    world.ledger.missions += 1;
    if encounter_id.contains("ripple") || encounter_id.contains("worm") {
        world.ledger.awakened += 1;
    } else {
        world.ledger.demons += if encounter_id == "paburo-nest" { 3 } else { 2 };
    }
    world.karma += enc.karma;
    world.rank = (world.rank + enc.rank).max(1);
    // Raki join and silver recruits are scene choices, not automatic.
    world.flags.insert(enc.flag.into(), true);
    if world.flags.get("doga-cleared") == Some(&true) {
        if let Some(st) = world.locations.get_mut("paburo") {
            if st.status == WorldStatus::Locked {
                *st = LocState {
                    status: WorldStatus::Beacon,
                    hours_left: 72.0,
                };
            }
        }
    }
    if world.flags.get("paburo-cleared") == Some(&true) {
        if let Some(st) = world.locations.get_mut("gonal") {
            if st.status == WorldStatus::Locked {
                *st = LocState {
                    status: WorldStatus::Beacon,
                    hours_left: 90.0,
                };
            }
        }
    }
    if world.flags.get("gonal-cleared") == Some(&true) {
        if let Some(st) = world.locations.get_mut("pieta") {
            if st.status == WorldStatus::Locked {
                *st = LocState {
                    status: WorldStatus::Beacon,
                    hours_left: 110.0,
                };
            }
        }
    }
}

pub fn nearest_location(x: f32, y: f32, radius: f32) -> Option<&'static LocationDef> {
    let mut best = None;
    let mut best_d = radius;
    for loc in catalog::LOCATIONS {
        let d = dist01(x, y, loc.x, loc.y);
        if d < best_d {
            best_d = d;
            best = Some(loc);
        }
    }
    best
}

pub fn clock_label(hours: f32) -> String {
    let day = (hours / 24.0).floor() as i32 + 1;
    let h = (hours as i32) % 24;
    format!("Day {day} {h:02}:00")
}
