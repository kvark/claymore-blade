import { ENCOUNTERS, ENEMIES, SKILLS, WARRIORS, derived } from "@/game/data/catalog";
import {
  facingToward,
  hexCone,
  hexDisc,
  hexDistance,
  hexEq,
  hexKey,
  hexLine,
  hexNeighbors,
  hexRing,
  hexSweep,
  placeFootprint,
  type Axial,
} from "./hex";
import { Rng } from "./rng";
import type {
  Attr,
  CombatLog,
  CombatState,
  HitKind,
  Part,
  SkillDef,
  Unit,
  UnitTemplate,
} from "./types";

function inBounds(h: Axial, cols: number, rows: number) {
  return h.q >= 0 && h.r >= 0 && h.q < cols && h.r < rows;
}

function liveCells(u: Unit): Axial[] {
  const placed = placeFootprint(u.footprint, u.origin, u.facing);
  return placed.filter((_, i) => {
    const part = u.parts.find((p) => p.hexIndex === i);
    return part ? part.hp > 0 : true;
  });
}

function coreHex(u: Unit): Axial {
  const placed = placeFootprint(u.footprint, u.origin, u.facing);
  return placed[u.coreIndex] ?? u.origin;
}

function occupiedMap(state: CombatState, ignoreId?: string): Map<string, string> {
  const m = new Map<string, string>();
  for (const u of state.units) {
    if (u.dead || u.id === ignoreId) continue;
    for (const c of liveCells(u)) m.set(hexKey(c), u.id);
  }
  return m;
}

function spawnFromTemplate(
  t: UnitTemplate,
  id: string,
  origin: Axial,
  facing: number,
): Unit {
  const d = derived(t.stats);
  const parts: Part[] = t.parts.map((p) => ({
    ...p,
    maxHp: p.hp,
  }));
  const hp = parts.reduce((s, p) => s + p.hp, 0);
  return {
    id,
    templateId: t.id,
    name: t.name,
    title: t.title,
    rank: t.rank,
    side: t.side,
    portrait: t.portrait,
    sprite: t.sprite,
    origin,
    facing,
    footprint: t.footprint.map((h) => ({ ...h })),
    coreIndex: t.coreIndex,
    parts,
    hp,
    maxHp: hp,
    yoki: d.yoki,
    maxYoki: d.yoki,
    trans: t.trans,
    ap: 2,
    maxAp: 2,
    stats: { ...t.stats },
    skills: [...t.skills],
    statuses: [],
    raisedTransThisTurn: false,
    color: t.color,
    dead: false,
  };
}

export function createBattle(
  encounterId: string,
  partyIds: string[],
  seed = (Date.now() % 1_000_000) + 1,
): CombatState {
  const enc = ENCOUNTERS[encounterId];
  if (!enc) throw new Error(`missing encounter ${encounterId}`);
  const units: Unit[] = [];
  partyIds.slice(0, 3).forEach((pid, i) => {
    const t = WARRIORS[pid];
    if (!t) return;
    const origin = enc.playerOrigins[i] ?? enc.playerOrigins[0]!;
    units.push(spawnFromTemplate(t, `p-${t.id}`, origin, 0));
  });
  enc.enemies.forEach((e, i) => {
    const t = ENEMIES[e.template];
    if (!t) return;
    units.push(spawnFromTemplate(t, `e-${t.id}-${i}`, e.origin, e.facing ?? 3));
  });

  const terrain: CombatState["terrain"] = {};
  const rng = new Rng(seed);
  for (let q = 0; q < enc.cols; q++) {
    for (let r = 0; r < enc.rows; r++) {
      const roll = rng.next();
      terrain[`${q},${r}`] =
        roll > 0.92 ? "ruin" : roll > 0.84 ? "mud" : roll < 0.04 ? "water" : "grass";
    }
  }
  for (const u of units) {
    for (const c of liveCells(u)) terrain[hexKey(c)] = "grass";
  }

  const order = [...units]
    .sort((a, b) => b.stats.A + rng.int(0, 9) - (a.stats.A + rng.int(0, 9)))
    .map((u) => u.id);

  const state: CombatState = {
    id: encounterId,
    title: enc.title,
    seed,
    turn: 0,
    round: 1,
    order,
    units,
    terrain,
    cols: enc.cols,
    rows: enc.rows,
    zones: [],
    log: [],
    briefing: enc.briefing,
    art: enc.art ?? "/art/battle-doga.jpg",
  };
  pushLog(state, "info", `${enc.title}. ${units.filter((u) => u.side === "enemy").length} on the board.`);
  beginTurn(state);
  return state;
}

function pushLog(state: CombatState, kind: CombatLog["kind"], text: string) {
  state.log.unshift({ t: state.round * 10 + state.turn, text, kind });
  if (state.log.length > 40) state.log.pop();
}

export function currentUnit(state: CombatState): Unit | undefined {
  const id = state.order[state.turn];
  return state.units.find((u) => u.id === id && !u.dead);
}

export function living(state: CombatState, side?: Unit["side"]) {
  return state.units.filter((u) => !u.dead && (side ? u.side === side : true));
}

function beginTurn(state: CombatState) {
  const u = currentUnit(state);
  if (!u) {
    advanceTurn(state);
    return;
  }
  u.ap = u.maxAp;
  u.raisedTransThisTurn = false;
  u.yoki = Math.min(u.maxYoki, u.yoki + 2);
  u.statuses = u.statuses
    .map((s) => ({ ...s, turns: s.turns - 1 }))
    .filter((s) => s.turns > 0);
  if (u.trans >= 90 && u.side === "player") {
    const rng = new Rng(state.seed + state.round * 17 + u.trans);
    if (rng.chance(0.25 + (u.trans - 90) / 80)) {
      u.ap = 0;
      pushLog(state, "trans", `${u.name} loses the bar. The turn is gone.`);
    }
  }
  if (u.statuses.some((s) => s.telegraph)) {
    for (const e of living(state, u.side === "player" ? "enemy" : "player")) {
      e.nextHint = pickAiSkill(state, e)?.name ?? "Advance";
    }
  }
  tickRipples(state, u.id);
}

function tickRipples(state: CombatState, actorId: string) {
  const keep: CombatState["zones"] = [];
  for (const z of state.zones) {
    const ring = hexRing(z.center, z.radius).filter((h) =>
      inBounds(h, state.cols, state.rows),
    );
    applyZone(state, ring, z.power, z.pa, "A", actorId, false, false);
    pushLog(state, "info", `Ripple expands to ${z.radius}.`);
    if (z.radius < z.maxRadius) {
      keep.push({ ...z, radius: z.radius + 1 });
    }
  }
  state.zones = keep;
}

function advanceTurn(state: CombatState) {
  if (state.over) return;
  const check = () => {
    if (living(state, "enemy").length === 0) state.over = "win";
    if (living(state, "player").length === 0) state.over = "lose";
  };
  check();
  if (state.over) return;
  let guard = 0;
  do {
    state.turn += 1;
    if (state.turn >= state.order.length) {
      state.turn = 0;
      state.round += 1;
    }
    guard += 1;
  } while (!currentUnit(state) && guard < state.order.length + 2);
  beginTurn(state);
  check();
}

export function skillOf(id: string) {
  return SKILLS[id];
}

export function canUse(u: Unit, skill: SkillDef, hasRaki: boolean) {
  if (u.ap < skill.ap) return false;
  if (u.trans < skill.trans) return false;
  if (u.yoki < skill.yoki) return false;
  if (skill.id === "drop" && !hasRaki) return false;
  if (skill.id === "ripple" && !u.parts.some((p) => p.zone === "ripple" && p.hp > 0))
    return false;
  return true;
}

export function moveCost(state: CombatState, hex: Axial) {
  const t = state.terrain[hexKey(hex)] ?? "grass";
  if (t === "water") return 99;
  if (t === "mud") return 2;
  return 1;
}

export function legalMoves(state: CombatState, unitId: string): Axial[] {
  const u = state.units.find((x) => x.id === unitId);
  if (!u || u.dead) return [];
  const occ = occupiedMap(state, u.id);
  const start = coreHex(u);
  const budget = Math.min(u.ap, derived(u.stats).move);
  const out: Axial[] = [];
  const seen = new Map<string, number>();
  const q: { h: Axial; c: number }[] = [{ h: start, c: 0 }];
  seen.set(hexKey(start), 0);
  while (q.length) {
    const cur = q.shift()!;
    for (const n of hexNeighbors(cur.h)) {
      if (!inBounds(n, state.cols, state.rows)) continue;
      const cost = cur.c + moveCost(state, n);
      if (cost > budget) continue;
      const k = hexKey(n);
      if ((seen.get(k) ?? 99) <= cost) continue;
      if (occ.has(k)) continue;
      seen.set(k, cost);
      out.push(n);
      q.push({ h: n, c: cost });
    }
  }
  return out;
}

export function zoneFor(
  state: CombatState,
  u: Unit,
  skill: SkillDef,
  target: Axial,
): Axial[] {
  const from = coreHex(u);
  const face = facingToward(from, target);
  switch (skill.shape) {
    case "self":
      return [from];
    case "single":
      return [target];
    case "line": {
      const len = skill.length ?? skill.range;
      const line = hexLine(from, target).slice(1, len + 1);
      return line.filter((h) => inBounds(h, state.cols, state.rows));
    }
    case "cone":
      return hexCone(from, face, skill.range).filter((h) =>
        inBounds(h, state.cols, state.rows),
      );
    case "blast":
      return hexDisc(target, skill.range).filter((h) =>
        inBounds(h, state.cols, state.rows),
      );
    case "ring":
      return hexRing(from, skill.range).filter((h) =>
        inBounds(h, state.cols, state.rows),
      );
    case "sweep":
      return hexSweep(from, face).filter((h) => inBounds(h, state.cols, state.rows));
    case "ripple":
      return hexRing(from, 1).filter((h) => inBounds(h, state.cols, state.rows));
    case "leap":
      return [target];
    default:
      return [target];
  }
}

export function legalTargets(
  state: CombatState,
  unitId: string,
  skillId: string,
): Axial[] {
  const u = state.units.find((x) => x.id === unitId);
  const skill = SKILLS[skillId];
  if (!u || !skill) return [];
  const from = coreHex(u);
  if (skill.self || skill.shape === "self" || skill.shape === "ripple") return [from];
  if (skill.shape === "leap") {
    const occ = occupiedMap(state, u.id);
    return hexDisc(from, skill.range).filter((h) => {
      if (hexEq(h, from)) return false;
      if (!inBounds(h, state.cols, state.rows)) return false;
      if ((state.terrain[hexKey(h)] ?? "grass") === "water") return false;
      return !occ.has(hexKey(h));
    });
  }
  const cells: Axial[] = [];
  for (let q = 0; q < state.cols; q++) {
    for (let r = 0; r < state.rows; r++) {
      const h = { q, r };
      const d = hexDistance(from, h);
      if (d < 1 || d > skill.range) continue;
      if (skill.shape === "single" || skill.shape === "blast") {
        cells.push(h);
      } else {
        cells.push(h);
      }
    }
  }
  return cells;
}

function attrOf(u: Unit, a: Attr) {
  return u.stats[a];
}

function resolveHit(
  rng: Rng,
  atk: Unit,
  def: Unit,
  skill: SkillDef,
  cover: boolean,
): { kind: HitKind; scale: number } {
  const pa = attrOf(atk, skill.pa);
  const pd = attrOf(def, skill.pd);
  const scale = 1 + 0.25 * Math.max(-4, Math.min(8, pa - pd));
  const hit = derived(atk.stats).hit + (skill.aimed ? -4 : 0);
  const dodge = derived(def.stats).dodge;
  const chance = 0.55 + (hit - dodge) * 0.03;
  const roll = rng.next();
  if (skill.unblockable) return { kind: "solid", scale };
  if (roll > chance + 0.15) return { kind: "miss", scale };
  if (roll > chance) return { kind: "glance", scale };
  const guarded = def.statuses.some((s) => (s.guard ?? 0) > 0) || cover;
  if (guarded && rng.chance(0.55)) return { kind: "blocked", scale };
  return { kind: "solid", scale };
}

function applyDamageToUnit(
  state: CombatState,
  target: Unit,
  amount: number,
  zone: Axial[],
  aimed: boolean,
  rng: Rng,
) {
  const placed = placeFootprint(target.footprint, target.origin, target.facing);
  const hitIdx: number[] = [];
  placed.forEach((h, i) => {
    const part = target.parts.find((p) => p.hexIndex === i);
    if (part && part.hp <= 0) return;
    if (zone.some((z) => hexEq(z, h))) hitIdx.push(i);
  });
  if (!hitIdx.length) return;
  const coreHit = hitIdx.includes(target.coreIndex);
  let dmg = Math.round(amount * (coreHit ? 1 : 0.5));
  const focus =
    aimed && hitIdx.length
      ? (hitIdx.includes(target.coreIndex) ? target.coreIndex : hitIdx[0]!)
      : null;

  if (focus != null) {
    const part = target.parts.find((p) => p.hexIndex === focus);
    if (part) {
      part.hp = Math.max(0, part.hp - dmg);
      if (part.hp === 0 && target.parts.length > 1) {
        pushLog(state, "sever", `${target.name}'s ${part.name} is carved off.`);
        if (part.zone) {
          target.skills = target.skills.filter((s) => SKILLS[s]?.id !== part.zone);
        }
      }
    }
  } else {
    const share = Math.max(1, Math.round(dmg / hitIdx.length));
    for (const i of hitIdx) {
      const part = target.parts.find((p) => p.hexIndex === i);
      if (!part) continue;
      part.hp = Math.max(0, part.hp - share);
      if (part.hp === 0 && target.parts.length > 1) {
        pushLog(state, "sever", `${target.name}'s ${part.name} is carved off.`);
      }
    }
  }
  target.hp = target.parts.reduce((s, p) => s + p.hp, 0);
  if (target.hp <= 0) {
    target.dead = true;
    target.hp = 0;
    pushLog(state, "death", `${target.name} falls.`);
  }
  void rng;
}

function applyZone(
  state: CombatState,
  zone: Axial[],
  power: number,
  pa: Attr,
  pd: Attr,
  attackerId: string,
  aimed: boolean,
  unblockable: boolean,
) {
  const atk = state.units.find((u) => u.id === attackerId);
  if (!atk) return;
  const rng = new Rng(state.seed + state.round * 31 + state.turn * 7 + power);
  const dummySkill = {
    ...SKILLS.cut,
    pa,
    pd,
    aimed,
    unblockable,
  } satisfies SkillDef;
  const hitIds = new Set<string>();
  for (const h of zone) {
    for (const u of state.units) {
      if (u.dead || u.id === attackerId) continue;
      if (u.side === atk.side && dummySkill) {
        /* friendly fire on */
      }
      const cells = liveCells(u);
      if (cells.some((c) => hexEq(c, h))) hitIds.add(u.id);
    }
  }
  for (const id of hitIds) {
    const def = state.units.find((u) => u.id === id);
    if (!def) continue;
    const cover = liveCells(def).some(
      (c) => zone.some((z) => hexEq(z, c)) && state.terrain[hexKey(c)] === "ruin",
    );
    const { kind, scale } = resolveHit(rng, atk, def, dummySkill, cover);
    const transMul = 1 + atk.trans / 200;
    const base = power * scale * transMul;
    const dmg =
      kind === "miss"
        ? 0
        : kind === "glance"
          ? Math.round(base * 0.2)
          : kind === "blocked"
            ? Math.round(base * rng.next() * 0.3)
            : Math.round(base);
    if (kind === "miss") pushLog(state, "miss", `${atk.name} misses ${def.name}.`);
    else if (kind === "blocked")
      pushLog(state, "hit", `${def.name} catches ${atk.name}'s blow.`);
    else
      pushLog(
        state,
        "hit",
        `${atk.name} → ${def.name}: ${kind} ${dmg}${aimed ? " (aimed)" : ""}.`,
      );
    if (dmg > 0) applyDamageToUnit(state, def, dmg, zone, aimed, rng);
  }
}

export type PlayerAction =
  | { type: "move"; hex: Axial }
  | { type: "skill"; skillId: string; hex: Axial }
  | { type: "raise" }
  | { type: "wait" };

export function act(
  state: CombatState,
  action: PlayerAction,
  opts?: { hasRaki?: boolean },
): CombatState {
  if (state.over) return state;
  const u = currentUnit(state);
  if (!u) return state;
  const next: CombatState = structuredClone(state);

  const actor = next.units.find((x) => x.id === u.id)!;
  if (action.type === "raise") {
    if (actor.raisedTransThisTurn || actor.ap < 0) return state;
    actor.trans = Math.min(100, actor.trans + 16);
    actor.raisedTransThisTurn = true;
    pushLog(next, "trans", `${actor.name} opens the bar (${actor.trans}).`);
    if (actor.trans >= 100) {
      pushLog(next, "trans", `${actor.name} is at the edge.`);
    }
    return next;
  }
  if (action.type === "wait") {
    actor.ap = 0;
    actor.trans = Math.max(0, actor.trans - 4);
    pushLog(next, "info", `${actor.name} waits.`);
    advanceTurn(next);
    return next;
  }
  if (action.type === "move") {
    const moves = legalMoves(next, actor.id);
    if (!moves.some((h) => hexEq(h, action.hex))) return state;
    const cost = Math.min(
      actor.ap,
      Math.max(1, hexDistance(coreHex(actor), action.hex)),
    );
    actor.facing = facingToward(coreHex(actor), action.hex);
    const delta = {
      q: action.hex.q - coreHex(actor).q,
      r: action.hex.r - coreHex(actor).r,
    };
    actor.origin = { q: actor.origin.q + delta.q, r: actor.origin.r + delta.r };
    actor.ap = Math.max(0, actor.ap - Math.max(1, cost));
    pushLog(next, "info", `${actor.name} steps.`);
    if (actor.ap <= 0) advanceTurn(next);
    return next;
  }

  const skill = SKILLS[action.skillId];
  if (!skill || !canUse(actor, skill, !!opts?.hasRaki)) return state;
  const targets = legalTargets(next, actor.id, skill.id);
  if (!targets.some((h) => hexEq(h, action.hex))) return state;

  actor.facing = facingToward(coreHex(actor), action.hex);
  actor.ap -= skill.ap;
  actor.yoki -= skill.yoki;
  if (skill.transDelta) {
    actor.trans = Math.max(0, Math.min(100, actor.trans + skill.transDelta));
  }
  if (skill.heal) {
    const heal = skill.heal;
    actor.parts.forEach((p) => {
      if (p.hp > 0) p.hp = Math.min(p.maxHp, p.hp + Math.round(heal / actor.parts.length));
      else if (skill.id === "regen") p.hp = Math.min(p.maxHp, Math.round(p.maxHp * 0.4));
    });
    actor.hp = actor.parts.reduce((s, p) => s + p.hp, 0);
    pushLog(next, "info", `${actor.name} knits flesh (+${heal}).`);
  }
  if (skill.guard) {
    actor.statuses.push({ id: "guard", name: "Guard", turns: 2, guard: skill.guard });
  }
  if (skill.telegraph) {
    actor.statuses.push({ id: "read", name: "Read Energy", turns: 3, telegraph: true });
    for (const e of living(next, "enemy")) {
      e.nextHint = pickAiSkill(next, e)?.name ?? "Advance";
    }
    pushLog(next, "info", `${actor.name} reads the field.`);
  }
  if (skill.shape === "leap" && skill.move) {
    const occ = occupiedMap(next, actor.id);
    if (!occ.has(hexKey(action.hex))) {
      if (skill.afterimage) {
        actor.statuses.push({
          id: "after",
          name: "Afterimage",
          turns: 2,
          afterimage: { ...coreHex(actor) },
        });
      }
      actor.origin = { ...action.hex };
      pushLog(next, "info", `${actor.name} is already gone.`);
    }
  }
  if (skill.shape === "ripple") {
    next.zones.push({
      id: `rip-${next.round}-${actor.id}`,
      kind: "ripple",
      sourceId: actor.id,
      center: { ...coreHex(actor) },
      radius: 1,
      maxRadius: 3,
      power: skill.power,
      pa: skill.pa,
    });
    pushLog(next, "info", `${actor.name} starts a ripple.`);
  } else if (skill.power > 0) {
    const zone = zoneFor(next, actor, skill, action.hex);
    const strikes = skill.strikes ? derived(actor.stats).strikes : 1;
    for (let i = 0; i < strikes; i++) {
      applyZone(
        next,
        zone,
        skill.power,
        skill.pa,
        skill.pd,
        actor.id,
        !!skill.aimed,
        !!skill.unblockable,
      );
    }
  }

  if (actor.ap <= 0) advanceTurn(next);
  if (living(next, "enemy").length === 0) next.over = "win";
  if (living(next, "player").length === 0) next.over = "lose";
  return next;
}

function pickAiSkill(state: CombatState, u: Unit): SkillDef | undefined {
  const usable = u.skills
    .map((id) => SKILLS[id])
    .filter((s): s is SkillDef => !!s && canUse(u, s, false) && s.power > 0);
  return usable.sort((a, b) => b.power - a.power)[0];
}

export function runAi(state: CombatState): CombatState {
  let cur = state;
  let guard = 0;
  while (!cur.over && currentUnit(cur)?.side === "enemy" && guard < 24) {
    guard += 1;
    const u = currentUnit(cur);
    if (!u) break;
    const foes = living(cur, "player");
    if (!foes.length) break;
    const from = coreHex(u);
    const nearest = [...foes].sort(
      (a, b) => hexDistance(from, coreHex(a)) - hexDistance(from, coreHex(b)),
    )[0]!;
    const skill = pickAiSkill(cur, u);
    if (skill && u.trans < skill.trans && !u.raisedTransThisTurn) {
      cur = act(cur, { type: "raise" });
      continue;
    }
    if (skill) {
      const targets = legalTargets(cur, u.id, skill.id);
      const foeCells = liveCells(nearest);
      const hit = targets.find((t) => {
        const z = zoneFor(cur, u, skill, t);
        return z.some((h) => foeCells.some((c) => hexEq(c, h)));
      });
      if (hit) {
        cur = act(cur, { type: "skill", skillId: skill.id, hex: hit });
        continue;
      }
    }
    const moves = legalMoves(cur, u.id);
    if (moves.length) {
      const step = [...moves].sort(
        (a, b) => hexDistance(a, coreHex(nearest)) - hexDistance(b, coreHex(nearest)),
      )[0]!;
      if (hexDistance(step, coreHex(nearest)) < hexDistance(from, coreHex(nearest))) {
        cur = act(cur, { type: "move", hex: step });
        continue;
      }
    }
    cur = act(cur, { type: "wait" });
  }
  return cur;
}

export { liveCells, coreHex, occupiedMap };
