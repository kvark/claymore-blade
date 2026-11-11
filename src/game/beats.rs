//! Pacing for enemy turns: one AI action per beat, claw windups as cut-ins.
//!
//! Everything here is presentation timing. Combat state only changes when a
//! beat fires (`Game::tick_enemy_tell`), so saves stay timer-free.

/// Seconds between enemy Raise / Move / Wait actions (queue chips visibly advance).
pub const ENEMY_BEAT: f32 = 0.3;
/// Claw windup before the strike resolves (the readable tell from #20).
pub const ENEMY_WINDUP: f32 = 0.65;
/// Hard cap on paced actions in one enemy phase (mirrors `run_ai_prep`'s guard).
pub const ENEMY_PHASE_MAX_BEATS: u32 = 48;

/// Cut-in ease-in after the windup is armed.
pub const CUTIN_IN: f32 = 0.14;
/// Cut-in ease-out after the strike lands (camera eases back).
pub const CUTIN_OUT: f32 = 0.30;
/// Relative camera push-in at full cut-in strength.
pub const CUTIN_PUSH: f32 = 0.16;
/// How far past the HUD-fit zoom cap the push may briefly go (the fit cap
/// includes generous fighter headroom, so ~10% keeps the board on screen).
pub const CUTIN_CAP_SLACK: f32 = 0.10;
/// Fraction of the way the clawing yoma drifts toward the board center.
pub const CUTIN_DRIFT: f32 = 0.22;

/// Enemy-phase speed while the player holds the fast-forward key.
pub const FAST_FORWARD: f32 = 2.0;

/// Extra presentation time to skip this frame so beats, windups and the
/// cut-in all run `FAST_FORWARD`x while held (real-time otherwise).
pub fn fast_forward_skip(held: bool, enemy_phase: bool, dt: f32) -> f32 {
    if held && enemy_phase {
        dt.max(0.0) * (FAST_FORWARD - 1.0)
    } else {
        0.0
    }
}

/// When the next enemy beat fires, given the time the turn reached an enemy.
pub fn next_beat_at(now: f32) -> f32 {
    now + ENEMY_BEAT
}

pub fn beat_due(now: f32, at: Option<f32>) -> bool {
    at.map(|t| now + 1e-4 >= t).unwrap_or(false)
}

/// Anime claw cut-in: speed lines + camera push-in around one windup.
#[derive(Clone, Debug, PartialEq)]
pub struct CutIn {
    pub unit_id: String,
    pub start: f32,
    /// When the claw lands; the cut-in eases out over `CUTIN_OUT` after this.
    pub strike_at: f32,
}

impl CutIn {
    pub fn end(&self) -> f32 {
        self.strike_at + CUTIN_OUT
    }

    pub fn strength(&self, now: f32) -> f32 {
        cutin_envelope(now, self.start, self.strike_at)
    }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Arm a cut-in for a new windup. A back-to-back claw by the same yoma picks
/// up from the current strength instead of snapping the camera back out.
pub fn rearm_cutin(prev: Option<&CutIn>, unit_id: &str, now: f32, windup: f32) -> CutIn {
    let k = prev
        .filter(|c| c.unit_id == unit_id)
        .map(|c| c.strength(now))
        .unwrap_or(0.0);
    // Invert the smoothstep rise closely enough: start earlier by k of the ramp.
    let start = now - CUTIN_IN * inv_smoothstep(k);
    CutIn {
        unit_id: unit_id.into(),
        start,
        strike_at: now + windup,
    }
}

fn inv_smoothstep(y: f32) -> f32 {
    let y = y.clamp(0.0, 1.0);
    // Closed form inverse of 3x^2 - 2x^3 on [0, 1].
    0.5 - ((1.0 - 2.0 * y).asin() / 3.0).sin()
}

/// 0 → 1 over `CUTIN_IN`, hold until the strike, 1 → 0 over `CUTIN_OUT`.
pub fn cutin_envelope(now: f32, start: f32, strike_at: f32) -> f32 {
    if now < start {
        return 0.0;
    }
    let rise = smoothstep((now - start) / CUTIN_IN);
    if now <= strike_at {
        return rise;
    }
    // A windup shorter than CUTIN_IN eases out from wherever it reached.
    let peak = smoothstep((strike_at - start) / CUTIN_IN);
    peak * (1.0 - smoothstep((now - strike_at) / CUTIN_OUT))
}

/// Zoom during the cut-in. Stays inside the HUD fit cap unless the player is
/// already at it, then overshoots by at most `CUTIN_CAP_SLACK`.
pub fn cutin_zoom(base: f32, cap: f32, strength: f32) -> f32 {
    let target = (base * (1.0 + CUTIN_PUSH)).min(cap.max(base) * (1.0 + CUTIN_CAP_SLACK));
    let target = target.max(base);
    base + (target - base) * strength.clamp(0.0, 1.0)
}

/// Horizontal anime speed streaks for the overlay pass: `[x, y, w, h, alpha]`,
/// normalized screen units. Streaks keep clear of the focus (clawing yoma).
pub fn speed_lines(strength: f32, time: f32, focus: [f32; 2], aspect: f32) -> Vec<[f32; 5]> {
    const N: usize = 26;
    const CLEAR_X: f32 = 0.13;
    const CLEAR_Y: f32 = 0.09;
    let mut out = Vec::new();
    if strength <= 0.01 {
        return out;
    }
    let px = 1.0 / 800.0;
    for i in 0..N {
        let h = hash(i as u32);
        let h2 = hash(i as u32 ^ 0x9e37);
        let y = 0.14 + 0.72 * h;
        let len = 0.10 + 0.26 * h2;
        let speed = 2.2 + 1.6 * hash(i as u32 ^ 0x51ed);
        // Streaks rush toward the focus: left half moves right, right half moves left.
        let dir = if i % 2 == 0 { 1.0 } else { -1.0 };
        let phase = (hash(i as u32 ^ 0x2545) + time * speed / 1.6).fract();
        let x = if dir > 0.0 {
            -len + phase * (1.0 + len)
        } else {
            1.0 - phase * (1.0 + len)
        };
        let thick = (1.2 + 2.0 * h2) * px * aspect.max(0.5).min(2.0);
        let mut seg = [x, y - thick * 0.5, len, thick];
        // Carve the clear ellipse-ish box around the focus.
        if (y - focus[1]).abs() < CLEAR_Y {
            let (cl, cr) = (focus[0] - CLEAR_X, focus[0] + CLEAR_X);
            let (sl, sr) = (seg[0], seg[0] + seg[2]);
            if sr > cl && sl < cr {
                // Keep the longer side outside the clear box.
                let left = (cl - sl).max(0.0);
                let right = (sr - cr).max(0.0);
                if left <= 0.0 && right <= 0.0 {
                    continue;
                }
                if left >= right {
                    seg[2] = left;
                } else {
                    seg[0] = cr;
                    seg[2] = right;
                }
            }
        }
        // Clip to the screen.
        let l = seg[0].max(0.0);
        let r = (seg[0] + seg[2]).min(1.0);
        if r - l < 0.01 {
            continue;
        }
        seg[0] = l;
        seg[2] = r - l;
        // Ink thickens toward the screen edges, thins near the clawing yoma.
        let mid = seg[0] + seg[2] * 0.5;
        let far = ((mid - focus[0]).abs() / 0.35).min(1.0);
        let alpha = strength * (0.25 + 0.35 * hash(i as u32 ^ 0x77)) * (0.35 + 0.65 * far);
        out.push([seg[0], seg[1], seg[2], seg[3], alpha]);
    }
    out
}

fn hash(i: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9).wrapping_add(0x7F4A_7C15);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_forward_only_skips_time_while_held_in_enemy_phase() {
        assert_eq!(fast_forward_skip(false, true, 0.016), 0.0);
        assert_eq!(fast_forward_skip(true, false, 0.016), 0.0);
        let skip = fast_forward_skip(true, true, 0.016);
        assert!((skip - 0.016 * (FAST_FORWARD - 1.0)).abs() < 1e-7);
        assert!((FAST_FORWARD - 2.0).abs() < 1e-6, "about 2x");
        assert_eq!(fast_forward_skip(true, true, -1.0), 0.0);
    }

    #[test]
    fn beat_spacing_is_point_three_seconds() {
        assert!((next_beat_at(1.0) - 1.3).abs() < 1e-6);
        assert!(!beat_due(1.29, Some(1.3)));
        assert!(beat_due(1.3, Some(1.3)));
        assert!(!beat_due(10.0, None));
    }

    #[test]
    fn cutin_envelope_rises_holds_and_eases_back() {
        let (s, k) = (2.0, 2.0 + ENEMY_WINDUP);
        assert_eq!(cutin_envelope(1.9, s, k), 0.0);
        assert_eq!(cutin_envelope(s, s, k), 0.0);
        let mid_in = cutin_envelope(s + CUTIN_IN * 0.5, s, k);
        assert!((mid_in - 0.5).abs() < 1e-4, "smoothstep midpoint, got {mid_in}");
        assert!((cutin_envelope(s + CUTIN_IN, s, k) - 1.0).abs() < 1e-6);
        assert!((cutin_envelope(k, s, k) - 1.0).abs() < 1e-6, "held through the windup");
        let mid_out = cutin_envelope(k + CUTIN_OUT * 0.5, s, k);
        assert!((mid_out - 0.5).abs() < 1e-4);
        assert_eq!(cutin_envelope(k + CUTIN_OUT, s, k), 0.0);
        // Monotonic in, monotonic out.
        let mut prev = 0.0;
        for i in 0..=20 {
            let v = cutin_envelope(s + CUTIN_IN * i as f32 / 20.0, s, k);
            assert!(v + 1e-6 >= prev);
            prev = v;
        }
        for i in 0..=20 {
            let v = cutin_envelope(k + CUTIN_OUT * i as f32 / 20.0, s, k);
            assert!(v <= prev + 1e-6);
            prev = v;
        }
        let c = CutIn { unit_id: "y".into(), start: s, strike_at: k };
        assert!((c.end() - (k + CUTIN_OUT)).abs() < 1e-6);
        // Whole cut-in is short: windup + ease-out.
        assert!(c.end() - c.start <= 1.0);
    }

    #[test]
    fn back_to_back_claws_keep_the_push_continuous() {
        let first = rearm_cutin(None, "yoma-0", 1.0, ENEMY_WINDUP);
        assert_eq!(first.start, 1.0);
        // Mid ease-out of the first claw.
        let now = first.strike_at + CUTIN_OUT * 0.4;
        let k = first.strength(now);
        assert!(k > 0.2 && k < 0.9);
        let next = rearm_cutin(Some(&first), "yoma-0", now, ENEMY_WINDUP);
        assert!((next.strength(now) - k).abs() < 1e-3, "no snap: {} vs {k}", next.strength(now));
        assert!((next.strike_at - (now + ENEMY_WINDUP)).abs() < 1e-6);
        // Another yoma starts its own cut-in from zero.
        let other = rearm_cutin(Some(&first), "yoma-1", now, ENEMY_WINDUP);
        assert_eq!(other.strength(now), 0.0);
    }

    #[test]
    fn cutin_zoom_respects_hud_cap() {
        // Room under the cap: full push.
        assert!((cutin_zoom(1.0, 1.4, 1.0) - 1.0 * (1.0 + CUTIN_PUSH)).abs() < 1e-6);
        assert_eq!(cutin_zoom(1.0, 1.4, 0.0), 1.0);
        // Already at the cap: overshoot bounded by the slack.
        let z = cutin_zoom(1.2, 1.2, 1.0);
        assert!(z <= 1.2 * (1.0 + CUTIN_CAP_SLACK) + 1e-6);
        assert!(z > 1.2);
        // Never pulls out.
        assert!(cutin_zoom(1.3, 1.0, 1.0) >= 1.3);
    }

    #[test]
    fn speed_lines_scale_with_strength_and_clear_the_focus() {
        assert!(speed_lines(0.0, 1.0, [0.5, 0.5], 1.6).is_empty());
        let focus = [0.5, 0.48];
        for t in [0.0, 0.17, 0.42, 0.9] {
            let lines = speed_lines(1.0, t, focus, 1.6);
            assert!(lines.len() >= 12, "t={t}: {}", lines.len());
            for l in &lines {
                assert!(l[0] >= 0.0 && l[0] + l[2] <= 1.0 + 1e-5);
                assert!(l[4] > 0.0 && l[4] <= 0.6 + 1e-6);
                let cy = l[1] + l[3] * 0.5;
                if (cy - focus[1]).abs() < 0.09 {
                    assert!(
                        l[0] + l[2] <= focus[0] - 0.13 + 1e-5 || l[0] >= focus[0] + 0.13 - 1e-5,
                        "streak crosses the clawing yoma: {l:?}"
                    );
                }
            }
        }
        let weak = speed_lines(0.3, 0.2, focus, 1.6);
        let strong = speed_lines(1.0, 0.2, focus, 1.6);
        assert!(weak.iter().zip(&strong).all(|(a, b)| a[4] < b[4]));
    }
}
