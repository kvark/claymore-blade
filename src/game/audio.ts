let ctx: AudioContext | null = null;

function ac() {
  if (typeof window === "undefined") return null;
  if (!ctx) {
    const C = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    ctx = new C();
  }
  if (ctx.state === "suspended") void ctx.resume();
  return ctx;
}

export function unlockAudio() {
  ac();
}

function beep(freq: number, dur: number, type: OscillatorType, gain = 0.04) {
  const c = ac();
  if (!c) return;
  const o = c.createOscillator();
  const g = c.createGain();
  o.type = type;
  o.frequency.value = freq;
  g.gain.value = gain;
  g.gain.exponentialRampToValueAtTime(0.0001, c.currentTime + dur);
  o.connect(g);
  g.connect(c.destination);
  o.start();
  o.stop(c.currentTime + dur);
}

export const sfx = {
  ui: () => beep(520, 0.06, "square", 0.03),
  move: () => beep(180, 0.08, "triangle", 0.03),
  hit: () => beep(140, 0.12, "sawtooth", 0.05),
  miss: () => beep(240, 0.07, "sine", 0.025),
  trans: () => beep(90, 0.18, "sine", 0.05),
  win: () => {
    beep(440, 0.12, "square", 0.04);
    setTimeout(() => beep(660, 0.16, "square", 0.04), 90);
  },
  lose: () => beep(70, 0.3, "sawtooth", 0.05),
};
