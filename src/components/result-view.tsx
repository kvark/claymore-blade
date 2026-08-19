import { useGame } from "@/game/store";
import { asset } from "@/lib/asset";

export function ResultView() {
  const result = useGame((s) => s.result);
  const dismiss = useGame((s) => s.dismissResult);
  if (!result) return null;
  return (
    <div className="absolute inset-0">
      <img
        src={result.win ? asset("/art/title.jpg") : asset("/art/ophelia.jpg")}
        alt=""
        className="absolute inset-0 h-full w-full object-cover opacity-40"
      />
      <div className="absolute inset-0 bg-ink/75" />
      <div className="relative mx-auto flex min-h-dvh max-w-lg flex-col justify-center px-6">
        <p className="font-display text-xs tracking-[0.28em] text-dust uppercase">
          {result.win ? "The board is empty" : "Hunt failed"}
        </p>
        <h2 className="mt-2 font-display text-4xl font-semibold">{result.title}</h2>
        <p className="mt-4 text-sm leading-7 text-ash/85">{result.body}</p>
        <button
          type="button"
          onClick={dismiss}
          className="mt-8 w-fit rounded-md bg-steel px-5 py-3 text-sm font-medium text-ink hover:bg-ash"
        >
          {result.win ? "Return to the island" : "The Organization sends another"}
        </button>
      </div>
    </div>
  );
}
