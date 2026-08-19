import { LOCATIONS, WARRIORS } from "@/game/data/catalog";
import { clockLabel } from "@/game/sim/world";
import { useGame } from "@/game/store";

export function WorldHud() {
  const world = useGame((s) => s.world);
  const travelTo = useGame((s) => s.travelTo);
  const setMode = useGame((s) => s.setMode);
  const beacons = LOCATIONS.filter((l) => world.locations[l.id]?.status === "beacon");

  return (
    <>
      <header className="pointer-events-none absolute top-0 right-0 left-0 z-10 flex items-start justify-between p-4 pr-28">
        <div className="pointer-events-auto rounded-lg border border-line bg-fog px-4 py-3">
          <p className="font-display text-xs tracking-[0.24em] text-dust uppercase">Island</p>
          <p className="font-display text-2xl leading-none">{clockLabel(world.hours)}</p>
          <p className="mt-1 text-xs text-dust">
            Rank {world.rank} · Karma {world.karma >= 0 ? "+" : ""}
            {world.karma}
          </p>
        </div>
        <div className="pointer-events-auto hidden gap-2 sm:flex">
          {world.party.map((id) => {
            const w = WARRIORS[id];
            if (!w) return null;
            return (
              <div key={id} className="overflow-hidden rounded-md border border-line">
                <img src={w.portrait} alt={w.name} className="h-14 w-10 object-cover" />
              </div>
            );
          })}
        </div>
      </header>

      <aside className="pointer-events-auto absolute bottom-4 left-4 z-10 w-[min(100%-2rem,22rem)] rounded-lg border border-line bg-fog p-4">
        <p className="font-display text-xs tracking-[0.22em] text-dust uppercase">Beacons</p>
        <ul className="mt-2 space-y-2">
          {beacons.length === 0 && (
            <li className="text-sm text-dust">No village is screaming. Walk. The Maw is still dark.</li>
          )}
          {beacons.map((b) => {
            const st = world.locations[b.id]!;
            return (
              <li key={b.id}>
                <button
                  type="button"
                  onClick={() => travelTo(b.id)}
                  className="flex w-full items-center justify-between rounded-md px-1 py-1 text-left hover:bg-ash/5"
                >
                  <span>
                    <span className="block text-sm font-medium">{b.name}</span>
                    <span className="text-xs text-dust">{b.region}</span>
                  </span>
                  <span className="font-display text-blood text-sm tabular-nums">{st.hoursLeft}h</span>
                </button>
              </li>
            );
          })}
        </ul>
        <div className="mt-3 flex gap-2">
          <button
            type="button"
            onClick={() => setMode("codex")}
            className="rounded-md border border-line px-3 py-1.5 text-xs text-dust hover:text-ash"
          >
            Codex
          </button>
          <button
            type="button"
            onClick={() => setMode("title")}
            className="rounded-md border border-line px-3 py-1.5 text-xs text-dust hover:text-ash"
          >
            Title
          </button>
        </div>
      </aside>
    </>
  );
}
