import type { Axial } from "./hex";

export type Side = "player" | "enemy";
export type Attr = "S" | "A" | "C" | "P" | "W";
export type HitKind = "miss" | "glance" | "blocked" | "solid";
export type ShapeKind =
  | "self"
  | "single"
  | "line"
  | "cone"
  | "blast"
  | "ring"
  | "sweep"
  | "ripple"
  | "leap";

export type Stats = {
  S: number;
  A: number;
  C: number;
  P: number;
  W: number;
};

export type Part = {
  id: string;
  name: string;
  hexIndex: number;
  hp: number;
  maxHp: number;
  zone?: string;
};

export type Status = {
  id: string;
  name: string;
  turns: number;
  guard?: number;
  hide?: boolean;
  telegraph?: boolean;
  afterimage?: Axial;
};

export type SkillDef = {
  id: string;
  name: string;
  blurb: string;
  ap: number;
  trans: number;
  yoki: number;
  shape: ShapeKind;
  range: number;
  length?: number;
  pa: Attr;
  pd: Attr;
  power: number;
  aimed?: boolean;
  clean?: boolean;
  self?: boolean;
  heal?: number;
  transDelta?: number;
  move?: number;
  guard?: number;
  telegraph?: boolean;
  afterimage?: boolean;
  unblockable?: boolean;
  strikes?: boolean;
  learn?: boolean;
};

export type UnitTemplate = {
  id: string;
  name: string;
  title: string;
  rank?: number;
  side: Side;
  portrait: string;
  sprite?: string;
  stats: Stats;
  skills: string[];
  trans: number;
  footprint: Axial[];
  coreIndex: number;
  parts: { id: string; name: string; hexIndex: number; hp: number; zone?: string }[];
  color: string;
};

export type Unit = {
  id: string;
  templateId: string;
  name: string;
  title: string;
  rank?: number;
  side: Side;
  portrait: string;
  sprite?: string;
  origin: Axial;
  facing: number;
  footprint: Axial[];
  coreIndex: number;
  parts: Part[];
  hp: number;
  maxHp: number;
  yoki: number;
  maxYoki: number;
  trans: number;
  ap: number;
  maxAp: number;
  stats: Stats;
  skills: string[];
  statuses: Status[];
  raisedTransThisTurn: boolean;
  nextHint?: string;
  color: string;
  dead: boolean;
};

export type DelayedZone = {
  id: string;
  kind: "ripple";
  sourceId: string;
  center: Axial;
  radius: number;
  maxRadius: number;
  power: number;
  pa: Attr;
};

export type CombatLog = {
  t: number;
  text: string;
  kind: "hit" | "miss" | "info" | "sever" | "death" | "trans";
};

export type CombatState = {
  id: string;
  title: string;
  seed: number;
  turn: number;
  round: number;
  order: string[];
  units: Unit[];
  terrain: Record<string, "grass" | "mud" | "ruin" | "water">;
  cols: number;
  rows: number;
  zones: DelayedZone[];
  log: CombatLog[];
  over?: "win" | "lose";
  briefing: string;
  art: string;
};

export type WorldStatus = "quiet" | "beacon" | "dead" | "cleared" | "locked";

export type LocationDef = {
  id: string;
  name: string;
  region: string;
  blurb: string;
  x: number;
  y: number;
  kind: "village" | "city" | "shrine" | "keep" | "office" | "wild";
  encounter?: string;
  deadline?: number;
  art?: string;
};

export type Ledger = {
  demons: number;
  awakened: number;
  silvers: number;
  humans: number;
  missions: number;
};

export type WorldState = {
  hours: number;
  partyX: number;
  partyY: number;
  party: string[];
  raki: boolean;
  rank: number;
  karma: number;
  authority: number;
  ledger: Ledger;
  locations: Record<string, { status: WorldStatus; hoursLeft: number }>;
  flags: Record<string, boolean>;
  lastTown?: string;
  travel?: { to: string; t: number };
};

export type GameMode = "title" | "intro" | "world" | "town" | "combat" | "codex" | "result";

export type PersistBlob = {
  v: 1;
  world: WorldState;
  mode: GameMode;
  combat: CombatState | null;
  result?: { title: string; body: string; win: boolean };
};
