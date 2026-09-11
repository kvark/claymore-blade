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

/// Coarse fog-of-war grid (map space 0..1).
pub const EXPLORED_W: usize = 64;
pub const EXPLORED_H: usize = 48;
/// Vision radius stamped around the party while on the island.
pub const VISION_RADIUS: f32 = 0.07;
/// Must be this close (normalized map space) to enter a pin.
pub const TOWN_ENTER_RADIUS: f32 = 0.05;

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
    /// Bit rows: `explored[row] & (1 << col)` means the cell is revealed.
    #[serde(default = "empty_explored")]
    pub explored: Vec<u64>,
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
    let mut world = WorldState {
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
        explored: empty_explored(),
    };
    // Spawn + Doga start revealed so the first pin is visible.
    let (px, py) = (world.party_x, world.party_y);
    stamp_explored(&mut world, px, py, VISION_RADIUS + 0.01);
    if let Some(doga) = catalog::location("doga") {
        stamp_explored(&mut world, doga.x, doga.y, VISION_RADIUS + 0.01);
    }
    world
}


pub fn empty_explored() -> Vec<u64> {
    vec![0u64; EXPLORED_H]
}

pub fn ensure_explored(world: &mut WorldState) {
    if world.explored.len() != EXPLORED_H {
        world.explored = empty_explored();
        let (px, py) = (world.party_x, world.party_y);
        stamp_explored(world, px, py, VISION_RADIUS + 0.01);
        if let Some(doga) = catalog::location("doga") {
            stamp_explored(world, doga.x, doga.y, VISION_RADIUS + 0.01);
        }
    }
}

pub fn cell_of(x: f32, y: f32) -> (usize, usize) {
    let c = (x.clamp(0.0, 0.999) * EXPLORED_W as f32) as usize;
    let r = (y.clamp(0.0, 0.999) * EXPLORED_H as f32) as usize;
    (c.min(EXPLORED_W - 1), r.min(EXPLORED_H - 1))
}

pub fn is_explored(world: &WorldState, col: usize, row: usize) -> bool {
    world
        .explored
        .get(row)
        .map(|bits| bits & (1u64 << col) != 0)
        .unwrap_or(false)
}

pub fn map_explored_at(world: &WorldState, x: f32, y: f32) -> bool {
    let (c, r) = cell_of(x, y);
    is_explored(world, c, r)
}

/// Reveal cells within `radius` (normalized map space) of `(x, y)`.
pub fn stamp_explored(world: &mut WorldState, x: f32, y: f32, radius: f32) {
    if world.explored.len() != EXPLORED_H {
        world.explored = empty_explored();
    }
    let (cx, cy) = cell_of(x, y);
    let rc = (radius * EXPLORED_W as f32).ceil() as i32 + 1;
    let rr = (radius * EXPLORED_H as f32).ceil() as i32 + 1;
    for dy in -rr..=rr {
        for dx in -rc..=rc {
            let c = cx as i32 + dx;
            let r = cy as i32 + dy;
            if c < 0 || r < 0 || c >= EXPLORED_W as i32 || r >= EXPLORED_H as i32 {
                continue;
            }
            let mx = (c as f32 + 0.5) / EXPLORED_W as f32;
            let my = (r as f32 + 0.5) / EXPLORED_H as f32;
            if dist01(x, y, mx, my) <= radius {
                if let Some(bits) = world.explored.get_mut(r as usize) {
                    *bits |= 1u64 << (c as usize);
                }
            }
        }
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
    if let Some(loc) = catalog::LOCATIONS.iter().find(|l| {
        l.encounter == Some(encounter_id) || (encounter_id == "doga-nest" && l.id == "doga")
    }) {
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
        world.ledger.demons += if encounter_id.contains("nest") { 3 } else { 2 };
    }
    world.karma += enc.karma;
    world.rank = (world.rank + enc.rank).max(1);
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

/// Hunt id for a pin. Dead beacons become nests (harder, no villagers).
pub fn hunt_for(world: &WorldState, loc_id: &str) -> Option<&'static str> {
    let loc = catalog::location(loc_id)?;
    let dead = world
        .locations
        .get(loc_id)
        .map(|s| s.status == WorldStatus::Dead)
        .unwrap_or(false);
    if dead {
        match loc.id {
            "doga" => Some("doga-nest"),
            _ => loc.encounter,
        }
    } else {
        loc.encounter
    }
}

pub fn clock_label(hours: f32) -> String {
    let day = (hours / 24.0).floor() as i32 + 1;
    let h = (hours as i32) % 24;
    format!("Day {day} {h:02}:00")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_beacon_becomes_a_nest() {
        let mut w = new_world();
        if let Some(st) = w.locations.get_mut("doga") {
            st.hours_left = 1.0;
        }
        tick_hours(&mut w, 2.0);
        assert_eq!(w.locations["doga"].status, WorldStatus::Dead);
        assert_eq!(hunt_for(&w, "doga"), Some("doga-nest"));
    }

    #[test]
    fn nest_win_clears_doga() {
        let mut w = new_world();
        apply_victory(&mut w, "doga-nest");
        assert_eq!(w.locations["doga"].status, WorldStatus::Cleared);
        assert_eq!(w.flags.get("doga-cleared"), Some(&true));
        assert_eq!(w.locations["paburo"].status, WorldStatus::Beacon);
    }

    #[test]
    fn new_world_reveals_spawn_and_doga() {
        let w = new_world();
        assert!(map_explored_at(&w, w.party_x, w.party_y));
        let doga = catalog::location("doga").unwrap();
        assert!(map_explored_at(&w, doga.x, doga.y));
        // Far corner stays shrouded.
        assert!(!map_explored_at(&w, 0.95, 0.08));
    }

    #[test]
    fn stamp_explored_opens_a_disc() {
        let mut w = new_world();
        w.explored = empty_explored();
        stamp_explored(&mut w, 0.5, 0.5, 0.06);
        assert!(map_explored_at(&w, 0.5, 0.5));
        assert!(map_explored_at(&w, 0.53, 0.5));
        assert!(!map_explored_at(&w, 0.8, 0.8));
    }

    #[test]
    fn town_enter_radius_is_tight() {
        assert!(TOWN_ENTER_RADIUS <= 0.06);
        let doga = catalog::location("doga").unwrap();
        // Spawn is next to Doga but not on top of it.
        let w = new_world();
        let d = dist01(w.party_x, w.party_y, doga.x, doga.y);
        assert!(d < TOWN_ENTER_RADIUS + 0.02, "spawn should be near Doga ({d})");
    }
}
