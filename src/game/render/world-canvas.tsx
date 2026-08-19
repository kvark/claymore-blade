import { useEffect, useRef } from "react";
import { LOCATIONS, WARRIORS } from "@/game/data/catalog";
import { nearestLocation } from "@/game/sim/world";
import { useGame } from "@/game/store";

export function WorldCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);
  const world = useGame((s) => s.world);
  const travelTo = useGame((s) => s.travelTo);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const map = new Image();
    map.src = "/art/world-map.jpg";
    let raf = 0;
    let last = performance.now();
    const loop = (now: number) => {
      const dt = Math.min(0.05, (now - last) / 1000);
      last = now;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
        canvas.width = Math.floor(w * dpr);
        canvas.height = Math.floor(h * dpr);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, w, h);
      ctx.fillStyle = "#0b0a09";
      ctx.fillRect(0, 0, w, h);

      let dx = 0;
      let dy = 0;
      const held = useGame.getState().keys;
      if (held.has("KeyA") || held.has("ArrowLeft")) dx -= 1;
      if (held.has("KeyD") || held.has("ArrowRight")) dx += 1;
      if (held.has("KeyW") || held.has("ArrowUp")) dy -= 1;
      if (held.has("KeyS") || held.has("ArrowDown")) dy += 1;
      if (dx || dy) {
        const len = Math.hypot(dx, dy) || 1;
        useGame.getState().moveParty(dx / len, dy / len, dt);
      }

      const st = useGame.getState().world;
      if (map.complete && map.naturalWidth) {
        const iw = map.naturalWidth;
        const ih = map.naturalHeight;
        const sx = iw * 0.07;
        const sy = ih * 0.03;
        const sw = iw * 0.86;
        const sh = ih * 0.84;
        const scale = Math.max(w / sw, h / sh);
        const dw = sw * scale;
        const dh = sh * scale;
        const ox = (w - dw) / 2;
        const oy = (h - dh) / 2;
        ctx.drawImage(map, sx, sy, sw, sh, ox, oy, dw, dh);
        ctx.fillStyle = "rgba(11,10,9,0.12)";
        ctx.fillRect(0, 0, w, h);

        const toScreen = (lx: number, ly: number) => ({
          x: ox + ((lx * iw - sx) / sw) * dw,
          y: oy + ((ly * ih - sy) / sh) * dh,
        });

        for (const loc of LOCATIONS) {
          const ls = st.locations[loc.id];
          const { x, y } = toScreen(loc.x, loc.y);
          const status = ls?.status ?? "quiet";
          if (status === "locked") continue;
          if (status === "beacon") {
            const pulse = 10 + Math.sin(now / 280) * 4;
            ctx.beginPath();
            ctx.arc(x, y, pulse + 10, 0, Math.PI * 2);
            ctx.fillStyle = "rgba(154,36,48,0.18)";
            ctx.fill();
          }
          ctx.beginPath();
          ctx.arc(x, y, status === "beacon" ? 7 : 5, 0, Math.PI * 2);
          ctx.fillStyle =
            status === "beacon"
              ? "#9a2430"
              : status === "dead"
                ? "#3a322c"
                : status === "cleared"
                  ? "#c8ccd4"
                  : "#8d8578";
          ctx.fill();
          ctx.strokeStyle = "rgba(235,228,214,0.45)";
          ctx.lineWidth = 1;
          ctx.stroke();
          ctx.font = "600 12px Figtree, sans-serif";
          ctx.fillStyle = "#ebe4d6";
          ctx.textAlign = "center";
          ctx.fillText(loc.name, x, y - 12);
        }

        const { x: px, y: py } = toScreen(st.partyX, st.partyY);
        ctx.beginPath();
        ctx.arc(px, py, 9, 0, Math.PI * 2);
        ctx.fillStyle = "#ebe4d6";
        ctx.fill();
        ctx.strokeStyle = "#0b0a09";
        ctx.lineWidth = 2;
        ctx.stroke();
        const lead = WARRIORS[st.party[0] ?? "kira"];
        ctx.font = "600 11px Figtree, sans-serif";
        ctx.fillStyle = "#ebe4d6";
        ctx.fillText(lead?.name ?? "Kira", px, py + 22);
      }
      raf = requestAnimationFrame(loop);
    };
    raf = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(raf);
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent, down: boolean) => {
      if (
        ["KeyW", "KeyA", "KeyS", "KeyD", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].includes(
          e.code,
        )
      ) {
        e.preventDefault();
        useGame.getState().holdKey(e.code, down);
      }
    };
    const down = (e: KeyboardEvent) => onKey(e, true);
    const up = (e: KeyboardEvent) => onKey(e, false);
    const clear = () => {
      ["KeyW", "KeyA", "KeyS", "KeyD", "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"].forEach(
        (c) => useGame.getState().holdKey(c, false),
      );
    };
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", clear);
    return () => {
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", clear);
    };
  }, []);

  useEffect(() => {
    window.__controlsTest = {
      getYaw: () => 0,
      getSpeed: () => (useGame.getState().keys.size ? 1 : 0),
      getX: () => useGame.getState().world.partyX,
      getY: () => useGame.getState().world.partyY,
      setKeys: (codes: string[]) => {
        const cur = useGame.getState().keys;
        cur.forEach((c) => useGame.getState().holdKey(c, false));
        codes.forEach((c) => useGame.getState().holdKey(c, true));
      },
    };
    return () => {
      delete window.__controlsTest;
    };
  }, []);

  function onClick(e: React.MouseEvent<HTMLCanvasElement>) {
    const canvas = ref.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;
    const iw = 1792;
    const ih = 1008;
    const sx = iw * 0.07;
    const sy = ih * 0.03;
    const sw = iw * 0.86;
    const sh = ih * 0.84;
    const scale = Math.max(w / sw, h / sh);
    const dw = sw * scale;
    const dh = sh * scale;
    const ox = (w - dw) / 2;
    const oy = (h - dh) / 2;
    const nx = ((e.clientX - rect.left - ox) / dw) * (sw / iw) + sx / iw;
    const ny = ((e.clientY - rect.top - oy) / dh) * (sh / ih) + sy / ih;
    const hit = nearestLocation(nx, ny, 0.05);
    if (hit && world.locations[hit.id]?.status !== "locked") travelTo(hit.id);
  }

  return (
    <canvas
      ref={ref}
      className="absolute inset-0 h-full w-full touch-none"
      onClick={onClick}
    />
  );
}

declare global {
  interface Window {
    __controlsTest?: {
      getYaw: () => number;
      getSpeed: () => number;
      getX?: () => number;
      getY?: () => number;
      setKeys?: (codes: string[]) => void;
    };
  }
}
