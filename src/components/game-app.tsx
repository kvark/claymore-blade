import { useEffect, useState } from "react";
import { SignedIn, SignedOut, UserButton } from "@/lib/auth/gates";
import { useCurrentUserState } from "@/lib/auth/use-current-user";
import { HexCanvas } from "@/game/render/hex-canvas";
import { WorldCanvas } from "@/game/render/world-canvas";
import { hasSave, useGame } from "@/game/store";
import { CombatHud } from "./combat-hud";
import { CodexView } from "./codex-view";
import { ResultView } from "./result-view";
import { TitleScreen } from "./title-screen";
import { TownView } from "./town-view";
import { WorldHud } from "./world-hud";
import { asset } from "@/lib/asset";

export function GameApp() {
  const mode = useGame((s) => s.mode);
  const boot = useGame((s) => s.boot);
  const { isPending } = useCurrentUserState();
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    boot();
    setSaved(hasSave());
  }, [boot]);

  useEffect(() => {
    if (mode === "title") setSaved(hasSave());
  }, [mode]);

  return (
    <div className="relative h-dvh min-h-dvh overflow-hidden bg-ink text-ash">
      {mode === "title" && <TitleScreen hasSave={saved} />}
      {mode === "intro" && <IntroCrawl />}
      {mode === "world" && (
        <>
          <WorldCanvas />
          <WorldHud />
        </>
      )}
      {mode === "town" && <TownView />}
      {mode === "combat" && (
        <>
          <HexCanvas />
          <CombatHud />
        </>
      )}
      {mode === "codex" && <CodexView />}
      {mode === "result" && <ResultView />}

      <div className="pointer-events-auto absolute top-3 right-3 z-30 flex items-center gap-2">
        {isPending ? (
          <div className="h-8 w-8 animate-pulse rounded-full bg-ash/10" />
        ) : (
          <>
            <SignedOut>
              <a
                href="/login"
                className="rounded-md border border-line bg-fog px-3 py-1.5 text-xs tracking-wide text-dust hover:text-ash"
              >
                Sign in
              </a>
            </SignedOut>
            <SignedIn>
              <UserButton />
            </SignedIn>
          </>
        )}
      </div>
    </div>
  );
}

function IntroCrawl() {
  return (
    <button
      type="button"
      className="absolute inset-0 flex flex-col items-center justify-center bg-ink px-6 text-left"
      onClick={() => useGame.getState().setMode("world")}
    >
      <img src={asset("/art/title.jpg")} alt="" className="absolute inset-0 h-full w-full object-cover opacity-25" />
      <div className="absolute inset-0 bg-linear-to-t from-ink via-ink/80 to-ink/50" />
      <div className="relative max-w-lg">
        <p className="font-display text-xs tracking-[0.32em] text-dust uppercase">Sutafu · branding hall</p>
        <h2 className="mt-3 font-display text-4xl font-semibold">You are Clare, No. 47.</h2>
        <p className="mt-4 text-sm leading-7 text-ash/85">
          The Organization put silver in your eyes and a sword too large for a human shoulder.
          Doga has lit a beacon. If you walk slowly, the well will not be a well when you arrive.
        </p>
        <p className="mt-4 text-sm leading-7 text-dust">
          Raise the bar to use what they put in you. Lower it if you want to remain a person.
          The island does not care which you choose.
        </p>
        <p className="mt-8 text-xs tracking-[0.22em] text-steel uppercase">Tap to walk</p>
      </div>
    </button>
  );
}
