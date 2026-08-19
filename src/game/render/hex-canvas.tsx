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
  axialToWorld,
  hexCornersXZ,
  hexEq,
  hexKey,
  isoToWorld,
  terrainHeight,
  worldToAxial,
  worldToIso,
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

type Cam = { panX: number; panY: number; zoom: number };

function boardSize(cols: number, rows: number, w: number, h: number) {
  const fit = Math.min(w / (cols * 1.55 + 1.2), h / (rows * 0.78 + 2.1));
  return Math.max(22, Math.min(58, fit));
}

function cameraOrigin(cols: number, rows: number, size: number, w: number, h: number, cam: Cam) {
  const mid = axialToWorld({ q: (cols - 1) / 2, r: (rows - 1) / 2 }, size);
  const c = worldToIso(mid.x, 0, mid.z);
  return {
    ox: w / 2 - c.x * cam.zoom + cam.panX,
    oy: h * 0.46 - c.y * cam.zoom + cam.panY,
    size,
  };
}

function hexAtPoint(
  clientX: number,
  clientY: number,
  rect: DOMRect,
  cols: number,
  rows: number,
  cam: Cam,
): Axial | undefined {
  const w = rect.width;
  const h = rect.height;
  const size = boardSize(cols, rows, w, h);
  const { ox, oy } = cameraOrigin(cols, rows, size, w, h, cam);
  const sx = (clientX - rect.left - ox) / cam.zoom;
  const sy = (clientY - rect.top - oy) / cam.zoom;
  const world = isoToWorld(sx, sy, size * 0.14);
  const hex = worldToAxial(world.x, world.z, size);
  if (hex.q < 0 || hex.r < 0 || hex.q >= cols || hex.r >= rows) return;
  return hex;
}

function fillPoly(ctx: CanvasRenderingContext2D, pts: { x: number; y: number }[], fill: string, stroke?: string, width = 1) {
  if (pts.length < 2) return;
  ctx.beginPath();
  pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
  ctx.closePath();
  ctx.fillStyle = fill;
  ctx.fill();
  if (stroke) {
    ctx.strokeStyle = stroke;
    ctx.lineWidth = width;
    ctx.stroke();
  }
}

function terrainColors(kind: string) {
  if (kind === "water") {
    return { top: "rgba(46,62,70,0.92)", se: "rgba(28,40,46,0.95)", sw: "rgba(18,28,32,0.96)", edge: "rgba(120,150,160,0.28)" };
  }
  if (kind === "mud") {
    return { top: "rgba(78,58,40,0.94)", se: "rgba(54,38,26,0.96)", sw: "rgba(36,24,16,0.97)", edge: "rgba(160,130,90,0.22)" };
  }
  if (kind === "ruin") {
    return { top: "rgba(78,76,72,0.94)", se: "rgba(52,50,48,0.96)", sw: "rgba(32,30,28,0.97)", edge: "rgba(210,205,195,0.28)" };
  }
  return { top: "rgba(48,52,42,0.9)", se: "rgba(32,34,28,0.94)", sw: "rgba(20,22,18,0.96)", edge: "rgba(200,196,180,0.2)" };
}

function drawPrism(
  ctx: CanvasRenderingContext2D,
  hex: Axial,
  size: number,
  height: number,
  colors: { top: string; se: string; sw: string; edge: string },
  overlay?: string,
  rim?: string,
) {
  const w = axialToWorld(hex, size);
  const corners = hexCornersXZ(w.x, w.z, size * 0.94);
  const top = corners.map((c) => worldToIso(c.x, height, c.z));
  const bot = corners.map((c) => worldToIso(c.x, Math.min(0, height), c.z));
  const sides: { i: number; depth: number; pts: { x: number; y: number }[] }[] = [];
  for (let i = 0; i < 6; i++) {
    const j = (i + 1) % 6;
    const midX = (corners[i].x + corners[j].x) / 2 - w.x;
    const midZ = (corners[i].z + corners[j].z) / 2 - w.z;
    if (midX + midZ <= 0) continue;
    sides.push({
      i,
      depth: (top[i].y + top[j].y) / 2,
      pts: [bot[i], bot[j], top[j], top[i]],
    });
  }
  sides.sort((a, b) => a.depth - b.depth);
  for (const s of sides) {
    const se = s.i === 0 || s.i === 1 || s.i === 5;
    fillPoly(ctx, s.pts, se ? colors.se : colors.sw);
  }
  fillPoly(ctx, top, overlay ?? colors.top, rim ?? colors.edge, rim ? 1.6 : 1);
}

export function HexCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);
  const cam = useRef<Cam>({ panX: 0, panY: 8, zoom: 1.05 });
  const drag = useRef<{ id: number; x: number; y: number; moved: boolean } | null>(null);
  const combatAct = useGame((s) => s.combatAct);
  const setHover = useGame((s) => s.setHover);
  const setSkill = useGame((s) => s.setSkill);

  useEffect(() => {
    const canvas = ref.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    let raf = 0;
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

      const bg = img(battle.art || "/art/battle-doga.jpg");
      if (bg.complete && bg.naturalWidth) {
        const scale = Math.max(w / bg.naturalWidth, h / bg.naturalHeight);
        const dw = bg.naturalWidth * scale;
        const dh = bg.naturalHeight * scale;
        ctx.filter = "saturate(0.55) brightness(0.45)";
        ctx.drawImage(bg, (w - dw) / 2, (h - dh) / 2, dw, dh);
        ctx.filter = "none";
      } else {
        ctx.fillStyle = "#161311";
        ctx.fillRect(0, 0, w, h);
      }
      const sky = ctx.createLinearGradient(0, 0, 0, h);
      sky.addColorStop(0, "rgba(11,10,9,0.15)");
      sky.addColorStop(0.45, "rgba(11,10,9,0.42)");
      sky.addColorStop(1, "rgba(11,10,9,0.78)");
      ctx.fillStyle = sky;
      ctx.fillRect(0, 0, w, h);

      const size = boardSize(battle.cols, battle.rows, w, h);
      const { ox, oy } = cameraOrigin(battle.cols, battle.rows, size, w, h, cam.current);
      ctx.save();
      ctx.setTransform(dpr * cam.current.zoom, 0, 0, dpr * cam.current.zoom, dpr * ox, dpr * oy);

      const actor = battle.units.find((u) => u.id === battle.order[battle.turn] && !u.dead);
      const moves = actor && actor.side === "player" ? legalMoves(battle, actor.id) : [];
      const skill = actor && st.ui.selectedSkill ? skillOf(st.ui.selectedSkill) : undefined;
      const targets =
        actor && skill && actor.side === "player" ? legalTargets(battle, actor.id, skill.id) : [];
      const preview =
        actor && skill && st.ui.hoverHex ? zoneFor(battle, actor, skill, st.ui.hoverHex) : [];
      const moveKeys = new Set(moves.map(hexKey));
      const targetKeys = new Set(targets.map(hexKey));
      const previewKeys = new Set(preview.map(hexKey));
      const hoverKey = st.ui.hoverHex ? hexKey(st.ui.hoverHex) : "";

      const occupy = new Map<string, (typeof battle.units)[number]>();
      for (const u of battle.units) {
        if (u.dead) continue;
        for (const c of liveCells(u)) occupy.set(hexKey(c), u);
      }

      const tiles: Axial[] = [];
      for (let q = 0; q < battle.cols; q++) {
        for (let r = 0; r < battle.rows; r++) tiles.push({ q, r });
      }
      tiles.sort((a, b) => {
        const aw = axialToWorld(a, size);
        const bw = axialToWorld(b, size);
        return aw.x + aw.z - (bw.x + bw.z) || a.q - b.q;
      });

      const cores = new Map<string, (typeof battle.units)[number]>();
      for (const u of battle.units) {
        if (!u.dead) cores.set(hexKey(coreHex(u)), u);
      }

      for (const hex of tiles) {
        const key = hexKey(hex);
        const terrain = battle.terrain[key] ?? "grass";
        const colors = terrainColors(terrain);
        let height = terrainHeight(terrain, size);
        const occupant = occupy.get(key);
        if (occupant) height += size * (occupant.parts.length > 1 ? 0.12 : 0.04);
        let overlay: string | undefined;
        let rim: string | undefined;
        if (previewKeys.has(key)) {
          overlay = "rgba(154,36,48,0.55)";
          rim = "rgba(235,228,214,0.85)";
        } else if (targetKeys.has(key)) {
          overlay = "rgba(154,36,48,0.28)";
          rim = "rgba(154,36,48,0.7)";
        } else if (moveKeys.has(key)) {
          overlay = "rgba(200,204,212,0.28)";
          rim = "rgba(200,204,212,0.7)";
        } else if (occupant) {
          overlay =
            occupant.side === "player" ? "rgba(200,204,212,0.22)" : "rgba(154,36,48,0.24)";
          rim = occupant.id === actor?.id ? "#ebe4d6" : occupant.color;
        }
        if (hoverKey === key && !rim) rim = "rgba(235,228,214,0.55)";
        for (const z of battle.zones) {
          const dq = hex.q - z.center.q;
          const dr = hex.r - z.center.r;
          const dist = (Math.abs(dq) + Math.abs(dq + dr) + Math.abs(dr)) / 2;
          if (dist === z.radius) {
            overlay = "rgba(154,36,48,0.4)";
            rim = "rgba(154,36,48,0.95)";
          }
        }
        drawPrism(ctx, hex, size, height, colors, overlay, rim);

        const unit = cores.get(key);
        if (!unit) continue;
        const world = axialToWorld(hex, size);
        const foot = worldToIso(world.x, height, world.z);
        const spr = img(unit.sprite ?? unit.portrait);
        const multi = liveCells(unit).length > 1;
        const ih = size * (multi ? 3.1 : 2.45);
        const iw = ih * 0.68;
        if (spr.complete && spr.naturalWidth) {
          ctx.save();
          ctx.shadowColor = "rgba(0,0,0,0.62)";
          ctx.shadowBlur = 18;
          ctx.shadowOffsetY = 6;
          ctx.drawImage(spr, foot.x - iw / 2, foot.y - ih * 0.92, iw, ih);
          ctx.restore();
        } else {
          ctx.beginPath();
          ctx.arc(foot.x, foot.y - size * 0.4, size * 0.38, 0, Math.PI * 2);
          ctx.fillStyle = unit.color;
          ctx.fill();
        }
        ctx.font = "600 11px Figtree, sans-serif";
        ctx.textAlign = "center";
        ctx.fillStyle = "#ebe4d6";
        ctx.fillText(unit.name, foot.x, foot.y + size * 0.22);
        const ratio = unit.hp / Math.max(1, unit.maxHp);
        ctx.fillStyle = "rgba(11,10,9,0.72)";
        ctx.fillRect(foot.x - 18, foot.y + size * 0.3, 36, 4);
        ctx.fillStyle = ratio < 0.35 ? "#9a2430" : "#c8ccd4";
        ctx.fillRect(foot.x - 18, foot.y + size * 0.3, 36 * ratio, 4);
        if (unit.nextHint) {
          ctx.fillStyle = "#c8ccd4";
          ctx.font = "500 10px Figtree, sans-serif";
          ctx.fillText(unit.nextHint, foot.x, foot.y - ih * 0.95);
        }
      }

      ctx.restore();
      raf = requestAnimationFrame(draw);
    };
    raf = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(raf);
  }, []);

  function hexAt(e: React.PointerEvent<HTMLCanvasElement>): Axial | undefined {
    const canvas = ref.current;
    const battle = useGame.getState().combat;
    if (!canvas || !battle) return;
    return hexAtPoint(
      e.clientX,
      e.clientY,
      canvas.getBoundingClientRect(),
      battle.cols,
      battle.rows,
      cam.current,
    );
  }

  return (
    <canvas
      ref={ref}
      className="absolute inset-0 h-full w-full touch-none"
      onWheel={(e) => {
        e.preventDefault();
        const next = cam.current.zoom * (e.deltaY < 0 ? 1.08 : 0.92);
        cam.current.zoom = Math.max(0.7, Math.min(2.1, next));
      }}
      onPointerMove={(e) => {
        if (drag.current && drag.current.id === e.pointerId) {
          cam.current.panX += e.clientX - drag.current.x;
          cam.current.panY += e.clientY - drag.current.y;
          if (Math.abs(e.clientX - drag.current.x) + Math.abs(e.clientY - drag.current.y) > 4) {
            drag.current.moved = true;
          }
          drag.current.x = e.clientX;
          drag.current.y = e.clientY;
          return;
        }
        setHover(hexAt(e));
      }}
      onPointerLeave={() => {
        drag.current = null;
        setHover(undefined);
      }}
      onPointerDown={(e) => {
        drag.current = { id: e.pointerId, x: e.clientX, y: e.clientY, moved: false };
        (e.target as HTMLCanvasElement).setPointerCapture(e.pointerId);
      }}
      onPointerUp={(e) => {
        const wasDrag = drag.current?.moved;
        drag.current = null;
        if (wasDrag) return;
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
