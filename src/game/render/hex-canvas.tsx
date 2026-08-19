import { useEffect, useRef } from "react";
import {
  coreHex,
  legalMoves,
  legalTargets,
  liveCells,
  skillOf,
  zoneFor,
} from "@/game/sim/combat";
import {
  axialToPixel,
  hexCorners,
  hexEq,
  hexKey,
  pixelToAxial,
  type Axial,
} from "@/game/sim/hex";
import { useGame } from "@/game/store";

const images = new Map<string, HTMLImageElement>();
function img(src: string) {
  let im = images.get(src);
  if (!im) {
    im = new Image();
    im.crossOrigin = "anonymous";
    im.src = src;
    images.set(src, im);
  }
  return im;
}

export function HexCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);
  const combat = useGame((s) => s.combat);
  const ui = useGame((s) => s.ui);
  const combatAct = useGame((s) => s.combatAct);
  const setHover = useGame((s) => s.setHover);
  const setSkill = useGame((s) => s.setSkill);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let raf = 0;
    const bg = img("/art/battle-hamlet.jpg");
    const draw = () => {
      const st = useGame.getState();
      const battle = st.combat;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight;
      const dpr = Math.min(2, window.devicePixelRatio || 1);
      if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) {
        canvas.width = Math.floor(w * dpr);
        canvas.height = Math.floor(h * dpr);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, w, h);
      if (!battle) return;
      if (bg.complete) {
        const scale = Math.max(w / bg.naturalWidth, h / bg.naturalHeight);
        const dw = bg.naturalWidth * scale;
        const dh = bg.naturalHeight * scale;
        ctx.drawImage(bg, (w - dw) / 2, (h - dh) / 2, dw, dh);
      } else {
        ctx.fillStyle = "#161311";
        ctx.fillRect(0, 0, w, h);
      }
      ctx.fillStyle = "rgba(11,10,9,0.38)";
      ctx.fillRect(0, 0, w, h);

      const size = Math.min(w / (battle.cols * 1.85), h / (battle.rows * 1.75));
      const gridW = Math.sqrt(3) * size * (battle.cols + 0.5);
      const gridH = (3 / 2) * size * battle.rows + size;
      const ox = (w - gridW) / 2 + size;
      const oy = (h - gridH) / 2 + size * 0.8;

      const toPix = (hex: Axial) => {
        const p = axialToPixel(hex, size);
        return { x: ox + p.x, y: oy + p.y };
      };

      const actor = battle.units.find((u) => u.id === battle.order[battle.turn] && !u.dead);
      const moves = actor && actor.side === "player" ? legalMoves(battle, actor.id) : [];
      const skill = actor && st.ui.selectedSkill ? skillOf(st.ui.selectedSkill) : undefined;
      const targets =
        actor && skill && actor.side === "player" ? legalTargets(battle, actor.id, skill.id) : [];
      let preview: Axial[] = [];
      if (actor && skill && st.ui.hoverHex) {
        preview = zoneFor(battle, actor, skill, st.ui.hoverHex);
      }

      for (let q = 0; q < battle.cols; q++) {
        for (let r = 0; r < battle.rows; r++) {
          const hex = { q, r };
          const { x, y } = toPix(hex);
          const terrain = battle.terrain[hexKey(hex)] ?? "grass";
          const pts = hexCorners(x, y, size - 1.2);
          ctx.beginPath();
          pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
          ctx.closePath();
          ctx.fillStyle =
            terrain === "water"
              ? "rgba(40,55,62,0.45)"
              : terrain === "mud"
                ? "rgba(70,52,36,0.42)"
                : terrain === "ruin"
                  ? "rgba(70,68,64,0.4)"
                  : "rgba(20,22,18,0.28)";
          ctx.fill();
          ctx.strokeStyle = "rgba(235,228,214,0.16)";
          ctx.lineWidth = 1;
          ctx.stroke();
        }
      }

      const paintSet = (cells: Axial[], fill: string, stroke?: string) => {
        for (const hex of cells) {
          const { x, y } = toPix(hex);
          const pts = hexCorners(x, y, size - 1);
          ctx.beginPath();
          pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
          ctx.closePath();
          ctx.fillStyle = fill;
          ctx.fill();
          if (stroke) {
            ctx.strokeStyle = stroke;
            ctx.lineWidth = 2;
            ctx.stroke();
          }
        }
      };
      paintSet(moves, "rgba(200,204,212,0.16)", "rgba(200,204,212,0.45)");
      paintSet(targets, "rgba(154,36,48,0.10)");
      paintSet(preview, "rgba(154,36,48,0.38)", "rgba(235,228,214,0.7)");

      for (const z of battle.zones) {
        const ring = [];
        // draw current ripple radius via preview-like paint in render
        for (let q = 0; q < battle.cols; q++) {
          for (let r = 0; r < battle.rows; r++) {
            const dq = q - z.center.q;
            const dr = r - z.center.r;
            const dist =
              (Math.abs(dq) + Math.abs(dq + dr) + Math.abs(dr)) / 2;
            if (dist === z.radius) ring.push({ q, r });
          }
        }
        paintSet(ring, "rgba(154,36,48,0.22)", "rgba(154,36,48,0.8)");
      }

      const sorted = [...battle.units].filter((u) => !u.dead).sort((a, b) => {
        const ac = coreHex(a);
        const bc = coreHex(b);
        return ac.r - bc.r || ac.q - bc.q;
      });
      for (const u of sorted) {
        const cells = liveCells(u);
        for (const c of cells) {
          const { x, y } = toPix(c);
          const pts = hexCorners(x, y, size - 2);
          ctx.beginPath();
          pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
          ctx.closePath();
          ctx.fillStyle =
            u.side === "player" ? "rgba(200,204,212,0.22)" : "rgba(154,36,48,0.22)";
          ctx.fill();
          ctx.strokeStyle = u.id === actor?.id ? "#ebe4d6" : u.color;
          ctx.lineWidth = u.id === actor?.id ? 2.4 : 1.4;
          ctx.stroke();
        }
        const c = coreHex(u);
        const { x, y } = toPix(c);
        const spr = img(u.sprite ?? u.portrait);
        const multi = cells.length > 1;
        const ih = size * (multi ? 2.8 : 2.15);
        const iw = ih * 0.7;
        if (spr.complete && spr.naturalWidth) {
          ctx.save();
          ctx.shadowColor = "rgba(0,0,0,0.55)";
          ctx.shadowBlur = 16;
          ctx.drawImage(spr, x - iw / 2, y - ih * 0.78, iw, ih);
          ctx.restore();
        } else {
          ctx.beginPath();
          ctx.arc(x, y, size * 0.45, 0, Math.PI * 2);
          ctx.fillStyle = u.color;
          ctx.fill();
        }
        ctx.font = "600 11px Figtree, sans-serif";
        ctx.textAlign = "center";
        ctx.fillStyle = "#ebe4d6";
        ctx.fillText(u.name, x, y + size * 0.85);
        const ratio = u.hp / Math.max(1, u.maxHp);
        ctx.fillStyle = "rgba(11,10,9,0.7)";
        ctx.fillRect(x - 18, y + size * 0.92, 36, 4);
        ctx.fillStyle = ratio < 0.35 ? "#9a2430" : "#c8ccd4";
        ctx.fillRect(x - 18, y + size * 0.92, 36 * ratio, 4);
        if (u.nextHint) {
          ctx.fillStyle = "#c8ccd4";
          ctx.font = "500 10px Figtree, sans-serif";
          ctx.fillText(u.nextHint, x, y - size * 1.05);
        }
      }
      raf = requestAnimationFrame(draw);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, []);

  function hexAt(e: React.PointerEvent<HTMLCanvasElement>): Axial | undefined {
    const canvas = ref.current;
    const battle = useGame.getState().combat;
    if (!canvas || !battle) return;
    const rect = canvas.getBoundingClientRect();
    const w = rect.width;
    const h = rect.height;
    const size = Math.min(w / (battle.cols * 1.85), h / (battle.rows * 1.75));
    const gridW = Math.sqrt(3) * size * (battle.cols + 0.5);
    const gridH = (3 / 2) * size * battle.rows + size;
    const ox = (w - gridW) / 2 + size;
    const oy = (h - gridH) / 2 + size * 0.8;
    const x = e.clientX - rect.left - ox;
    const y = e.clientY - rect.top - oy;
    const hex = pixelToAxial(x, y, size);
    if (hex.q < 0 || hex.r < 0 || hex.q >= battle.cols || hex.r >= battle.rows) return;
    return hex;
  }

  return (
    <canvas
      ref={ref}
      className="absolute inset-0 h-full w-full touch-none"
      onPointerMove={(e) => setHover(hexAt(e))}
      onPointerLeave={() => setHover(undefined)}
      onPointerDown={(e) => {
        const hex = hexAt(e);
        const st = useGame.getState();
        const battle = st.combat;
        if (!hex || !battle || battle.over) return;
        const actor = battle.units.find((u) => u.id === battle.order[battle.turn] && !u.dead);
        if (!actor || actor.side !== "player") return;
        const skill = st.ui.selectedSkill ? skillOf(st.ui.selectedSkill) : undefined;
        if (skill) {
          combatAct({ type: "skill", skillId: skill.id, hex });
          setSkill(undefined);
          return;
        }
        const moves = legalMoves(battle, actor.id);
        if (moves.some((m) => hexEq(m, hex))) combatAct({ type: "move", hex });
      }}
    />
  );
}
