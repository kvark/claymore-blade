import { ENCOUNTERS, LOCATIONS } from "@/game/data/catalog";
import type { WorldState, WorldStatus } from "./types";

export function newWorld(): WorldState {
  const locations: WorldState["locations"] = {};
  for (const loc of LOCATIONS) {
    let status: WorldStatus = "quiet";
    if (loc.id === "doga") status = "beacon";
    else if (loc.id === "paburo" || loc.id === "gonal" || loc.id === "pieta")
      status = "locked";
    else if (loc.id === "maw") status = "quiet";
    locations[loc.id] = { status, hoursLeft: loc.deadline ?? 0 };
  }
  return {
    hours: 6,
    partyX: 0.28,
    partyY: 0.54,
    party: ["clare"],
    raku: false,
    rank: 47,
    karma: 0,
    authority: 40,
    ledger: { demons: 0, awakened: 0, silvers: 0, humans: 0, missions: 0 },
    locations,
    flags: {},
    lastTown: undefined,
  };
}

export function dist01(ax: number, ay: number, bx: number, by: number) {
  const dx = ax - bx;
  const dy = ay - by;
  return Math.hypot(dx, dy);
}

export function hoursForTravel(a: { x: number; y: number }, b: { x: number; y: number }) {
  return Math.max(3, Math.round(dist01(a.x, a.y, b.x, b.y) * 48));
}

export function tickHours(world: WorldState, hours: number): WorldState {
  const next: WorldState = structuredClone(world);
  next.hours += hours;
  for (const loc of LOCATIONS) {
    const st = next.locations[loc.id];
    if (!st) continue;
    if (st.status === "beacon") {
      st.hoursLeft = Math.max(0, st.hoursLeft - hours);
      if (st.hoursLeft === 0) {
        st.status = "dead";
        next.karma -= 12;
      }
    }
  }
  return next;
}

export function applyVictory(world: WorldState, encounterId: string): WorldState {
  const enc = ENCOUNTERS[encounterId];
  const next: WorldState = structuredClone(world);
  if (!enc) return next;
  const loc = LOCATIONS.find((l) => l.encounter === encounterId);
  if (loc) next.locations[loc.id] = { status: "cleared", hoursLeft: 0 };
  next.ledger.missions += 1;
  if (encounterId.includes("ripple") || encounterId.includes("worm"))
    next.ledger.awakened += 1;
  else next.ledger.demons += encounterId === "paburo-nest" ? 3 : 2;
  if (enc.reward.karma) next.karma += enc.reward.karma;
  if (enc.reward.rank) next.rank = Math.max(1, next.rank + enc.reward.rank);
  if (enc.reward.raku) next.raku = true;
  if (enc.reward.recruit) {
    for (const id of enc.reward.recruit) {
      if (!next.party.includes(id)) next.party.push(id);
    }
  }
  next.flags[enc.reward.flag] = true;

  if (next.flags["doga-cleared"] && next.locations.paburo?.status === "locked") {
    next.locations.paburo = { status: "beacon", hoursLeft: 72 };
  }
  if (next.flags["paburo-cleared"] && next.locations.gonal?.status === "locked") {
    next.locations.gonal = { status: "beacon", hoursLeft: 90 };
  }
  if (next.flags["gonal-cleared"] && next.locations.pieta?.status === "locked") {
    next.locations.pieta = { status: "beacon", hoursLeft: 110 };
  }
  return next;
}

export function nearestLocation(x: number, y: number, radius = 0.045) {
  let best: (typeof LOCATIONS)[number] | undefined;
  let bestD = radius;
  for (const loc of LOCATIONS) {
    const d = dist01(x, y, loc.x, loc.y);
    if (d < bestD) {
      bestD = d;
      best = loc;
    }
  }
  return best;
}

export function clockLabel(hours: number) {
  const day = Math.floor(hours / 24) + 1;
  const h = hours % 24;
  const pad = h.toString().padStart(2, "0");
  return `Day ${day} · ${pad}:00`;
}
