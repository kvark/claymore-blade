import { SKILLS } from "@/game/data/catalog";
import { canUse, currentUnit, living } from "@/game/sim/combat";
import { useGame } from "@/game/store";

export function CombatHud() {
  const combat = useGame((s) => s.combat);
  const world = useGame((s) => s.world);
  const ui = useGame((s) => s.ui);
  const setSkill = useGame((s) => s.setSkill);
  const combatAct = useGame((s) => s.combatAct);
  if (!combat) return null;
  const actor = currentUnit(combat);
  const mine = actor?.side === "player";

  return (
    <div className="pointer-events-none absolute inset-0 z-10 flex flex-col justify-between p-3 sm:p-4">
      <div className="flex items-start justify-between gap-3 pr-24">
        <div className="pointer-events-auto max-w-md rounded-lg border border-line bg-fog px-4 py-3">
          <p className="font-display text-xs tracking-[0.22em] text-dust uppercase">
            {combat.title} · Round {combat.round}
          </p>
          <p className="mt-1 text-xs leading-5 text-ash/80">{combat.briefing}</p>
          <ol className="mt-2 flex flex-wrap gap-1">
            {combat.order
              .map((id) => combat.units.find((u) => u.id === id))
              .filter(Boolean)
              .map((u) => (
                <li
                  key={u!.id}
                  className={`rounded-sm px-1.5 py-0.5 text-[10px] ${
                    u!.dead
                      ? "text-dust/40 line-through"
                      : u!.id === actor?.id
                        ? "bg-steel text-ink"
                        : "bg-ink/40 text-dust"
                  }`}
                >
                  {u!.name}
                </li>
              ))}
          </ol>
        </div>
      </div>

      <div className="flex flex-col gap-3 lg:flex-row lg:items-end">
        <div className="pointer-events-auto max-h-36 overflow-y-auto rounded-lg border border-line bg-fog px-3 py-2 text-[11px] leading-5 text-dust lg:w-72">
          {combat.log.slice(0, 8).map((l, i) => (
            <p key={`${l.t}-${i}`} className={l.kind === "death" || l.kind === "sever" ? "text-blood" : ""}>
              {l.text}
            </p>
          ))}
        </div>

        <div className="pointer-events-auto min-w-0 flex-1 rounded-xl border border-line bg-fog p-3">
          {actor && (
            <>
              <div className="flex items-center gap-3">
                <img src={actor.portrait} alt="" className="h-14 w-10 rounded-sm object-cover" />
                <div className="min-w-0 flex-1">
                  <div className="flex items-baseline justify-between gap-2">
                    <p className="font-display text-xl leading-none">{actor.name}</p>
                    <p className="text-[11px] text-dust tabular-nums">
                      AP {actor.ap}/{actor.maxAp}
                    </p>
                  </div>
                  <p className="text-[11px] text-dust">{actor.title}</p>
                  <Meter label="Flesh" value={actor.hp} max={actor.maxHp} tone="steel" />
                  <Meter label="Yoki" value={actor.yoki} max={actor.maxYoki} tone="steel" />
                  <Meter label="Trans" value={actor.trans} max={100} tone="blood" />
                </div>
              </div>
              {actor.parts.length > 1 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {actor.parts.map((p) => (
                    <span
                      key={p.id}
                      className={`rounded-sm border px-1.5 py-0.5 text-[10px] ${
                        p.hp <= 0 ? "border-blood/40 text-blood line-through" : "border-line text-dust"
                      }`}
                    >
                      {p.name} {p.hp}/{p.maxHp}
                    </span>
                  ))}
                </div>
              )}
              {mine && (
                <>
                <p className="mt-2 text-[11px] text-dust">
                  Pale tiles are steps. Drag the board, scroll to zoom. Choose a technique, then a cell.
                </p>
                <div className="mt-3 flex flex-wrap gap-1.5">
                  {actor.skills.map((id) => {
                    const s = SKILLS[id];
                    if (!s) return null;
                    const ok = canUse(actor, s, world.raki);
                    const on = ui.selectedSkill === id;
                    return (
                      <button
                        key={id}
                        type="button"
                        disabled={!ok}
                        onClick={() => {
                          if (s.self || s.shape === "self" || s.shape === "ripple") {
                            const core = actor.origin;
                            combatAct({ type: "skill", skillId: id, hex: core });
                            setSkill(undefined);
                            return;
                          }
                          setSkill(on ? undefined : id);
                        }}
                        className={`rounded-md border px-2.5 py-1.5 text-left text-[11px] ${
                          on
                            ? "border-steel bg-steel text-ink"
                            : ok
                              ? "border-line bg-raised text-ash hover:border-steel/40"
                              : "border-transparent bg-ink/30 text-dust/50"
                        }`}
                      >
                        <span className="block font-medium">{s.name}</span>
                        <span className="text-[10px] opacity-70">
                          {s.ap} AP
                          {s.trans ? ` · T${s.trans}` : ""}
                        </span>
                      </button>
                    );
                  })}
                  <button
                    type="button"
                    onClick={() => combatAct({ type: "raise" })}
                    disabled={actor.raisedTransThisTurn}
                    className="rounded-md border border-blood/40 px-2.5 py-1.5 text-[11px] text-ash hover:bg-blood/20 disabled:opacity-40"
                  >
                    Raise bar
                  </button>
                  <button
                    type="button"
                    onClick={() => combatAct({ type: "wait" })}
                    className="rounded-md border border-line px-2.5 py-1.5 text-[11px] text-dust hover:text-ash"
                  >
                    Wait
                  </button>
                </div>
                </>
              )}
              {!mine && <p className="mt-2 text-xs text-dust">The other side is moving.</p>}
            </>
          )}
        </div>

        <div className="pointer-events-none hidden w-36 flex-col gap-1 lg:flex">
          {living(combat).map((u) => (
            <div key={u.id} className="flex items-center gap-2 rounded-md border border-line bg-fog px-2 py-1">
              <span
                className="h-2 w-2 rounded-full"
                style={{ background: u.side === "player" ? "#c8ccd4" : "#9a2430" }}
              />
              <span className="truncate text-[11px]">{u.name}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function Meter({
  label,
  value,
  max,
  tone,
}: {
  label: string;
  value: number;
  max: number;
  tone: "steel" | "blood";
}) {
  const pct = Math.max(0, Math.min(100, (value / Math.max(1, max)) * 100));
  return (
    <div className="mt-1 flex items-center gap-2">
      <span className="w-10 text-[10px] tracking-wide text-dust uppercase">{label}</span>
      <div className="h-1.5 flex-1 rounded-full bg-ink/70">
        <div
          className={`h-full rounded-full ${tone === "blood" ? "bg-blood" : "bg-steel"}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <span className="w-10 text-right text-[10px] text-dust tabular-nums">{Math.round(value)}</span>
    </div>
  );
}
