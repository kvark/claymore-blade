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
| Organization ranks, black cards, handlers | Soft presence. Rank is a number on the HUD and a social cue. |
| Awakened as multi-hex bosses | Yes. Footprint + zonal attacks. |
| Limb loss / regeneration | Zonal damage and "sever" outcomes. |
| Party of warriors | Recruits via dialog choice after key hunts. |

---

## 3. Systems that ship in the slice

- **Trans-meter** (0–100). Skills cost and raise it. High values unlock Teresa techniques and risk awakening cues.
- **Hex combat** with facing, zones, and AI turns.
- **World map** with towns, beacons, hours, and soft failure on lateness.
- **Scene mode** for interactive dialogs and recruit choices (Raki, Miria/Helen, Deneve, Ophelia pre-fight, Doga elder).
- **Save** between islands / after fights.

---

## 4. What stays out of v1

- Full character creation / gender / half-vs-quarter.
- Pause real-time combat mode.
- Online or multiplayer.
- Full Organization politics and black-card side quests.
- Every anime arc (keep a focused spine).

---

## 5. Story spine (anime-faithful)

Keep the emotional through-line of the early anime without retelling every episode. The player experiences a compressed, choice-aware slice:

1. **Doga** — first beacon. Quiet village. Raki is already there, watching. After the yoma is down, he asks to travel with you. Choice: accept or soft-refuse (he still trails).
2. **Road and small nests** — short hunts that teach meter, zones, and ranking talk.
3. **Paburo mountains** — larger nest. After victory, Miria and Helen appear. They have been hunting the same trail. Recruit choice (Miria + Helen as a package).
4. **Gonal / Ophelia** — the ripple. Ophelia is waiting. Pre-fight dialog is mandatory the first time; she is playful, dangerous, and ranks you. Then the fight.
5. **Pieta** — the northern front. After the worm, Deneve is among the survivors. Recruit choice.
6. **Open island** — remaining beacons, optional side nests, and the growing sense that the Organization is watching the meter and the company you keep.

The spine ends before the full northern war and the later Clare/Teresa revelations. Those are reserved for a later chapter. The player should feel the weight of ranks, the cost of the meter, and the value of the few people who choose to stand with a No. 47.

---

## 6. Characters (voice / role)

| Character | Role in slice | Voice notes |
| --- | --- | --- |
| **Clare** | Player. Quiet, precise, carries Teresa. | Short sentences. Rare emotion. When the meter rises, the lines harden. |
| **Raki** | Human boy from Doga. Follows. | Earnest, a little loud, protective beyond his strength. Asks questions the warriors will not. |
| **Miria** | No. 6. Leader of the small band. | Calm, analytical, ranks everything. Speaks of survival and of the Organization without heat. |
| **Helen** | No. 22. Elastic, irreverent. | Dry jokes, stretch-limb quips, calls Clare out when she goes silent. |
| **Deneve** | No. 15. Regenerator, weary. | Practical, low energy, does not waste words. Trust is slow. |
| **Ophelia** | No. 4. The ripple. Antagonist of the mid-slice. | Playful, cruel, ranks you to your face. Treats the fight as a game until it is not. |
| **Doga elder** | Town voice. | Tired, grateful, afraid of what the beacon means. |

Party composition after recruit choices shapes banter and who stands on the hexes. Raki is non-combat (or very limited) in v1; he is presence and stakes.

---

## 7. Dialog banks (implementable)

All lines are short speaker + text. SceneState walks a list; at the end it offers 0–2 choices. Flags gate once-only scenes.

### Intro crawl

A few lines on the title → world transition: the brand, the beacons, the order to walk the island.

### Town — Doga (elder)

Once-only. Gratitude mixed with the knowledge that the Organization will send someone else if you fail.

### Raki join (after Doga yoma)

Raki asks to travel. Yes → flag raki. No → flag raki-refused (he still appears at the next town).

### Recruit — Paburo (Miria + Helen)

After the nest. They have been on the same trail. Yes adds both to party.

### Recruit — Pieta (Deneve)

After the worm. Survivors. Yes adds Deneve.

### Ophelia pre-fight (Gonal ripple)

Mandatory first time. She ranks Clare, toys with the idea of a game, then the fight starts. Flag ophelia-spoken.

### Combat barks

Short, meter-aware lines for Clare and party on hit / raise / near-awaken. Not full conversations.

### Result screens

Win / lose titles and bodies that can reference the hunt id lightly.

Implementation lives in `dialog.rs` (banks + SceneState) and is driven from `game.rs` (Mode::Scene, pending_encounter, finish_combat_id queues).

---

## 8. World map design

### Regions and feel

- **Southern lowlands** (Doga start) — roads, small farms, first beacons. Safe-ish.
- **Central hills / Paburo** — denser forest, larger nests, first multi-warrior encounters.
- **Northern approaches / Pieta** — colder, more Organization presence, the sense of a front.
- **Side paths** — abandoned mines, ruined watch posts, optional hunts that do not advance the spine.

### Pins and unlock chain

- Towns unlock by proximity + story flags.
- Beacons appear when the Organization (or the village) lights them; they go dark on a timer if ignored.
- After Doga → road opens toward Paburo.
- After Paburo recruit → path toward Gonal / Ophelia becomes clear.
- After Ophelia → northern route to Pieta soft-unlocks.
- Codex entries and rank chatter unlock with the same flags.

### Town slides

Each town has a short background + 1–2 interactable NPCs (elder, merchant, optional warrior). The Doga elder scene is the template: enter town → optional talk → leave or rest (hours).

### Routes

Roads are free movement corridors. Leaving the road into the wild costs more hours and can trigger random weak nests. The player should feel the island as a place, not a menu of mission select.

---

## 9. Combat (hex)

(unchanged core from earlier design — zones, facing, trans-meter costs, multi-hex awakened, AI turns.)

---

## 10. UI and modes

Title → Intro crawl → World map → Town → Hunt (hex) → Result → (optional Scene for recruit / Ophelia) → World.

Scene mode reuses the same panel language as result / town: speaker, text, continue prompt or yes/no buttons.

---

## 11. Technical notes

One crate. Blade for native + web. Catalog of encounters, skills, art. Persist world + party + flags. Scene banks are static data; choices write flags and party.

---

## 12. Open questions for later

- How much Teresa voice leaks into Clare lines at high meter.
- Whether Raki ever gets a combat role.
- Full northern arc and Organization confrontation as chapter 2.
