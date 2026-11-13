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

/// FLASH (Quicksword) cut-in ease-in: a hair snappier than the claw.
pub const FLASH_IN: f32 = 0.10;
/// FLASH cut-in hold at full strength. The strike already resolved, so this
/// is a purely presentational beat; it never blocks input.
pub const FLASH_HOLD: f32 = 0.20;
/// FLASH cut-in ease-out (camera eases back off Clare).
pub const FLASH_OUT: f32 = 0.22;

/// Which side owns the cut-in: a yoma claw windup (blood) or Clare's FLASH
/// (pale silver / yoki gold-white).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CutInKind {
    #[default]
    Claw,
    Flash,
}

impl CutInKind {
    /// (ease-in, ease-out) seconds.
    pub fn ramps(self) -> (f32, f32) {
        match self {
            CutInKind::Claw => (CUTIN_IN, CUTIN_OUT),
            CutInKind::Flash => (FLASH_IN, FLASH_OUT),
        }
    }
}

/// Anime cut-in: speed lines + camera push-in around one windup or FLASH.
#[derive(Clone, Debug, PartialEq)]
pub struct CutIn {
    pub unit_id: String,
    pub start: f32,
    /// When the claw lands (or the FLASH hold ends); the cut-in eases out
    /// over the kind's ease-out after this.
    pub strike_at: f32,
    pub kind: CutInKind,
}

impl CutIn {
    pub fn end(&self) -> f32 {
        self.strike_at + self.kind.ramps().1
    }

    pub fn strength(&self, now: f32) -> f32 {
        let (rin, rout) = self.kind.ramps();
        envelope(now, self.start, self.strike_at, rin, rout)
    }
}

/// Presentational FLASH cut-in on the striker: the Quicksword already landed
/// (instant resolve), so the push only frames it and then eases back.
pub fn flash_cutin(unit_id: &str, now: f32) -> CutIn {
    CutIn { unit_id: unit_id.into(), start: now, strike_at: now + FLASH_HOLD, kind: CutInKind::Flash }
}

fn smoothstep(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Arm a cut-in for a new windup. A back-to-back claw by the same yoma picks
/// up from the current strength instead of snapping the camera back out.
pub fn rearm_cutin(prev: Option<&CutIn>, unit_id: &str, now: f32, windup: f32) -> CutIn {
    let k = prev
        .filter(|c| c.unit_id == unit_id && c.kind == CutInKind::Claw)
        .map(|c| c.strength(now))
        .unwrap_or(0.0);
    // Invert the smoothstep rise closely enough: start earlier by k of the ramp.
    let start = now - CUTIN_IN * inv_smoothstep(k);
    CutIn {
        unit_id: unit_id.into(),
        start,
        strike_at: now + windup,
        kind: CutInKind::Claw,
    }
}

fn inv_smoothstep(y: f32) -> f32 {
    let y = y.clamp(0.0, 1.0);
    // Closed form inverse of 3x^2 - 2x^3 on [0, 1].
    0.5 - ((1.0 - 2.0 * y).asin() / 3.0).sin()
}

/// 0 → 1 over `CUTIN_IN`, hold until the strike, 1 → 0 over `CUTIN_OUT`.
pub fn cutin_envelope(now: f32, start: f32, strike_at: f32) -> f32 {
    envelope(now, start, strike_at, CUTIN_IN, CUTIN_OUT)
}

fn envelope(now: f32, start: f32, strike_at: f32, rin: f32, rout: f32) -> f32 {
    if now < start {
        return 0.0;
    }
    let rise = smoothstep((now - start) / rin);
    if now <= strike_at {
        return rise;
    }
    // A windup shorter than the ease-in eases out from wherever it reached.
    let peak = smoothstep((strike_at - start) / rin);
    peak * (1.0 - smoothstep((now - strike_at) / rout))
}

/// Zoom during the cut-in. Stays inside the HUD fit cap unless the player is
/// already at it, then overshoots by at most `CUTIN_CAP_SLACK`.
pub fn cutin_zoom(base: f32, cap: f32, strength: f32) -> f32 {
    let target = (base * (1.0 + CUTIN_PUSH)).min(cap.max(base) * (1.0 + CUTIN_CAP_SLACK));
    let target = target.max(base);
    base + (target - base) * strength.clamp(0.0, 1.0)
}

/// Normalized screen rectangle `[x, y, w, h]`.
pub type Rect = [f32; 4];

/// Half-width of the focus clear box around the clawing yoma.
pub const CLEAR_X: f32 = 0.13;
/// Half-height of the focus clear box around the clawing yoma.
pub const CLEAR_Y: f32 = 0.09;

fn overlaps_y(r: &Rect, y0: f32, y1: f32) -> bool {
    r[1] < y1 && r[1] + r[3] > y0
}

/// Horizontal anime speed streaks for the overlay pass: `[x, y, w, h, alpha]`,
/// normalized screen units. Streaks rush in from the screen edges toward the
/// clawing yoma and stop at the first keep-out rect on their row (the board
/// footprint, standing units, the caption plate), so ink never crosses the
/// board or the "YOMA REACHES" caption. The focus box is always kept clear.
pub fn speed_lines(
    strength: f32,
    time: f32,
    focus: [f32; 2],
    aspect: f32,
    keep_out: &[Rect],
) -> Vec<[f32; 5]> {
    const N: usize = 34;
    let mut out = Vec::new();
    if strength <= 0.01 {
        return out;
    }
    let px = 1.0 / 800.0;
    let focus_box = [focus[0] - CLEAR_X, focus[1] - CLEAR_Y, CLEAR_X * 2.0, CLEAR_Y * 2.0];
    for i in 0..N {
        let h = hash(i as u32);
        let h2 = hash(i as u32 ^ 0x9e37);
        let y = 0.12 + 0.74 * h;
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
        let thick = (1.4 + 2.4 * h2) * px * aspect.max(0.5).min(2.0);
        let (y0, y1) = (y - thick * 0.5, y + thick * 0.5);
        // Free span on this row, from the streak's own screen edge to the
        // nearest blocker it would run into.
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        for r in keep_out.iter().chain(std::iter::once(&focus_box)) {
            if !overlaps_y(r, y0, y1) {
                continue;
            }
            if dir > 0.0 {
                hi = hi.min(r[0]);
            } else {
                lo = lo.max(r[0] + r[2]);
            }
        }
        let l = x.max(lo);
        let r = (x + len).min(hi);
        if r - l < 0.01 {
            continue;
        }
        // Ink thickens toward the screen edges, thins as it nears the board.
        let mid = (l + r) * 0.5;
        let far = ((mid - focus[0]).abs() / 0.35).min(1.0);
        let alpha = strength * (0.30 + 0.35 * hash(i as u32 ^ 0x77)) * (0.35 + 0.65 * far);
        out.push([l, y0, r - l, thick, alpha]);
    }
    out
}

/// Keep-out rects for the cut-in streaks: one per board cell (its projected
/// top face, padded) plus a taller box over every standing unit, all in
/// normalized screen units. `cells` / `units` are screen centres at ground
/// height; `tile` is the hex radius in normalized `[x, y]`.
pub fn board_keep_out(cells: &[[f32; 2]], units: &[[f32; 2]], tile: [f32; 2]) -> Vec<Rect> {
    // Iso hex top: about ±1.0 radius wide, ±0.6 tall; ruins rise ~0.6 more.
    let (hw, down, up) = (tile[0] * 1.05, tile[1] * 0.75, tile[1] * 1.3);
    let mut out: Vec<Rect> = cells
        .iter()
        .map(|c| [c[0] - hw, c[1] - up, hw * 2.0, up + down])
        .collect();
    // Archive bodies stand up to ~2 tiles tall (Long Yoma); wings spread wider.
    let (uw, uup) = (tile[0] * 1.4, tile[1] * 2.6);
    out.extend(units.iter().map(|c| [c[0] - uw, c[1] - uup, uw * 2.0, uup + down]));
    out
}

/// Margin kept between streaks and a caption plate.
pub const CAPTION_MARGIN: f32 = 0.012;

/// Caption plate grown by `CAPTION_MARGIN` on every side.
pub fn caption_keep_out(plate: Rect) -> Rect {
    let m = CAPTION_MARGIN;
    [plate[0] - m, plate[1] - m, plate[2] + m * 2.0, plate[3] + m * 2.0]
}

/// Number of stacked edge bands in the cut-in vignette.
pub const WASH_RINGS: usize = 7;
/// Alpha of one vignette band at full strength (bands stack at the rim).
pub const WASH_BAND_ALPHA: f32 = 0.16;
/// Letterbox bar height at full strength (normalized screen height).
pub const LETTERBOX_H: f32 = 0.05;

/// Blood-ink wash for the cut-in as `(rect, rgba)` quads, back to front:
/// a red tint over everything, a red-black vignette that thickens toward the
/// screen edges (stacked bands, darkest at the rim, bottom bands ending at the
/// action panel `panel_top`), and a letterbox bar that slides in at the top.
pub fn cutin_wash(strength: f32, panel_top: f32) -> Vec<(Rect, [f32; 4])> {
    let k = strength.clamp(0.0, 1.0);
    let mut out = Vec::new();
    if k <= 0.01 {
        return out;
    }
    out.push(([0.0, 0.0, 1.0, 1.0], [0.14, 0.0, 0.01, 0.26 * k]));
    let ink = [0.008, 0.0, 0.001];
    for i in 0..WASH_RINGS {
        // Bands grow inward; their overlap stacks into a soft gradient.
        let d = 0.035 * (i + 1) as f32;
        let a = WASH_BAND_ALPHA * k;
        let c = [ink[0], ink[1], ink[2], a];
        out.push(([0.0, 0.0, 1.0, d * 0.8], c)); // top
        out.push(([0.0, panel_top - d * 0.35, 1.0, d * 0.35], c)); // bottom, thin: near row stays lit
        out.push(([0.0, 0.0, d, 1.0], c)); // left
        out.push(([1.0 - d, 0.0, d, 1.0], c)); // right
    }
    let bar = LETTERBOX_H * k;
    // Top bar only: the action panel already frames the bottom, and a second
    // bar above it would clip the board's near row during the push-in.
    out.push(([0.0, 0.0, 1.0, bar], [0.0, 0.0, 0.0, 0.9 * k]));
    out
}

/// Peak alpha the vignette reaches at the screen edge (all bands stacked).
pub fn wash_edge_alpha(strength: f32) -> f32 {
    let a = WASH_BAND_ALPHA * strength.clamp(0.0, 1.0);
    1.0 - (1.0 - a).powi(WASH_RINGS as i32)
}

/// FLASH wash as `(rect, rgba)` quads: a pale yoki lift over everything, a
/// gold-white glare thickening toward the edges (bottom bands end at the
/// action panel `panel_top`) and the same ink letterbox bar as the claw.
pub fn flash_wash(strength: f32, panel_top: f32) -> Vec<(Rect, [f32; 4])> {
    let k = strength.clamp(0.0, 1.0);
    let mut out = Vec::new();
    if k <= 0.01 {
        return out;
    }
    out.push(([0.0, 0.0, 1.0, 1.0], [0.86, 0.88, 0.92, 0.015 * k]));
    for i in 0..WASH_RINGS {
        let d = 0.035 * (i + 1) as f32;
        let c = [0.98, 0.93, 0.76, FLASH_BAND_ALPHA * k];
        out.push(([0.0, 0.0, 1.0, d * 0.8], c));
        out.push(([0.0, panel_top - d * 0.35, 1.0, d * 0.35], c));
        out.push(([0.0, 0.0, d, 1.0], c));
        out.push(([1.0 - d, 0.0, d, 1.0], c));
    }
    out.push(([0.0, 0.0, 1.0, LETTERBOX_H * k], [0.0, 0.0, 0.0, 0.9 * k]));
    out
}

/// Alpha of one FLASH glare band at full strength. Blending runs in linear
/// space on an sRGB target, so pale-on-dark needs far less alpha than the
/// claw's ink to read as the same weight.
pub const FLASH_BAND_ALPHA: f32 = 0.02;
/// Gold-white FLASH speed streaks (claw streaks are bone-white ink).
pub const FLASH_STREAK: [f32; 3] = [1.0, 0.95, 0.74];
/// FLASH cut-in caption.
pub const FLASH_CAPTION: &str = "QUICKSWORD";

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
        let c = CutIn { unit_id: "y".into(), start: s, strike_at: k, kind: CutInKind::Claw };
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
        assert!(speed_lines(0.0, 1.0, [0.5, 0.5], 1.6, &[]).is_empty());
        let focus = [0.5, 0.48];
        for t in [0.0, 0.17, 0.42, 0.9] {
            let lines = speed_lines(1.0, t, focus, 1.6, &[]);
            assert!(lines.len() >= 12, "t={t}: {}", lines.len());
            for l in &lines {
                assert!(l[0] >= 0.0 && l[0] + l[2] <= 1.0 + 1e-5);
                assert!(l[4] > 0.0 && l[4] <= 0.65 + 1e-6);
                let focus_box = [focus[0] - CLEAR_X, focus[1] - CLEAR_Y, CLEAR_X * 2.0, CLEAR_Y * 2.0];
                assert!(!intersects(l, &focus_box), "streak crosses the clawing yoma: {l:?}");
            }
        }
        let weak = speed_lines(0.3, 0.2, focus, 1.6, &[]);
        let strong = speed_lines(1.0, 0.2, focus, 1.6, &[]);
        assert!(weak.iter().zip(&strong).all(|(a, b)| a[4] < b[4]));
    }

    fn intersects(l: &[f32; 5], r: &Rect) -> bool {
        l[0] < r[0] + r[2] - 1e-5
            && l[0] + l[2] > r[0] + 1e-5
            && l[1] < r[1] + r[3]
            && l[1] + l[3] > r[1]
    }

    #[test]
    fn speed_lines_stop_at_the_board_and_the_caption() {
        let focus = [0.52, 0.45];
        let board = [0.30, 0.20, 0.42, 0.60];
        let caption = caption_keep_out([0.49, 0.69, 0.14, 0.03]);
        let keep = [board, caption];
        let mut seen = 0;
        for i in 0..60 {
            let t = i as f32 * 0.037;
            for l in speed_lines(1.0, t, focus, 1.6, &keep) {
                seen += 1;
                assert!(!intersects(&l, &board), "t={t}: streak on the board {l:?}");
                assert!(!intersects(&l, &caption), "t={t}: streak through the caption {l:?}");
            }
        }
        // Streaks still play in the margins around the board.
        assert!(seen > 60 * 6, "only {seen} streaks");
    }

    #[test]
    fn board_keep_out_covers_cells_and_standing_units() {
        let cells = [[0.4, 0.5], [0.6, 0.5]];
        let units = [[0.6, 0.5]];
        let tile = [0.03, 0.02];
        let keep = board_keep_out(&cells, &units, tile);
        assert_eq!(keep.len(), 3);
        let inside = |p: [f32; 2]| {
            keep.iter().any(|r| p[0] >= r[0] && p[0] <= r[0] + r[2] && p[1] >= r[1] && p[1] <= r[1] + r[3])
        };
        // Cell centres and their hex edges.
        for c in cells {
            assert!(inside(c));
            assert!(inside([c[0] - tile[0], c[1]]) && inside([c[0] + tile[0], c[1]]));
        }
        // A body two tiles tall over the unit hex.
        assert!(inside([0.6, 0.5 - tile[1] * 2.4]));
        assert!(!inside([0.4, 0.5 - tile[1] * 2.4]), "empty cells get no body box");
    }

    #[test]
    fn cutin_wash_reads_as_a_cut_in() {
        assert!(cutin_wash(0.0, 0.84).is_empty());
        let full = cutin_wash(1.0, 0.84);
        // Vignette edge is much darker than the old flat 0.16 wash.
        assert!(wash_edge_alpha(1.0) > 0.55, "edge {}", wash_edge_alpha(1.0));
        assert!(wash_edge_alpha(0.5) < wash_edge_alpha(1.0));
        // Centre of the screen only gets the light red tint (board stays readable).
        let centre = [0.5, 0.45];
        let over_centre: Vec<_> = full
            .iter()
            .filter(|(r, _)| centre[0] >= r[0] && centre[0] <= r[0] + r[2] && centre[1] >= r[1] && centre[1] <= r[1] + r[3])
            .collect();
        assert_eq!(over_centre.len(), 1);
        assert!(over_centre[0].1[3] <= 0.3);
        // Bars and bands stay above the action panel.
        for (r, c) in &full {
            assert!(r[1] + r[3] <= 0.84 + 1e-5 || r[3] >= 1.0, "{r:?} spills under the HUD");
            assert!(c[3] > 0.0 && c[3] <= 0.9 + 1e-6);
            assert!(c[0] >= c[1] && c[0] >= c[2], "wash must stay blood/ink, got {c:?}");
        }
        // Letterbox bars slide in with strength.
        let bar = |k: f32| cutin_wash(k, 0.84).iter().rev().next().map(|(r, _)| r[3]).unwrap();
        assert!(bar(0.5) < bar(1.0));
        assert!((bar(1.0) - LETTERBOX_H).abs() < 1e-6);
    }

    #[test]
    fn flash_cutin_is_short_and_eases_back() {
        let c = flash_cutin("clare", 3.0);
        assert_eq!(c.kind, CutInKind::Flash);
        assert_eq!(c.strength(2.99), 0.0);
        assert!((c.strength(3.0 + FLASH_IN * 0.5) - 0.5).abs() < 1e-4);
        assert!((c.strength(3.0 + FLASH_IN) - 1.0).abs() < 1e-6);
        assert!((c.strength(c.strike_at) - 1.0).abs() < 1e-6, "held");
        assert!((c.strength(c.strike_at + FLASH_OUT * 0.5) - 0.5).abs() < 1e-4);
        assert_eq!(c.strength(c.end()), 0.0);
        assert!((c.end() - (3.0 + FLASH_HOLD + FLASH_OUT)).abs() < 1e-6);
        // Shorter than the claw's whole cut-in (windup + ease-out).
        assert!(c.end() - c.start <= 0.5);
        assert!(c.end() - c.start < ENEMY_WINDUP + CUTIN_OUT);
        // A claw arming over a live FLASH (a different unit) starts its own.
        let claw = rearm_cutin(Some(&c), "yoma-0", 3.2, ENEMY_WINDUP);
        assert_eq!(claw.kind, CutInKind::Claw);
        assert_eq!(claw.strength(3.2), 0.0);
    }

    #[test]
    fn flash_wash_is_pale_yoki_not_blood() {
        assert!(flash_wash(0.0, 0.84).is_empty());
        let full = flash_wash(1.0, 0.84);
        assert_eq!(full.len(), cutin_wash(1.0, 0.84).len());
        let centre = [0.5, 0.45];
        let over: Vec<_> = full
            .iter()
            .filter(|(r, _)| centre[0] >= r[0] && centre[0] <= r[0] + r[2] && centre[1] >= r[1] && centre[1] <= r[1] + r[3])
            .collect();
        assert_eq!(over.len(), 1, "centre only gets the light lift");
        assert!(over[0].1[3] <= 0.1, "board stays readable under the glare");
        for (r, c) in &full[..full.len() - 1] {
            assert!(r[1] + r[3] <= 0.84 + 1e-5 || r[3] >= 1.0, "{r:?} spills under the HUD");
            assert!(c[0].min(c[1]).min(c[2]) > 0.7, "pale silver/gold-white, got {c:?}");
            assert!(c[1] > c[0] * 0.8, "never blood-red: {c:?}");
        }
        assert!(FLASH_STREAK[0] >= FLASH_STREAK[1] && FLASH_STREAK[1] > FLASH_STREAK[2]);
    }
}
