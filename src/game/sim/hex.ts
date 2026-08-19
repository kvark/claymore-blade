export type Axial = { q: number; r: number };

export const HEX_DIRS: Axial[] = [
  { q: 1, r: 0 },
  { q: 1, r: -1 },
  { q: 0, r: -1 },
  { q: -1, r: 0 },
  { q: -1, r: 1 },
  { q: 0, r: 1 },
];

export function hexKey(h: Axial): string {
  return `${h.q},${h.r}`;
}

export function hexEq(a: Axial, b: Axial): boolean {
  return a.q === b.q && a.r === b.r;
}

export function hexAdd(a: Axial, b: Axial): Axial {
  return { q: a.q + b.q, r: a.r + b.r };
}

export function hexSub(a: Axial, b: Axial): Axial {
  return { q: a.q - b.q, r: a.r - b.r };
}

export function hexScale(a: Axial, n: number): Axial {
  return { q: a.q * n, r: a.r * n };
}

export function rotate60(h: Axial, times: number): Axial {
  let q = h.q;
  let r = h.r;
  const t = ((times % 6) + 6) % 6;
  for (let i = 0; i < t; i++) {
    const nq = -r;
    const nr = q + r;
    q = nq;
    r = nr;
  }
  return { q, r };
}

export function cubeRound(qf: number, rf: number): Axial {
  const sf = -qf - rf;
  let q = Math.round(qf);
  let r = Math.round(rf);
  let s = Math.round(sf);
  const qd = Math.abs(q - qf);
  const rd = Math.abs(r - rf);
  const sd = Math.abs(s - sf);
  if (qd > rd && qd > sd) q = -r - s;
  else if (rd > sd) r = -q - s;
  return { q, r };
}

export function axialToPixel(h: Axial, size: number): { x: number; y: number } {
  return {
    x: size * Math.sqrt(3) * (h.q + h.r / 2),
    y: size * (3 / 2) * h.r,
  };
}

export function pixelToAxial(x: number, y: number, size: number): Axial {
  const q = ((Math.sqrt(3) / 3) * x - (1 / 3) * y) / size;
  const r = ((2 / 3) * y) / size;
  return cubeRound(q, r);
}

export function hexDistance(a: Axial, b: Axial): number {
  return (
    (Math.abs(a.q - b.q) +
      Math.abs(a.q + a.r - b.q - b.r) +
      Math.abs(a.r - b.r)) /
    2
  );
}

export function hexNeighbors(h: Axial): Axial[] {
  return HEX_DIRS.map((d) => hexAdd(h, d));
}

export function hexDisc(center: Axial, radius: number): Axial[] {
  const out: Axial[] = [];
  for (let q = -radius; q <= radius; q++) {
    const rMin = Math.max(-radius, -q - radius);
    const rMax = Math.min(radius, -q + radius);
    for (let r = rMin; r <= rMax; r++) {
      out.push({ q: center.q + q, r: center.r + r });
    }
  }
  return out;
}

export function hexRing(center: Axial, radius: number): Axial[] {
  if (radius <= 0) return [{ ...center }];
  return hexDisc(center, radius).filter((h) => hexDistance(center, h) === radius);
}

export function hexLine(a: Axial, b: Axial): Axial[] {
  const n = hexDistance(a, b);
  if (n === 0) return [{ ...a }];
  const out: Axial[] = [];
  for (let i = 0; i <= n; i++) {
    const t = i / n;
    out.push(cubeRound(a.q + (b.q - a.q) * t, a.r + (b.r - a.r) * t));
  }
  return out;
}

export function hexCone(origin: Axial, facing: number, range: number): Axial[] {
  const fwd = HEX_DIRS[((facing % 6) + 6) % 6];
  const fq = fwd.q;
  const fr = fwd.r;
  const fs = -fq - fr;
  const out: Axial[] = [];
  for (const h of hexDisc(origin, range)) {
    const d = hexDistance(origin, h);
    if (d === 0) continue;
    const dq = h.q - origin.q;
    const dr = h.r - origin.r;
    const ds = -dq - dr;
    const dot = fq * dq + fr * dr + fs * ds;
    if (dot >= d - 0.5) out.push(h);
  }
  return out;
}

export function hexSweep(origin: Axial, facing: number): Axial[] {
  const f = ((facing % 6) + 6) % 6;
  return [HEX_DIRS[(f + 5) % 6], HEX_DIRS[f], HEX_DIRS[(f + 1) % 6]].map((d) =>
    hexAdd(origin, d),
  );
}

export function facingToward(from: Axial, to: Axial): number {
  const dq = to.q - from.q;
  const dr = to.r - from.r;
  if (dq === 0 && dr === 0) return 0;
  let best = 0;
  let bestDot = -Infinity;
  HEX_DIRS.forEach((d, i) => {
    const ds = -d.q - d.r;
    const hs = -dq - dr;
    const dot = d.q * dq + d.r * dr + ds * hs;
    if (dot > bestDot) {
      bestDot = dot;
      best = i;
    }
  });
  return best;
}

export function placeFootprint(
  shape: Axial[],
  origin: Axial,
  facing: number,
): Axial[] {
  return shape.map((h) => hexAdd(origin, rotate60(h, facing)));
}

export function hexCorners(cx: number, cy: number, size: number): { x: number; y: number }[] {
  const out: { x: number; y: number }[] = [];
  for (let i = 0; i < 6; i++) {
    const angle = (Math.PI / 180) * (60 * i - 30);
    out.push({ x: cx + size * Math.cos(angle), y: cy + size * Math.sin(angle) });
  }
  return out;
}
