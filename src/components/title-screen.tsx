import { useGame } from "@/game/store";
import { asset } from "@/lib/asset";

export function TitleScreen({ hasSave }: { hasSave: boolean }) {
  const newHunt = useGame((s) => s.newHunt);
  const continueHunt = useGame((s) => s.continueHunt);
  const setMode = useGame((s) => s.setMode);

  return (
    <div className="absolute inset-0">
      <img src={asset("/art/title.jpg")} alt="" className="absolute inset-0 h-full w-full object-cover" />
      <div className="absolute inset-0 bg-linear-to-r from-ink via-ink/70 to-ink/20" />
      <div className="absolute inset-0 bg-linear-to-t from-ink via-transparent to-ink/30" />
      <div className="relative flex min-h-dvh flex-col justify-end px-6 pt-16 pb-10 sm:justify-center sm:px-12">
        <p className="font-display text-xs tracking-[0.38em] text-dust uppercase">
          The Organization still numbers the dead
        </p>
        <h1 className="mt-3 font-display text-[clamp(3.4rem,12vw,7.2rem)] leading-[0.85] font-semibold tracking-[-0.03em]">
          CLAYMORE
        </h1>
        <p className="mt-5 max-w-md text-sm leading-7 text-ash/80 sm:text-base">
          Silver-eyed warriors. A Fallout map. Hex hunts. Multi-cell demons you carve apart.
          The bar in your chest is not a metaphor.
        </p>
        <div className="mt-8 flex max-w-sm flex-col gap-2">
          <button
            type="button"
            onClick={newHunt}
            className="rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink transition hover:bg-ash"
          >
            New hunt
          </button>
          {hasSave && (
            <button
              type="button"
              onClick={continueHunt}
              className="rounded-md border border-line bg-surface/70 px-5 py-3 text-sm font-medium text-ash hover:border-steel/40"
            >
              Continue
            </button>
          )}
          <button
            type="button"
            onClick={() => setMode("codex")}
            className="rounded-md px-5 py-3 text-sm text-dust hover:text-ash"
          >
            Design codex
          </button>
        </div>
        <p className="mt-8 max-w-sm text-[11px] leading-5 text-dust">
          WASD walks the island. Click a pin to travel. In a hunt: drag the isometric board,
          click a tile to step, pick a technique, click a cell to cut. Raise the bar when you must.
        </p>
      </div>
    </div>
  );
}
