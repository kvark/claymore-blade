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

## 5. Story spine (anime-faithful)

The campaign is not a retelling of every episode. It is Clare's path through the same island the series used, with the same wounds, in a form that fits beacons, ranks, and hex hunts.

### 5.1 What the player is

Clare is No. 47. She is half yoma by design and more than half by choice: Teresa of the Faint Smile's flesh is inside her. The Organization ranked her near the bottom and sent her out to die usefully. She is quiet, exact, and wrong for a rank that low. She does not speechify. When she speaks, it is short and aimed.

The player does not "become the hero of the north." The player walks beacons, cuts what wears human skin, and decides how much of Teresa's power to open. Priscilla is the long shadow; she is not the vertical-slice boss.

### 5.2 Act structure

| Act | Anime anchor | Play |
| --- | --- | --- |
| **I · The brand** | Episodes 1–3 (Doga, Raki, first hunt) | Title → island → Doga beacon → first yoma → Raki attaches |
| **II · Sisters on the road** | Paburo Mountains, Miria / Helen / Deneve | Recruit after shared hunts; learn party lead is rank, not kindness |
| **III · The smiling one** | Ophelia | First multi-cell awakened; sever the arm or die in the ripple |
| **IV · The north** | Pieta / Northern Campaign | Time-limited rally; worm; Deneve's refusal to die; optional wipe |
| **V · The Organization** (post-slice) | Sutafu, staff, trials | Rank board, orders you may refuse, Irene's arm, Galatea's eye |
| **VI · The abyss** (later) | Priscilla | Not in v1. Campaign shadow only. |

### 5.3 Themes the systems must serve

- **Expendable silver.** The Organization does not love you. Rank is a tool. Disobey and the map changes (search parties, locked beacons, Sutafu summons).
- **Teresa's ghost.** Clare is not Teresa. Every high-trans choice is borrowing a dead No.1. Dialog and UI never say "power-up"; they say *open the bar*.
- **Human weight.** Raki is not comic relief. He is the floor under the trans-meter. Without a human nearby, lowering the bar is harder.
- **Named sisters, not loot.** Miria, Helen, Deneve, Ophelia are people with ranks and habits. Recruiting them is political, not a shop purchase.
- **Beacons are clocks.** Late arrival is not a fail-state title card; it is fewer living, then an empty nest that hits harder.

### 5.4 Vertical-slice beat sheet

1. **Title.** Claymore. No. 47. New Hunt / Continue.
2. **Intro (no choice).** Organization branded you, ranked you, sent you. Doga has lit a beacon. Walk there. Do not raise the bar unless you must.
3. **Island, first light.** Doga pulses. Stora is quiet. Sutafu is a cold pin in the east.
4. **Doga, before the fight.** Elder lies. Child stares. Something in the well wore a neighbor yesterday.
5. **Hunt: two yoma.** Teach Cut, Guard, Wait, Raise. Raki watches from the edge (off-board support if unlocked).
6. **After Doga.** Raki will not stay. He follows. You may refuse him once; he follows anyway at the next town.
7. **Road to Paburo.** Miria's aura on the map. She is already hunting. You are late to *her* board.
8. **Paburo nest.** Stretch-limb yoma. Miria and Helen can be fought beside, then spoken to. Recruit if karma and rank allow.
9. **Gonal.** Ophelia is four hexes of wrong anatomy. She smiles. She wants a "friend." Sever the arm that makes the ripple.
10. **Pieta (optional).** North gathers. Highest rank leads. Worm occupies five cells. Deneve does not die cleanly.
11. **Result / ledger.** Rank ticks. Karma ticks. Codex opens names you have met.

---

## 6. Characters (voice, role, series debt)

Series names. Stats and skills stay in the technique tables; this section is **how they talk and why they move**.

### 6.1 Clare — No. 47 (playable)

| | |
| --- | --- |
| **Anime** | Quiet survivor. Teresa's arm and will. Hunts Priscilla across the whole story. |
| **Voice** | Short sentences. No jokes. Questions are rare and practical. |
| **Want** | Cut the thing that killed Teresa. Survive long enough to do it. |
| **Need** | Accept that sisters are not tools; that Raki is not a weight. |
| **Trans** | She opens the bar faster than a No. 47 should. UI copy treats this as *danger*, not *cool*. |
| **Tell** | Silver eyes, pale hair, sword too long for her frame. Stillness before a cut. |

Sample lines (Clare):

- "I'm here for the yoma."
- "Stay behind me."
- "I won't stop."
- "That one is already dead. It just hasn't fallen."

### 6.2 Raki — human companion

| | |
| --- | --- |
| **Anime** | Boy from Doga. Family killed by yoma. Follows Clare; becomes her reason to stay human-shaped. |
| **Voice** | Earnest, too loud in quiet rooms, brave past sense. |
| **Role in systems** | Off-board support: lure, morale, **trans drop**. Without him, `Trans Drop` needs high Control. |
| **Want** | Become strong enough that Clare does not leave him. |
| **Tell** | Travel cloak, empty hands, stands where he should not. |

Sample lines (Raki):

- "I'm going with you. I don't care if you say no."
- "You're not a monster. I saw you choose."
- "If I stay close, you can come back down. Right?"

### 6.3 Teresa of the Faint Smile — No. 1 (fallen, memory)

Not a unit in v1. She is **dialog color and trans flavor**.

- Clare hears her in high-trans moments (one line, never a conversation).
- Hanel has statues and wrong stories about her.
- Irene will speak of her as a fact, not a legend.

Sample (memory, not voiced NPC):

- "You are still too soft."
- "Smile when you cut. It unsettles them."

### 6.4 Miria — No. 6

| | |
| --- | --- |
| **Anime** | Phantom. Deserter. Organizes the hidden rebellion. Cold tactician who still bleeds for her sisters. |
| **Voice** | Measured, command tone, rare warmth under steel. |
| **Recruit** | After Paburo, if you did not abandon the nest and your karma is not scavenger-low. |
| **Want** | Pull silvers out from under the Organization. |
| **Party** | Highest rank among early recruits → she is map-leader when present. |

Sample lines (Miria):

- "You're late, No. 47. Try not to die before you're useful."
- "Phantom is not speed. It is being somewhere they have already looked."
- "We do not serve the Organization. We survive it."

### 6.5 Helen — No. 22

| | |
| --- | --- |
| **Anime** | Stretching limbs. Crude, loyal, laughs at the wrong time. |
| **Voice** | Rough, teasing, suddenly serious when blood is real. |
| **Recruit** | With Miria or after a stretch-yoma hunt where she sees you cut clean. |
| **Combat tell** | Line-4 reach; dialog jokes about "long arms." |

Sample lines (Helen):

- "Rank 47? Cute. Don't trip on that sword."
- "I can hit them from here. You go get the ugly one."
- "Deneve's the careful one. I'm the fun one. Try to keep up."

### 6.6 Deneve — No. 15

| | |
| --- | --- |
| **Anime** | Regeneration. Calm. The one who should have died and did not. |
| **Voice** | Quiet, precise, almost soft. No wasted words. |
| **Recruit** | Pieta or a hunt where a limb is lost and restored. |
| **Combat tell** | Regen skill; she is the player's lesson that "dead" is not always final for silvers. |

Sample lines (Deneve):

- "I can put it back. Once. Don't make a habit of losing pieces."
- "Helen talks. I watch. Both are useful."
- "If the north is a trap, we walk into it with open eyes."

### 6.7 Ophelia — No. 4 (boss, then absence)

| | |
| --- | --- |
| **Anime** | Ripple Blade. Sadist. Obsession with "friends." Awakens. |
| **Voice** | Bright, wrong, affectionate to prey. |
| **Role** | Gonal boss. Multi-cell. Ripple zone. Severable arm. |
| **After** | She does not join. Her absence is the point. |

Sample lines (Ophelia):

- "You look like someone I could keep."
- "Don't run. Friends don't run."
- "Ripple. Ripple. There — your legs forgot the ground."

### 6.8 Irene — No. 2 (ex)

Teacher of Quicksword. One arm given to Clare in the series. In the game: a **road encounter / Sutafu thread**, not a permanent party member in the slice.

Sample lines (Irene):

- "That cut is mine. You're still borrowing it."
- "Quicksword is not a swing. It is a decision that finished before they blinked."
- "If you meet Priscilla, do not smile. She will."

### 6.9 Galatea — No. 3

God-Eye. Reads intent. Organization leash until she is not. UI telegraph skill is *her* signature.

Sample lines (Galatea):

- "I already saw which way you step. Choose the other."
- "The Organization thinks I am a lantern. Lanterns burn both ways."

### 6.10 Flora — No. 8 / Jean — No. 9

Windcutter and Drill Sword. Present as **codex + optional hunt allies** before full recruit trees.

- Flora: precision, not haste. "The wind does not rush. It arrives."
- Jean: single-hex devastation. "One hole. Through everything."

### 6.11 Priscilla — the abyss

Campaign shadow only in v1. Name appears in Clare's rare high-trans lines and in Hanel's wrong statues. No fight, no map pin you can walk to.

### 6.12 Organization voices

| Role | Voice |
| --- | --- |
| **Handler / staff** | Bureaucratic calm. You are inventory. |
| **Board notice** | Rank, deadline, region, no sympathy. |
| **Punishment line** | Soft. Final. |

Sample (staff):

- "No. 47. Doga. Before the moon turns. Alive preferred. Not required."
- "Failure is data. Try not to be interesting data."

---

## 7. Dialog banks (implementable)

All lines are short enough for the 5×7 font and for mobile. Tags match mode / trigger.

### 7.1 Intro crawl

```
You are Clare, No. 47.

The Organization branded you, ranked you,
and sent you to the island to cut the things
that wear human skin.

Doga has lit a beacon.
Walk there.
Do not raise the bar unless you must.
```

### 7.2 Title / menu flavor (rotate)

- "Silver eyes see what men will not."
- "Rank is a number. The bar is a choice."
- "They call you a Claymore. You are a knife with a name."

### 7.3 Island — first arrival at Doga

**Elder (town slide):**

- "You're… one of them. Good. Something in the well has been wearing faces."
- "We lit the beacon three nights ago. Two families are gone."

**Raki (after hunt or at gate):**

- "You killed it. You killed the thing that took them."
- "I have nowhere left. I'm coming."

**Clare (player prompts):**

- [Accept] "Keep up."
- [Refuse] "You'll die on the road." → Raki: "Then I'll die closer to it than here."

### 7.4 Hunt briefings

| Hunt | Briefing text |
| --- | --- |
| **Doga · the well** | Two of them wore neighbors yesterday. A cut is enough. Raise the bar if you must. Do not max it. |
| **Paburo · the nest** | Miria is already here. Stay out of the long lines. The nest has learned to reach. |
| **Gonal · Ophelia** | Ophelia is four hexes of wrong anatomy. Cut the arm. Cut the tail. Do not accept her friendship. |
| **Pieta · the north** | Highest rank leads. The worm occupies five cells. Sever the tail or it turns the board into a throat. |

### 7.5 Combat bark (log lines)

Keep the existing mechanical log (`Clare → Yoma: Solid 22`). Add optional flavor inserts:

| Tag | Line |
| --- | --- |
| raise | "Clare opens the bar." |
| raise_high | "The silver in her eyes goes thin." |
| sever | "The arm learns it is optional." |
| raki_drop | "Raki's voice cuts through. The bar falls." |
| miria_phantom | "Miria is already gone from where they aimed." |
| helen_stretch | "Helen takes the far hex without stepping." |
| ophelia_ripple | "The ground forgets who owns it." |
| death_yoma | "The face stops being a face." |
| death_silver | "A number goes dark." |

### 7.6 Town services

**Hunt / Rest / Leave** stay as buttons. Flavor under them:

- Rest: "Sleep is a low bar. Take it."
- Leave: "The road does not care about rank."
- Board empty: "No new beacons. The island is holding its breath."

### 7.7 Recruit beats

**Miria (Paburo, after victory):**

- Miria: "You cut clean. Most low ranks flinch."
- Clare: "…"
- Miria: "Walk with us until the next nest. Or don't. The Organization will not mourn either choice."
- [Recruit] / [Not yet]

**Helen:**

- "If Miria's in, I'm in. Somebody has to make the jokes."

**Deneve (Pieta):**

- "I will not die here. That is not a boast. It is a schedule."
- [Recruit]

### 7.8 Ophelia encounter (pre-fight)

- Ophelia: "A little sister. How nice."
- Clare: "I'm here to end you."
- Ophelia: "That's what friends are for."

### 7.9 Failure / victory stingers

**Victory:**

- "The board is quiet. You walk back with blood on the silver. The beacon goes dark."

**Defeat:**

- "You fall. The Organization will send another number."

**Late beacon (map):**

- "You arrive to ash. The nest has moved into the cellars."

---

## 8. World map design

### 8.1 Regions (feel)

| Region | Climate / palette | Emotion |
| --- | --- | --- |
| **Lautrec** | Mud, wet pine, low cloud | First blood, villages that lie to themselves |
| **Toulouse / central roads** | Dust-stone, patrols | You are seen; Lacroa search parties |
| **Highlands (Paburo)** | Thin air, black trees | Nest logic; Miria's hunting ground |
| **Mucha (south)** | Dry hills, iron soil | Ophelia; things that smile |
| **Alfons (north)** | Ice, wind, sparse towns | Pieta; party hunts; no second chances |
| **East / Sutafu** | Order, pale stone, no birds | The brand that made you |

### 8.2 Node layout (normalized 0–1 map coordinates)

Painted map art stays; pins are data. Coordinates match the playable crate.

| Id | Name | x | y | Kind | Encounter | Deadline (h) |
| --- | --- | --- | --- | --- | --- | --- |
| doga | Doga | 0.28 | 0.58 | village | doga-yoma | 36 |
| stora | Stora | 0.36 | 0.66 | village | — | — |
| hanel | Hanel | 0.33 | 0.48 | city | — | — |
| shire | Mount Shire | 0.50 | 0.32 | shrine | — | — |
| paburo | Paburo | 0.50 | 0.46 | wild | paburo-nest | 72 |
| lacroa | Lacroa | 0.62 | 0.50 | village | — | — |
| gonal | Gonal | 0.50 | 0.72 | village | gonal-ripple | 96 |
| pieta | Pieta | 0.48 | 0.20 | city | pieta-worm | 140 |
| maw | Witch's Maw | 0.18 | 0.40 | keep | — | — |
| sutafu | Sutafu | 0.78 | 0.48 | office | — | — |

### 8.3 Routes and travel

Not a graph the player draws; implied by walk speed and terrain cost:

```
Doga —— Stora —— Hanel
  \       |        \
   \      |         Lacroa —— Sutafu
    \   Paburo —— Shire —— Pieta
     \    |
      Gonal
Maw sits west of Hanel on a dead road (slow, optional lore).
```

| Terrain under the party | Hours per map unit |
| --- | --- |
| Road | 0.6× |
| Open / painted path | 1.0× |
| Highland / pine | 1.4× |
| Ice approach (Alfons) | 1.6× |
| Dead road to Maw | 1.8× |

Beacon deadlines tick in world-hours. Rest in town adds hours without movement.

### 8.4 Map read (UI)

| Element | Design |
| --- | --- |
| **Party** | Clare sprite or banner; slight bob |
| **Beacon** | Warm pulse (amber). Deadline chip if < 24h |
| **Cleared** | Cold green pin, no pulse |
| **Dead / late** | Black pin, no villagers on enter |
| **Locked** | Dim pin; click plays error; needs prior flag |
| **Aura haze** | Soft disc: green-gold warrior / dirty-red yoma / white-violet awakened |
| **Sutafu** | Always visible; does not pulse unless summoned |

**Perception on the map.** Auras render as colored haze. Strength falls with distance. High Perception prints a verb: *feeding, fleeing, waiting, hunting you*.

**Parties.** Some hunts require three or more silvers at a rally hex before a world-deadline. On the global map the highest rank is the party leader (you still steer in local space). In combat you command the whole cell.

Unlock chain for the slice:

```
Doga (start Beacon)
  → clear Doga → Paburo unlocks
  → clear Paburo → Gonal unlocks
  → clear Gonal → Pieta unlocks
Sutafu / Maw / Hanel / Shire / Lacroa / Stora: always enterable as town slides (lore, rest, board), no hunt until later acts.
```

### 8.5 Town slides (what you see)

| Town | Visual beat | Dialog hook |
| --- | --- | --- |
| Doga | Mud, well, thin goats | Elder + Raki |
| Stora | Market stalls, board | Rumors, first non-hunt board |
| Hanel | Teresa statues, wrong plaques | Lore dump; "the girl she saved" |
| Shire | Wind, shrine, one bed | Dying sister promise (seed for later) |
| Paburo | Black trees, no smoke | Miria's camp traces |
| Lacroa | Posters with numbers | Search parties; Organization pressure |
| Gonal | Dry square, too clean | Ophelia was "helpful" here |
| Pieta | Cold stone, muster yard | Rally timer, highest rank |
| Maw | Broken keep, Zakol shadow | Optional dread; no slice boss |
| Sutafu | Pale halls, rank board | Orders, punishment, Irene thread |

### 8.6 What the map must teach without a tutorial box

1. Beacons are **time**.
2. Rank is **visible** in the chrome (No. 47).
3. North is **harder** (distance + deadline + party).
4. East is **watching** (Sutafu never looks friendly).
5. You can walk past a fight; the pin may go black.

---

## 9. Attributes

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

## 10. Trans-meter (yoki)

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

## 11. Combat (hex, turn-based)

### 11.1 Board

- Pointy-top axial hexes.
- Typical hunt: 11 × 9. Pieta / Maw: 13 × 11.
- Terrain: grass, road, mud (cost 2), ruin (cover: +block), water (impass), height (advantage on Pa).

### 11.2 Turn

Initiative = Agility + 1d10, rolled once.
On your turn you get **2 AP**:

- Step to a neighbor = 1 AP (2 in mud)
- Skill / attack = 1–2 AP
- Raise or hold trans = 0 AP (once)
- Sprint (2 AP, no skill)
- Wait / Rest (2 AP, regen, drop trans slightly)

Queue is visible. Interrupts exist only as skills (Phantom, God-Eye). The old interruption-penalty system is reserved for the optional pause-RT mode.

### 11.3 Hit table

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

### 11.4 Multi-cell monsters

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

### 11.5 Zones

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

## 12. Techniques (from the series)

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

## 13. Progression, rank, karma

- Rank is an Organization number, 47 → 1. It changes how silvers speak to you and which beacons you are offered.
- Ledger: demons, awakened, silvers, humans, missions.
- Karma is a signed int from that ledger plus choices (save the village / make it in time / refuse an order / kill a human).
- Karma shifts prices, lines, and whether a silver will stand with you or hunt you.
- Organization **authority** is separate. Obey: +small. Disobey: −20 to −30. Cross a hard rule (kill a human, refuse a purge, approach awakening in public): the Organization turns.

---

## 14. Playing as a demon (Phase 2)

Unlocked if Clare awakens, or as a New Hunt option later.

- No XP, no new skills, no attribute ups.
- Hunger: one week without feeding is death.
- Trans is free in both directions. Human-form is paper.
- Demon unions (the old No.1s) have territories. You pick a side or you are meat.

Not in the vertical slice except as a lose state with a short epilogue.

---

## 15. Vertical slice (what ships in this prototype)

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

## 16. Blade (Rust) — one renderer, two windows

Claymore is a **Blade** game. The old note that "there is no wasm path" is wrong: `blade-graphics` already targets **WebGL2**, and `blade-render` now has a **rasterization** pipeline (`blade-render::Rasterizer`, `raster.wgsl`) that does not need hardware ray tracing. `blade-engine` picks it with `RenderBackend::Rasterizer`. The full RT engine stays a native extra, not the hunt view.

```
Claymore/
  crates/
    claymore-sim/      # hex, hit table, scale. no GPU. source of truth
    claymore-scene/    # isometric camera + hex-prism mesh (CPU)
    claymore-view/     # HuntBoard + (feature gpu) Rasterizer wiring
  src/game/            # web vertical slice: same rules, same iso math
    sim/               # mirrors claymore-sim (TS)
    render/hex-canvas  # projects claymore-scene prisms in 2D until wasm lands
```

### What runs where

| Surface | Backend | Notes |
| --- | --- | --- |
| Hunt (this preview) | Canvas 2D isometric of the **same prism mesh / 30° camera** | Playable now. Drag to pan, wheel to zoom. |
| Hunt (native) | `blade-render::Rasterizer` + instanced hex prisms | `cargo run -p claymore-view` once `gpu` feature is wired to a window |
| Hunt (web, next) | same Rasterizer on `blade-graphics` WebGL2 | `wasm32-unknown-unknown`, canvas id `blade` |
| Island / town | still the painted map + slides | Blade heightmesh later |
| Ray-traced beauty | `RenderBackend::RayTracer` | Vulkan + RT hardware only. Not the game. |

### Scene plan (unchanged, now isometric)

| Mode | What Rasterizer draws |
| --- | --- |
| Hunt | Instanced hex prisms (`claymore-scene::unit_hex_prism`), decal zones, billboard units. No Rapier — the grid is the physics. Camera is isometric (narrow FOV from `hunt_camera`). |
| Island | Heightmesh + Kenney Nature / Foliage instances, one walker capsule. |
| Town | Authored diorama or the existing 2D slide. |

Determinism: `claymore-sim` ticks on a fixed seed. Blade presents. The TS sim is a port, not a second design.

### How this repo gets to GitHub

Remote is [kvark/claymore-blade](https://github.com/kvark/claymore-blade). Commit the crates + the isometric slice and push `main`. Native `cargo test` does not need a GPU. WebGL Rasterizer is the next compile, not a rewrite.


---

## 17. Kenney Game Assets All-in-1 — what to pull

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

## 18. Systems left for later (not forgotten)

- Full character creation (look, sex, pre-history, half/quarter).
- Pause-RT combat mode as a toggle.
- Possess, demon unions, Phase 2 campaign.
- MMO note from Oct 7: ignore. Single-player + later co-op hunts, not an MMO.
- Teaching skills to other silvers as a relationship action.
- Location-based transformation (advanced Perception).

---

## 19. Success for the prototype

- You can walk the island and feel the beacons as a clock.
- A 1-cell yoma fight teaches the language.
- A 4-cell awakened fight makes zonal damage obvious: you cut an arm off, the ripple dies, the core still kills you if you stand in the next ring.
- Quicksword, Stretching, Phantom, Drill Sword, Yoki Sense are in and readable.
- Trans is tempting and stupid in equal measure.
- It looks like the sketches and the series, not like a UI kit.
