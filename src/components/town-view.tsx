import { ENCOUNTERS, locById, WARRIORS } from "@/game/data/catalog";
import { clockLabel } from "@/game/sim/world";
import { useGame } from "@/game/store";
import { asset } from "@/lib/asset";

export function TownView() {
  const world = useGame((s) => s.world);
  const setMode = useGame((s) => s.setMode);
  const restTown = useGame((s) => s.restTown);
  const startEncounter = useGame((s) => s.startEncounter);
  const loc = locById(world.lastTown ?? "doga");
  if (!loc) return null;
  const enc = loc.encounter ? ENCOUNTERS[loc.encounter] : undefined;
  const st = world.locations[loc.id];
  const huntReady = !!enc && !!st && (st.status === "beacon" || st.status === "dead");
  const backdrop =
    loc.art ??
    (loc.kind === "village" || loc.kind === "city" ? asset("/art/tavern.jpg") : asset("/art/battle-doga.jpg"));

  return (
    <div className="absolute inset-0">
      <img src={backdrop} alt="" className="absolute inset-0 h-full w-full object-cover" />
      <div className="absolute inset-0 bg-ink/70" />
      <div className="relative mx-auto flex min-h-dvh max-w-3xl flex-col justify-end px-5 py-8 sm:justify-center">
        <p className="font-display text-xs tracking-[0.28em] text-dust uppercase">
          {loc.region} · {clockLabel(world.hours)}
        </p>
        <h2 className="mt-2 font-display text-5xl font-semibold">{loc.name}</h2>
        <p className="mt-3 max-w-xl text-sm leading-7 text-ash/85">{loc.blurb}</p>
        {st?.status === "beacon" && (
          <p className="text-blood mt-2 text-sm">{st.hoursLeft} hours before this pin goes black.</p>
        )}
        {st?.status === "dead" && (
          <p className="mt-2 text-sm text-dust">The beacon died. This is a nest now.</p>
        )}
        {st?.status === "cleared" && (
          <p className="mt-2 text-sm text-dust">The hunt here is finished. They still flinch when you pass.</p>
        )}
        {world.raki && loc.id === "doga" && (
          <p className="mt-3 text-sm text-steel">Raki waits by the well. Trans Drop is yours while he lives.</p>
        )}

        <div className="mt-6 flex flex-wrap gap-2">
          {huntReady && enc && (
            <button
              type="button"
              onClick={() => startEncounter(enc.id)}
              className="rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink hover:bg-ash"
            >
              Begin hunt
            </button>
          )}
          <button
            type="button"
            onClick={restTown}
            className="rounded-md border border-line bg-surface/80 px-5 py-3 text-sm hover:border-steel/40"
          >
            Rest until morning
          </button>
          <button
            type="button"
            onClick={() => setMode("world")}
            className="rounded-md px-5 py-3 text-sm text-dust hover:text-ash"
          >
            Back to the road
          </button>
        </div>

        <div className="mt-8 flex gap-2">
          {world.party.map((id) => {
            const w = WARRIORS[id];
            if (!w) return null;
            return (
              <div key={id} className="w-20">
                <img src={w.portrait} alt="" className="aspect-2/3 w-full rounded-md object-cover" />
                <p className="mt-1 text-center text-[11px] text-dust">{w.name}</p>
              </div>
            );
          })}
          {world.raki && (
            <div className="w-20">
              <img src={asset("/art/raki.jpg")} alt="" className="aspect-2/3 w-full rounded-md object-cover" />
              <p className="mt-1 text-center text-[11px] text-dust">Raki</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
