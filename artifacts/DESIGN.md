# Claymore

**A roleplaying tactical hunt**  
The October 2007–13 notes (`wave.pdf` is a Google Wave export titled `=+ Claymore += GAME DESIGN`) made playable. Island of silver-eyed warriors, an Organization that brands them, and the yoma they were built to kill.

This document is the source of truth for the playable slice and for a later native Blade port.

---

## 1. Pitch

You are Clare, No. 47 — half human, half yoma, ranked, expendable. Villages light beacons when something starts eating them. You walk the island, take the hunts, and decide how much of Teresa you are willing to use.

Two scales:

1. **The island** — Fallout-2 style world map. Free movement. Towns, roads, mountains, abandoned places. Time passes. Beacons go dark if you are late.
2. **The hunt** — turn-based hex combat. Warriors occupy one cell. Awakened occupy many. Damage is zonal. A severed limb is a hole in a boss's footprint.

The old notes argued for pause-driven real-time (Dragon Age). The current brief overrides that: **tactical battles are turn-based on a hex grid.** The pause-RT layer is archived as a future optional mode, not the v1 combat.

---

## 2. What the old notes decided (kept)

| Note | Decision |
| --- | --- |
| Start as human, become a warrior, risk becoming a demon | Three life-phases. V1 starts in Phase 1 (already branded). |
| Look / sex / pre-history / battle style / half vs quarter demon | Character creation. V1 ships Clare plus recruitable series roster. |
| Trans-meter | Core resource. Raise it to unlock and amplify techniques. Lowering it is a skill, not a given. |
| Fallout-2 world map, village beacons, time pressure | Kept. Late arrival = fewer living, then an abandoned yoma nest. |
| Perception reads auras at range | Kept. Aura color + strength from the remote trans-meter. High Perception adds intent. |
| Party gathers on a timer; highest rank leads on the global map | Kept for Pieta-style hunts. |
| Hit outcomes are discrete, not HP soup | miss / glance / blocked / solid. |
| Effect scale `1.0 + 0.25 * max(-4, Pa − Pd)` | Kept. |
| Aimed shots can take limbs | Kept for bosses and as a perk. |
| Attributes S / A / C / P / W | Kept, with derived stats as written. |
| Rank, tavern board, karma, kill ledger | Kept. |
| One human companion ("puppet") | Kept. Raki. Morale, lure, trans restore. |
| Playing as a demon is a different game | Phase 2. No XP. Hunger clock. Free trans, fragile human-form. |
| Learning = watch + practice + rage | Kept as the skill-unlock loop. |
| Relationships: finite warriors, Organization authority, demon unions | Kept as the political layer. |

## 3. What this brief changes

- Battles are **hex, turn-based**, not pause-RT.
- Bosses are **multi-cell** with **zonal** attacks and **zonal** vulnerability.
- The playable roster is the **Claymore** television series: Clare, Miria, Helen, Deneve, Ophelia, Raki, the Organization. Skills keep series names (Quicksword, Windcutter, Stretching, Phantom, Drill Sword, Regeneration, Yoki Sense, Ripple Blade).
- Native long-term target is **Blade (Rust)**. The live prototype is a web vertical slice of the same rules.

---

## 4. Tone and look

Dark European island. Mud roads, wet stone, pine, ruined keeps. Warriors wear black armored undersuits and pale cloaks, silver eyes, hair from white to ash. Swords are too large. Yoki is a sick pale light, not fireworks.

Palette (UI and world):

| Token | Hex | Use |
| --- | --- | --- |
| Ink | `#0b0a09` | Night, chrome |
| Ash | `#ebe4d6` | Type, steel highlights |
| Steel | `#c8ccd4` | Primary actions, blades |
| Dust | `#8d8578` | Secondary type |
| Blood | `#9a2430` | Wounds, high trans, danger chips only |

No gold UI. No purple. Type: **Cormorant Garamond** (display) + **Figtree** (body).

Reference art already in the project:

- `artifacts/character.jpg` — warrior identity (silver hair, black suit, greatsword, pale eyes)
- `artifacts/awaken-demon.jpg` — awakened anatomy (elongated, bone-white, wrong joints)
- `artifacts/symbols.gif` — Organization rank marks, 1–47
- `artifacts/wave.pdf` — the original Google Wave notes

---

## 5. World (the island)

The island is the whole campaign. Five regions, one Organization, three old wounds.

```
                    ALFONS (north, ice)
                         Pieta
                    Mount Shire  ·  Paburo
        LAUTREC (west)              EAST (Organization)
     Witch's Maw · Hanel · Lacroa      Sutafu
              Doga · Stora
                         Gonal
                    MUCHA (south)
```

| Place | Region | Role |
| --- | --- | --- |
| **Doga** | Lautrec | First hunt. A boy named Raki who will not leave you alone. |
| **Stora** | Lautrec | Market, rumors, first tavern board. |
| **Mount Shire** | Alfons border | Shrine. A dying sister. A promise. |
| **Hanel** | Lautrec | City. Statues of Teresa and the girl she saved. |
| **Lacroa** | Toulouse | Search parties. You are seen. |
| **Pieta** | Alfons | Northern gathering. Time-limited party hunt. |
| **Paburo** | Highlands | Stretching-limb yoma nest. |
| **Gonal** | Mucha | Ophelia. First multi-cell awakened. |
| **Witch's Maw** | Lautrec / Zakol | Abandoned keep. Something older than a rank. |
| **Sutafu** | East | The Organization. Rank, orders, punishment. |

**Time.** One world-hour per map tile at walk speed; roads cheaper; mountains slower. A beacon has a deadline. Miss it and the pin turns black. Black pins become yoma homes — harder fights, no villagers, better salvage.

**Perception.** Auras render as colored haze on the map. Green-gold = warrior yoki. Dirty red = yoma. White-violet = awakened. Strength falls with distance. High Perception prints a verb: *feeding, fleeing, waiting, hunting you*.

**Parties.** Some hunts require three or more silvers at a rally hex before a world-deadline. On the global map the highest rank is the party leader (you still steer in local space). In combat you command the whole cell.

---

## 6. People

Series names. The October notes used placeholders; this document does not.

| Name | Rank | What they bring |
| --- | --- | --- |
| **Clare** | 47 | Yoki Sense, Quicksword, later Windcutter. Playable default. |
| **Teresa of the Faint Smile** | 1 (fallen) | Memory, not a unit. The reason Clare exists. |
| **Irene** | 2 (ex) | Teacher of Quicksword. One arm given. |
| **Miria** | 6 | Phantom. Party leader energy. Recruit after Paburo. |
| **Galatea** | 3 | God-Eye. Reads future actions (UI telegraph). |
| **Ophelia** | 4 | Ripple Blade. First named awakened boss. |
| **Flora** | 8 | Windcutter. Precision, not speed. |
| **Jean** | 9 | Drill Sword. Highest single-hex strike. |
| **Deneve** | 15 | Regeneration. Limb restore. |
| **Helen** | 22 | Stretching. Reach 3–4 hexes. |
| **Raki** | — | The human companion. Morale, lure, trans drop. |
| **Priscilla** | abyss | Campaign shadow. Not in the vertical slice. |

V1 party cap: **three silvers + Raki off-board as a support action.**

---

## 7. Attributes

| Attr | Short | Governs |
| --- | --- | --- |
| Strength | S | Health, attack damage, break chance on limbs |
| Agility | A | Move, initiative, attack speed (multi-strike skills), hit |
| Control | C | Trans ceiling, hit, resist awakening, aimed shots |
| Perception | P | Aura range, dodge, read-energy, interrupt windows |
| Wisdom | W | Yoki pool, dodge, trans threshold, learn speed |

Derived (from the notes, locked):

```
hit        = A + C
dodge      = A + P + W
detect     = P
move       = A
transMax   = C + W          # the point you can still pull back from
health     = S
yoki       = W
damage     = S
strikes    = A              # extra hits on Quicksword / Windcutter
```

Start styles (creation, later):

- Defend — +C +P
- Attack — +S +A
- Balanced — +1 all
- Half-demon — more power, worse transMax
- Quarter-demon — better Control, slower unique skills

---

## 8. Trans-meter (yoki)

A single bar, 0–100.

- You may **raise** it at any time (free action, or as a rider on a skill).
- Raising it unlocks the next band of techniques and multiplies outgoing scale.
- Using a demon-mode skill while the bar is high has a chance to **promote** that skill into a standard (no longer locked).
- Lowering it is not free. It depends on Control+Wisdom, and on a nearby human (Raki). There is a floor you cannot cross without help.
- At 90+ : lose-control checks each turn (skip, wild swing, or awaken).
- 100 with a failed Control check: **Awakening**. Character leaves the party. If it was you, Phase 2.

Bands:

| Trans | Band | What opens |
| --- | --- | --- |
| 0–19 | Human | Attack, Defend, Move, Hide Energy |
| 20–39 | Silver | Aimed, Jump, Heal, Yoki Sense |
| 40–59 | Edge | Quicksword, Stretching, Phantom, Windcutter |
| 60–79 | Demon-mode | Drill Sword, Demon Legs, Ripple, Control Demon |
| 80–99 | Breaking | Wings, Possess, full regenerate, boss-tier zones |
| 100 | Awake | You are the encounter |

---

## 9. Combat (hex, turn-based)

### 9.1 Board

- Pointy-top axial hexes.
- Typical hunt: 11 × 9. Pieta / Maw: 13 × 11.
- Terrain: grass, road, mud (cost 2), ruin (cover: +block), water (impass), height (advantage on Pa).

### 9.2 Turn

Initiative = Agility + 1d10, rolled once.
On your turn you get **2 AP**:

- Step to a neighbor = 1 AP (2 in mud)
- Skill / attack = 1–2 AP
- Raise or hold trans = 0 AP (once)
- Sprint (2 AP, no skill)
- Wait / Rest (2 AP, regen, drop trans slightly)

Queue is visible. Interrupts exist only as skills (Phantom, God-Eye). The old interruption-penalty system is reserved for the optional pause-RT mode.

### 9.3 Hit table

Human brains want buckets, not 17-damage. Every strike resolves to one of:

| Outcome | Damage | Notes |
| --- | --- | --- |
| Miss | 0 | Whiff, no rage |
| Glance | 20% | Late dodge |
| Blocked | 0–30% | Defender spent Defend, or cover |
| Solid | 100% × scale | Full. Can roll a limb if aimed |

```
scale = 1.0 + 0.25 * clamp(Pa - Pd, -4, 8)
```

Pa / Pd are the skill's listed pair (usually S vs A, or C vs C, etc.).

Aimed shots: −hit, on Solid may **sever** the targeted part.

### 9.4 Multi-cell monsters

An awakened is a **footprint**: a set of axial cells plus one **core**.

Example — Ophelia, the Ripple (4-cell):

```
   [arm]
[core][torso][tail]
```

Example — Pieta worm (5-cell line, rotates as it moves).

Rules:

- The unit occupies every live cell. Pathing treats the whole shape. Rotation costs 1 AP.
- A zone that overlaps **any** cell applies once, then is modified by **which** cells:
  - Core hit: 100% and a stagger check
  - Limb hit: 50%, and that cell can be **severed**
- A severed cell is removed from the footprint. The boss loses the zone that limb was used for. Movement may break (lost tail = cannot rotate cheaply).
- When only the core remains, the boss is "skinned" — faster, smaller, desperate.
- Some bosses **regrow** a cell by spending a turn and eating.

This is zonal damage in both directions: you carve the monster, it carves the board.

### 9.5 Zones

Every attack has a **shape** placed from a source hex + facing:

| Shape | Geometry |
| --- | --- |
| Single | one hex |
| Line N | N hexes in facing |
| Cone R | hex wedge, width grows |
| Blast R | filled disc |
| Ring R | hollow disc |
| Sweep | three neighbors in an arc |
| Ripple | ring that expands 1 hex per tick for 3 ticks |
| Cross | two lines through origin |

Friendly fire: on, unless the skill says Clean.

---

## 10. Techniques (from the series)

Human / common

| Id | Name | AP | Trans | Shape | Note |
| --- | --- | --- | --- | --- | --- |
| attack | Cut | 1 | 0 | Single | Pa = S |
| aimed | Aimed Cut | 1 | 20 | Single | Limb table |
| defend | Guard | 1 | 0 | Self | Next Pd bonus |
| jump | Vault | 1 | 20 | Leap 2 | Relocate |
| heal | Heal | 1 | 20 | Self | Heal a part |
| rest | Rest | 2 | 0 | Self | Regen, −trans |
| hide | Hide Energy | 1 | 0 | Self | Drop detect |
| trans_drop | Trans Drop | 1 | — | Self | Needs Raki or high C |

Learned / signature

| Id | Name | Who | Trans | Shape | Series |
| --- | --- | --- | --- | --- | --- |
| flash | Quicksword | Clare / Irene | 40 | Sweep + self-guard | Quicksword |
| wind | Windcutter | Flora / Clare later | 40 | Line 3, high hit | Windcutter |
| stretch | Stretching | Helen | 40 | Line 4 | Remote arm |
| phantom | Phantom | Miria | 40 | Leap 3 + after-image | Phantom |
| spiral | Drill Sword | Jean | 60 | Single, huge Pa | Drill Sword |
| godseye | God-Eye | Galatea | 40 | Self | Telegraph enemy intents |
| ripple | Ripple Blade | Ophelia | 60 | Ripple zone | Ripple Blade |
| regen | Regeneration | Deneve | 40 | Self | Restore a severed part |
| micro | Micro Cuts | Clare from Irene | 60 | Single × strikes | Invisible barrage |
| hyper | Hyper Blade | from No.4 line | 60 | Sustained, unblockable | Vibrating arm |
| control | Bend Demon | Galatea | 60 | Aura | Force misses |
| read | Yoki Sense | Clare | 20 | Self | See next action |
| legs | Demon Legs | any breaking | 80 | Self | +move, locked |
| wings | Demon Wings | any breaking | 80 | Self | Fly, ignore terrain |
| possess | Possess | rare | 80 | Single | Take a demon for N turns |
| learn | Watch | any | 20 | Self | Theory on a target |
| rage_hold | Hold Rage | any | 20 | Self | Bank rage |

**Rage.** Being hit or hitting fills a short rage pool. The next skill spends it for bonus scale and counts as **practice**. Enough theory (Watch) + practice unlocks the graph node. This is the notes' learning system, kept.

**Progression of a technique:** 3 ranks (Warcraft-style). Rank 2–3 from practice on things stronger than you.

---

## 11. Progression, rank, karma

- Rank is an Organization number, 47 → 1. It changes how silvers speak to you and which beacons you are offered.
- Ledger: demons, awakened, silvers, humans, missions.
- Karma is a signed int from that ledger plus choices (save the village / make it in time / refuse an order / kill a human).
- Karma shifts prices, lines, and whether a silver will stand with you or hunt you.
- Organization **authority** is separate. Obey: +small. Disobey: −20 to −30. Cross a hard rule (kill a human, refuse a purge, approach awakening in public): the Organization turns.

---

## 12. Playing as a demon (Phase 2)

Unlocked if Clare awakens, or as a New Hunt option later.

- No XP, no new skills, no attribute ups.
- Hunger: one week without feeding is death.
- Trans is free in both directions. Human-form is paper.
- Demon unions (the old No.1s) have territories. You pick a side or you are meat.

Not in the vertical slice except as a lose state with a short epilogue.

---

## 13. Vertical slice (what ships in this prototype)

Playable in the browser, one sitting (~20–30 min):

1. Title → New Hunt as **Clare, No.47**.
2. Island map. Doga is burning.
3. Hex hunt: two yoma in a ruined hamlet. Learn Cut, Guard, Raise Trans, Yoki Sense.
4. Town: tavern board, rest, Raki joins as support.
5. Paburo: Miria + Helen appear. 3 yoma, one with stretch. Recruit.
6. Gonal: **Ophelia** — 4-cell awakened, ripple zone, severable arm.
7. Pieta rally (optional): 5-cell worm, Deneve recruit, first real wipe risk.
8. Rank, ledger, save. Codex of the design.

Save is local (`claymore.save.v1`). Signed-in hunters also write a cloud row.

---

## 14. Blade (Rust) — one renderer, two windows

Claymore is a **Blade** game. The old note that "there is no wasm path" is wrong: `blade-graphics` already targets **WebGL2**, and `blade-render` now has a **rasterization** pipeline (`blade-render::Rasterizer`, `raster.wgsl`) that does not need hardware ray tracing. `blade-engine` picks it with `RenderBackend::Rasterizer`. The full RT engine stays a native extra, not the hunt view.

```
Claymore/                  one crate (`claymore`)
  rust/                    hex, combat, iso camera, prism mesh, HuntBoard
  src/game/                web slice — same rules, same iso math
    sim/                   mirrors rust/hex + rust/combat
    render/hex-canvas      projects the prism mesh until wasm Rasterizer lands
```

### What runs where

| Surface | Backend | Notes |
| --- | --- | --- |
| Hunt (this preview) | Canvas 2D isometric of the **same prism mesh / 30° camera** | Playable now. Drag to pan, wheel to zoom. |
| Hunt (native) | `blade-render::Rasterizer` + instanced hex prisms | same crate, later a windowed bin |
| Hunt (web, next) | same Rasterizer on `blade-graphics` WebGL2 | `wasm32-unknown-unknown` |
| Island / town | still the painted map + slides | Blade heightmesh later |
| Ray-traced beauty | `RenderBackend::RayTracer` | Vulkan + RT hardware only. Not the game. |

### Scene plan (unchanged, now isometric)

| Mode | What Rasterizer draws |
| --- | --- |
| Hunt | Instanced hex prisms (`unit_hex_prism`), decal zones, billboard units. No Rapier — the grid is the physics. Camera is isometric (narrow FOV from `hunt_camera`). |
| Island | Heightmesh + Kenney Nature / Foliage instances, one walker capsule. |
| Town | Authored diorama or the existing 2D slide. |

Determinism: `claymore-sim` ticks on a fixed seed. Blade presents. The TS sim is a port, not a second design.

### How this repo gets to GitHub

Remote is [kvark/claymore-blade](https://github.com/kvark/claymore-blade). GitHub Pages is [kvark.github.io/claymore-blade](https://kvark.github.io/claymore-blade/) (`GITHUB_PAGES=1` static SPA, Actions on `main`). `cargo test` needs no GPU.


---

## 15. Kenney Game Assets All-in-1 — what to pull

You already bought [Kenney Game Assets](https://kenney.itch.io/kenney-game-assets). Do **not** dump the whole 60k. Extract these folders into `assets/kenney/` (native) / `public/kenney/` (web).

### Pull now (web + native greybox)

| Pack | Use |
| --- | --- |
| **Hexagon Pack** | Hunt floor variants, overlays, selection |
| **Hexagon Base Pack** | Extra terrain hexes (mud, water, height) |
| **Hexagon Buildings Pack** | Village / ruin props on hexes |
| **Isometric Nature** | Pines, rocks, cliffs on the island and hunt edges |
| **Isometric Medieval Town** | Hanel / Stora roofs as map icons |
| **Isometric Miniature Dungeon** | Witch's Maw / indoor hunts |
| **RPG Tileset** | Town slide backgrounds, tavern floor |
| **UI Pack** + **UI Pack: RPG extension** | Restyle to steel; bars, panels, buttons |
| **Input Prompts** | WASD / click glyphs on the first hunt |
| **Cursor Pack** | Blade cursor, hunt cursor |
| **Particle Pack** | Slash puffs, dust, yoki wisps (recolor) |
| **Rune Pack** | Organization marks, rank board (pair with `symbols.gif`) |
| **Impact / RPG Audio** | Hits, blocks, UI ticks |
| **Interface Sounds** | Board, save, hover |
| **Music Loops** (any dark / ambient) | Island bed; mute-able |

### Pull for the Blade port (3D)

| Pack | Use |
| --- | --- |
| **Nature Kit** + **Nature Kit (Classic)** + **Foliage Pack** | Island scatter |
| **Castle Kit** | Sutafu, Witch's Maw |
| **Graveyard Kit** | Pieta, abandoned beacons |
| **Modular Dungeon Kit** + **Modular Cave Kit** | Indoor hunts |
| **Mini Dungeon** | Fast blockout |
| **Furniture Kit** | Tavern interiors |

### Do not pull

Space, tanks, racing, platformer, pirate, holiday, googly eyes, sci-fi interiors, minigolf. They fight the island.

### How they sit next to authored art

Kenney is **terrain, props, UI bones, audio**. Faces, awakened bodies, cloaks, rank-accurate silhouettes stay authored (your sketches + generated Claymore sheets). Recolor Kenney hexes to the ink/ash/blood palette — do not leave them toy-bright.

---

## 16. Systems left for later (not forgotten)

- Full character creation (look, sex, pre-history, half/quarter).
- Pause-RT combat mode as a toggle.
- Possess, demon unions, Phase 2 campaign.
- MMO note from Oct 7: ignore. Single-player + later co-op hunts, not an MMO.
- Teaching skills to other silvers as a relationship action.
- Location-based transformation (advanced Perception).

---

## 17. Success for the prototype

- You can walk the island and feel the beacons as a clock.
- A 1-cell yoma fight teaches the language.
- A 4-cell awakened fight makes zonal damage obvious: you cut an arm off, the ripple dies, the core still kills you if you stand in the next ring.
- Quicksword, Stretching, Phantom, Drill Sword, Yoki Sense are in and readable.
- Trans is tempting and stupid in equal measure.
- It looks like the sketches and the series, not like a UI kit.
