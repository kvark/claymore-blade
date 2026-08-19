import { createFileRoute, Link } from "@tanstack/react-router";
import { GROK_PROVIDERS, authEnabled, signIn } from "@/lib/auth/client";
import { asset } from "@/lib/asset";

export const Route = createFileRoute("/login")({ component: Login });

function Login() {
  return (
    <main className="relative grid min-h-dvh place-items-center overflow-hidden bg-ink px-5 text-ash">
      <img
        src={asset("/art/title.jpg")}
        alt=""
        className="pointer-events-none absolute inset-0 h-full w-full object-cover opacity-35"
      />
      <div className="absolute inset-0 bg-linear-to-t from-ink via-ink/70 to-ink/40" />
      <div className="relative w-full max-w-sm rounded-xl border border-line bg-surface/90 p-6 shadow-[0_24px_80px_rgb(0_0_0/0.45)]">
        <p className="font-display text-xs tracking-[0.28em] text-dust uppercase">The Organization</p>
        <h1 className="mt-2 font-display text-4xl font-semibold tracking-tight">Sign in</h1>
        <p className="mt-2 text-sm leading-relaxed text-dust">
          Guest hunts save on this device. A signed-in hunter keeps the ledger with them.
        </p>
        <div className="mt-6 space-y-2">
          {authEnabled ? (
            GROK_PROVIDERS.map((p) => (
              <button
                key={p.providerId}
                type="button"
                onClick={() => signIn(p.providerId, { callbackURL: "/" })}
                className="w-full rounded-md border border-line bg-raised px-4 py-3 text-sm font-medium text-ash transition hover:border-steel/40 hover:bg-steel/10"
              >
                Continue with {p.label}
              </button>
            ))
          ) : (
            <p className="text-sm text-dust">Sign-in is disabled.</p>
          )}
        </div>
        <Link
          to="/"
          className="mt-5 block text-center text-sm text-dust underline-offset-4 hover:text-ash hover:underline"
        >
          Hunt as a guest
        </Link>
      </div>
    </main>
  );
}
