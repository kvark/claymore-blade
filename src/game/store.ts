import { create } from "zustand";
import { ENCOUNTERS, locById } from "@/game/data/catalog";
import { act, createBattle, runAi, type PlayerAction } from "@/game/sim/combat";
import {
  applyVictory,
  hoursForTravel,
  nearestLocation,
  newWorld,
  tickHours,
} from "@/game/sim/world";
import type { Axial } from "@/game/sim/hex";
import type { CombatState, GameMode, PersistBlob, WorldState } from "@/game/sim/types";
import { sfx, unlockAudio } from "./audio";

const SAVE_KEY = "claymore.save.v1";

export type UiCombat = {
  selectedSkill?: string;
  hoverHex?: Axial;
  preview?: Axial[];
};

type Store = {
  mode: GameMode;
  world: WorldState;
  combat: CombatState | null;
  result?: { title: string; body: string; win: boolean };
  ui: UiCombat;
  keys: Set<string>;
  introStep: number;
  boot: () => void;
  newHunt: () => void;
  continueHunt: () => void;
  persist: () => void;
  setMode: (m: GameMode) => void;
  moveParty: (dx: number, dy: number, dt: number) => void;
  travelTo: (id: string) => void;
  enterLocation: (id: string) => void;
  restTown: () => void;
  startEncounter: (id: string) => void;
  combatAct: (a: PlayerAction) => void;
  setSkill: (id?: string) => void;
  setHover: (h?: Axial) => void;
  holdKey: (code: string, down: boolean) => void;
  dismissResult: () => void;
};

function blobOf(s: Pick<Store, "mode" | "world" | "combat" | "result">): PersistBlob {
  return { v: 1, mode: s.mode === "title" ? "world" : s.mode, world: s.world, combat: s.combat, result: s.result };
}

function writeSave(s: PersistBlob) {
  try {
    localStorage.setItem(SAVE_KEY, JSON.stringify(s));
  } catch {
    /* ignore */
  }
}

export function readSave(): PersistBlob | null {
  try {
    const raw = localStorage.getItem(SAVE_KEY);
    if (!raw) return null;
    const p = JSON.parse(raw) as PersistBlob;
    if (p.v !== 1) return null;
    return p;
  } catch {
    return null;
  }
}

export const useGame = create<Store>((set, get) => ({
  mode: "title",
  world: newWorld(),
  combat: null,
  ui: {},
  keys: new Set(),
  introStep: 0,

  boot: () => {
    const existing = readSave();
    if (existing) {
      set({
        world: existing.world,
        combat: existing.combat,
        mode: "title",
        result: existing.result,
      });
    }
    if (typeof window !== "undefined") {
      (window as unknown as { __wave?: unknown }).__wave = {
        start: (id: string) => get().startEncounter(id),
      };
    }
  },

  persist: () => {
    const s = get();
    writeSave(blobOf(s));
  },

  newHunt: () => {
    unlockAudio();
    sfx.ui();
    set({
      mode: "intro",
      world: newWorld(),
      combat: null,
      result: undefined,
      introStep: 0,
      ui: {},
    });
    get().persist();
  },

  continueHunt: () => {
    unlockAudio();
    const s = readSave();
    if (!s) return;
    sfx.ui();
    set({
      world: s.world,
      combat: s.combat,
      result: s.result,
      mode: s.combat ? "combat" : s.mode === "intro" ? "world" : s.mode,
    });
  },

  setMode: (mode) => {
    set({ mode });
    get().persist();
  },

  moveParty: (dx, dy, dt) => {
    const { world, mode } = get();
    if (mode !== "world") return;
    const speed = 0.18;
    const nx = Math.min(0.92, Math.max(0.08, world.partyX + dx * speed * dt));
    const ny = Math.min(0.88, Math.max(0.10, world.partyY + dy * speed * dt));
    if (nx === world.partyX && ny === world.partyY) return;
    let next = { ...world, partyX: nx, partyY: ny };
    next = tickHours(next, dt * 2.4);
    const loc = nearestLocation(nx, ny, 0.028);
    set({ world: next });
    if (loc && loc.id !== world.lastTown) {
      get().enterLocation(loc.id);
    }
  },

  travelTo: (id) => {
    const loc = locById(id);
    if (!loc) return;
    const { world } = get();
    const hours = hoursForTravel(
      { x: world.partyX, y: world.partyY },
      { x: loc.x, y: loc.y },
    );
    let next = tickHours(
      { ...world, partyX: loc.x, partyY: loc.y, lastTown: undefined },
      hours,
    );
    set({ world: next });
    get().enterLocation(id);
  },

  enterLocation: (id) => {
    const loc = locById(id);
    if (!loc) return;
    sfx.ui();
    set({
      world: { ...get().world, lastTown: id, partyX: loc.x, partyY: loc.y },
      mode: "town",
    });
    get().persist();
  },

  restTown: () => {
    const { world } = get();
    let next = tickHours(world, 8);
    // heal is implicit — battles spawn fresh units
    next = { ...next, karma: next.karma + 0 };
    sfx.ui();
    set({ world: next });
    get().persist();
  },

  startEncounter: (id) => {
    const { world } = get();
    const enc = ENCOUNTERS[id];
    if (!enc) return;
    unlockAudio();
    const party = [...world.party];
    if (id === "paburo-nest") {
      if (!party.includes("miria")) party.push("miria");
      if (!party.includes("helen")) party.push("helen");
    }
    if (id === "pieta-worm" && !party.includes("deneve")) party.push("deneve");
    const combat0 = createBattle(id, party);
    const combat = runAi(combat0);
    set({ combat, mode: "combat", world: { ...world, party }, ui: {} });
    get().persist();
  },

  combatAct: (a) => {
    const { combat, world } = get();
    if (!combat || combat.over) return;
    let next = act(combat, a, { hasRaku: world.raku });
    if (a.type === "move") sfx.move();
    else if (a.type === "raise") sfx.trans();
    else sfx.hit();
    if (!next.over && next !== combat) {
      next = runAi(next);
    }
    if (next.over === "win") {
      sfx.win();
      const enc = ENCOUNTERS[next.id];
      const w = applyVictory(world, next.id);
      set({
        combat: next,
        world: w,
        result: {
          win: true,
          title: enc?.title ?? "Hunt ended",
          body: victoryCopy(next.id, w),
        },
        mode: "result",
        ui: {},
      });
      get().persist();
      return;
    }
    if (next.over === "lose") {
      sfx.lose();
      set({
        combat: next,
        result: {
          win: false,
          title: "The bar took you",
          body: "The Office will send someone else. They always do. The village will not remember your number.",
        },
        mode: "result",
        ui: {},
      });
      get().persist();
      return;
    }
    set({ combat: next, ui: { ...get().ui, preview: undefined } });
  },

  setSkill: (id) => set({ ui: { ...get().ui, selectedSkill: id } }),
  setHover: (h) => set({ ui: { ...get().ui, hoverHex: h } }),
  holdKey: (code, down) => {
    const keys = new Set(get().keys);
    if (down) keys.add(code);
    else keys.delete(code);
    set({ keys });
  },
  dismissResult: () => {
    const { result } = get();
    if (!result?.win) {
      set({ mode: "title", combat: null, result: undefined, world: newWorld() });
      get().persist();
      return;
    }
    set({ mode: "world", combat: null, result: undefined });
    get().persist();
  },
}));

function victoryCopy(id: string, w: WorldState) {
  if (id === "doga-yoma")
    return `The well is only a well again. A boy named Raki will not leave the road. He can pull the bar down when you cannot. Rank still ${w.rank}.`;
  if (id === "paburo-nest")
    return `Miria folds her cloak. Helen shakes blood off an arm that is longer than it should be. They will walk with you. Gonal has gone quiet in the wrong way.`;
  if (id === "gonal-ripple")
    return `Ophelia is meat on four empty hexes. The last ring fades. Pieta is calling every number that can still stand.`;
  if (id === "pieta-worm")
    return `The worm is a line of dead cells. Deneve puts a hand back on. The north will tell this story incorrectly.`;
  return "The board is empty.";
}

export function hasSave() {
  return !!readSave();
}
