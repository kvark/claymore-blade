import { LORE, SKILLS, WARRIORS } from "@/game/data/catalog";
import { useGame } from "@/game/store";

const chapters = [
  {
    t: "The brief",
    b: "Claymore is the October 2007 notes made playable. An island like Fallout 2. Hunts on a hex grid. Warriors occupy one cell. Awakened occupy many. Damage is zonal — you cut a limb off the board.",
  },
  {
    t: "What the notes kept",
    b: "Trans-meter. Discrete hits (miss / glance / blocked / solid). Scale 1 + 0.25×clamp(Pa−Pd). Perception auras. Time-dying villages. Rank, karma, a single human companion. Learning by watching and bleeding.",
  },
  {
    t: "What changed",
    b: "The notes wanted pause-RT. This hunt is turn-based hex. Pause-RT is archived, not deleted. The roster uses the television series: Clare, Miria, Helen, Deneve, Ophelia, Raki, the Organization.",
  },
  {
    t: "The bar",
    b: "Raise it at any time. Skills unlock in bands: 20 Aimed and Yoki Sense, 40 Quicksword and Phantom, 60 Drill Sword, 80 wings. At 90 the turn can vanish. At 100 you are the encounter.",
  },
  {
    t: "Zonal bosses",
    b: "Ophelia is four hexes: core, torso, tail, cutting arm. A zone that hits a limb does half and can sever it. Sever the tail and Ripple Blade dies. Only the core left is faster and smaller.",
  },
  {
    t: "Blade",
    b: "Native target is Blade (Rust): claymore-sim as the rules crate, blade-engine for the island and hunt scenes, egui for the tavern. This preview is the rules-accurate web slice — Vulkan stays on the metal.",
  },
];

export function CodexView() {
  const setMode = useGame((s) => s.setMode);
  const world = useGame((s) => s.world);
  const back = world.hours > 6 ? "world" : "title";

  return (
    <div className="absolute inset-0 overflow-y-auto bg-ink">
      <div className="mx-auto max-w-3xl px-5 py-10">
        <button
          type="button"
          onClick={() => setMode(back)}
          className="text-xs tracking-[0.2em] text-dust uppercase hover:text-ash"
        >
          Close
        </button>
        <h1 className="mt-4 font-display text-5xl font-semibold">Codex</h1>
        <p className="mt-2 text-sm text-dust">From the October notes, the sketches, and the series.</p>

        <div className="mt-8 space-y-8">
          {chapters.map((c) => (
            <section key={c.t}>
              <h2 className="font-display text-2xl">{c.t}</h2>
              <p className="mt-2 text-sm leading-7 text-ash/85">{c.b}</p>
            </section>
          ))}
        </div>

        <h2 className="mt-12 font-display text-2xl">Roster</h2>
        <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          {Object.values(WARRIORS).map((w) => (
            <figure key={w.id} className="overflow-hidden rounded-lg border border-line">
              <img src={w.portrait} alt={w.name} className="aspect-2/3 w-full object-cover" />
              <figcaption className="px-2 py-2">
                <p className="text-sm">{w.name}</p>
                <p className="text-[11px] text-dust">{w.title}</p>
              </figcaption>
            </figure>
          ))}
        </div>

        <h2 className="mt-12 font-display text-2xl">The island remembers</h2>
        <div className="mt-4 grid grid-cols-2 gap-3 sm:grid-cols-4">
          {LORE.map((w) => (
            <figure key={w.id} className="overflow-hidden rounded-lg border border-line">
              <img src={w.portrait} alt={w.name} className="aspect-2/3 w-full object-cover" />
              <figcaption className="px-2 py-2">
                <p className="text-sm">{w.name}</p>
                <p className="text-[11px] text-dust">{w.title}</p>
              </figcaption>
            </figure>
          ))}
        </div>

        <h2 className="mt-12 font-display text-2xl">Techniques</h2>
        <ul className="mt-4 divide-y divide-line border-y border-line">
          {Object.values(SKILLS)
            .filter((s) => !["claw", "lunge", "slam", "whip"].includes(s.id))
            .map((s) => (
              <li key={s.id} className="py-3">
                <p className="text-sm font-medium">
                  {s.name}
                  <span className="ml-2 text-[11px] font-normal text-dust">
                    T{s.trans} · {s.shape}
                  </span>
                </p>
                <p className="mt-1 text-xs leading-5 text-dust">{s.blurb}</p>
              </li>
            ))}
        </ul>

        <h2 className="mt-12 font-display text-2xl">Kenney — pull these</h2>
        <p className="mt-2 text-sm leading-7 text-dust">
          From the All-in-1 pack: Hexagon Pack + Base + Buildings; Isometric Nature, Medieval Town,
          Miniature Dungeon; RPG Tileset; UI Pack + RPG extension; Input Prompts; Cursor Pack;
          Particle Pack; Rune Pack; RPG / Interface audio. For Blade later: Nature Kit, Castle Kit,
          Graveyard Kit, Modular Dungeon / Cave. Leave the tanks and spaceships in the box.
        </p>
      </div>
    </div>
  );
}
