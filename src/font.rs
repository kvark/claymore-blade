//! Tiny 5×7 ASCII atlas, packed into an 128×64 Rgba8 texture.

pub const ATLAS_W: u32 = 128;
pub const ATLAS_H: u32 = 64;
pub const CELL: u32 = 8;

/// Rows of 5 bits, 7 rows. Space (32) through ~ (126).
fn glyph(c: u8) -> [u8; 7] {
    match c {
        b'A' => [
            0b01000, 0b10100, 0b10100, 0b11100, 0b10100, 0b10100, 0b00000,
        ],
        b'B' => [
            0b11000, 0b10100, 0b11000, 0b10100, 0b10100, 0b11000, 0b00000,
        ],
        b'C' => [
            0b01100, 0b10000, 0b10000, 0b10000, 0b10000, 0b01100, 0b00000,
        ],
        b'D' => [
            0b11000, 0b10100, 0b10100, 0b10100, 0b10100, 0b11000, 0b00000,
        ],
        b'E' => [
            0b11100, 0b10000, 0b11000, 0b10000, 0b10000, 0b11100, 0b00000,
        ],
        b'F' => [
            0b11100, 0b10000, 0b11000, 0b10000, 0b10000, 0b10000, 0b00000,
        ],
        b'G' => [
            0b01100, 0b10000, 0b10000, 0b10100, 0b10100, 0b01100, 0b00000,
        ],
        b'H' => [
            0b10100, 0b10100, 0b11100, 0b10100, 0b10100, 0b10100, 0b00000,
        ],
        b'I' => [
            0b11100, 0b01000, 0b01000, 0b01000, 0b01000, 0b11100, 0b00000,
        ],
        b'J' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b10100, 0b01000, 0b00000,
        ],
        b'K' => [
            0b10100, 0b10100, 0b11000, 0b10100, 0b10100, 0b10100, 0b00000,
        ],
        b'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11100, 0b00000,
        ],
        b'M' => [
            0b10100, 0b11100, 0b11100, 0b10100, 0b10100, 0b10100, 0b00000,
        ],
        b'N' => [
            0b10100, 0b11100, 0b11100, 0b11100, 0b10100, 0b10100, 0b00000,
        ],
        b'O' => [
            0b01000, 0b10100, 0b10100, 0b10100, 0b10100, 0b01000, 0b00000,
        ],
        b'P' => [
            0b11000, 0b10100, 0b10100, 0b11000, 0b10000, 0b10000, 0b00000,
        ],
        b'Q' => [
            0b01000, 0b10100, 0b10100, 0b10100, 0b01100, 0b00100, 0b00000,
        ],
        b'R' => [
            0b11000, 0b10100, 0b10100, 0b11000, 0b10100, 0b10100, 0b00000,
        ],
        b'S' => [
            0b01100, 0b10000, 0b01000, 0b00100, 0b00100, 0b11000, 0b00000,
        ],
        b'T' => [
            0b11100, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b00000,
        ],
        b'U' => [
            0b10100, 0b10100, 0b10100, 0b10100, 0b10100, 0b01100, 0b00000,
        ],
        b'V' => [
            0b10100, 0b10100, 0b10100, 0b10100, 0b01000, 0b01000, 0b00000,
        ],
        b'W' => [
            0b10100, 0b10100, 0b10100, 0b11100, 0b11100, 0b10100, 0b00000,
        ],
        b'X' => [
            0b10100, 0b10100, 0b01000, 0b01000, 0b10100, 0b10100, 0b00000,
        ],
        b'Y' => [
            0b10100, 0b10100, 0b01000, 0b01000, 0b01000, 0b01000, 0b00000,
        ],
        b'Z' => [
            0b11100, 0b00100, 0b01000, 0b01000, 0b10000, 0b11100, 0b00000,
        ],
        b'0' => [
            0b01000, 0b10100, 0b10100, 0b10100, 0b10100, 0b01000, 0b00000,
        ],
        b'1' => [
            0b01000, 0b11000, 0b01000, 0b01000, 0b01000, 0b11100, 0b00000,
        ],
        b'2' => [
            0b11000, 0b00100, 0b00100, 0b01000, 0b10000, 0b11100, 0b00000,
        ],
        b'3' => [
            0b11000, 0b00100, 0b01000, 0b00100, 0b00100, 0b11000, 0b00000,
        ],
        b'4' => [
            0b10100, 0b10100, 0b11100, 0b00100, 0b00100, 0b00100, 0b00000,
        ],
        b'5' => [
            0b11100, 0b10000, 0b11000, 0b00100, 0b00100, 0b11000, 0b00000,
        ],
        b'6' => [
            0b01100, 0b10000, 0b11000, 0b10100, 0b10100, 0b01000, 0b00000,
        ],
        b'7' => [
            0b11100, 0b00100, 0b01000, 0b01000, 0b10000, 0b10000, 0b00000,
        ],
        b'8' => [
            0b01000, 0b10100, 0b01000, 0b10100, 0b10100, 0b01000, 0b00000,
        ],
        b'9' => [
            0b01000, 0b10100, 0b10100, 0b01100, 0b00100, 0b11000, 0b00000,
        ],
        b'.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01000, 0b00000,
        ],
        b',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b01000, 0b10000, 0b00000,
        ],
        b':' => [
            0b00000, 0b01000, 0b00000, 0b00000, 0b01000, 0b00000, 0b00000,
        ],
        b'-' => [
            0b00000, 0b00000, 0b00000, 0b11100, 0b00000, 0b00000, 0b00000,
        ],
        b'+' => [
            0b00000, 0b01000, 0b01000, 0b11100, 0b01000, 0b01000, 0b00000,
        ],
        b'/' => [
            0b00100, 0b00100, 0b01000, 0b01000, 0b10000, 0b10000, 0b00000,
        ],
        b'\'' => [
            0b01000, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        b'(' => [
            0b01000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01000, 0b00000,
        ],
        b')' => [
            0b10000, 0b01000, 0b01000, 0b01000, 0b01000, 0b10000, 0b00000,
        ],
        b'?' => [
            0b01000, 0b10100, 0b00100, 0b01000, 0b00000, 0b01000, 0b00000,
        ],
        b'!' => [
            0b01000, 0b01000, 0b01000, 0b01000, 0b00000, 0b01000, 0b00000,
        ],
        b'[' => [
            0b11000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11000, 0b00000,
        ],
        b']' => [
            0b11000, 0b01000, 0b01000, 0b01000, 0b01000, 0b11000, 0b00000,
        ],
        _ => [0; 7],
    }
}

pub fn atlas_rgba() -> Vec<u8> {
    let mut px = vec![0u8; (ATLAS_W * ATLAS_H * 4) as usize];
    for c in 32u8..127 {
        let col = (c - 32) % 16;
        let row = (c - 32) / 16;
        let bits = glyph(c);
        let ox = col as u32 * CELL;
        let oy = row as u32 * CELL;
        for (y, row_bits) in bits.iter().enumerate() {
            for x in 0..5 {
                if row_bits & (1 << (4 - x)) != 0 {
                    let i = (((oy + y as u32 + 1) * ATLAS_W + ox + x as u32 + 1) * 4) as usize;
                    if i + 3 < px.len() {
                        px[i] = 230;
                        px[i + 1] = 220;
                        px[i + 2] = 200;
                        px[i + 3] = 255;
                    }
                }
            }
        }
    }
    px
}

pub fn glyph_uv(c: char) -> (f32, f32, f32, f32) {
    let b = if c.is_ascii() { c as u8 } else { b'?' };
    let idx = b.saturating_sub(32).min(94);
    let col = (idx % 16) as f32;
    let row = (idx / 16) as f32;
    let u = col * CELL as f32 / ATLAS_W as f32;
    let v = row * CELL as f32 / ATLAS_H as f32;
    let du = CELL as f32 / ATLAS_W as f32;
    let dv = CELL as f32 / ATLAS_H as f32;
    (u, v, du, dv)
}

/// Glyph width / height in normalized units on the 16:10 design screen.
pub const GLYPH_ASPECT: f32 = 0.7;
/// Pen advance as a fraction of glyph width.
pub const ADVANCE: f32 = 0.85;
/// Screen aspect the HUD type was laid out for (1280x800).
pub const DESIGN_ASPECT: f32 = 1.6;

thread_local! {
    static SCREEN: std::cell::Cell<[f32; 2]> = const { std::cell::Cell::new([1280.0, 800.0]) };
}

/// Current target size in pixels (set by the renderer each frame).
pub fn set_screen(w: f32, h: f32) {
    SCREEN.with(|s| s.set([w.max(1.0), h.max(1.0)]));
}

/// `[width, height]` multipliers on a nominal normalized glyph so its pixel
/// aspect stays the 1280x800 design's on any screen. Height follows
/// `min(w, h)` (tall phones get smaller type, not squeezed type).
pub fn glyph_scale_for(w: f32, h: f32) -> [f32; 2] {
    let a = w.max(1.0) / h.max(1.0);
    let sh = a.min(1.0);
    [DESIGN_ASPECT * sh / a, sh]
}

pub fn glyph_scale() -> [f32; 2] {
    SCREEN.with(|s| {
        let [w, h] = s.get();
        glyph_scale_for(w, h)
    })
}

/// Normalized glyph width for a nominal `glyph_h`.
pub fn glyph_w(glyph_h: f32) -> f32 {
    glyph_h * GLYPH_ASPECT * glyph_scale()[0]
}

/// Normalized drawn glyph height for a nominal `glyph_h` (centered in it).
pub fn glyph_draw_h(glyph_h: f32) -> f32 {
    glyph_h * glyph_scale()[1]
}

/// Normalized width of `s` at nominal `glyph_h` (newlines ignored).
pub fn text_w(s: &str, glyph_h: f32) -> f32 {
    let n = s.chars().filter(|c| *c != '\n').count() as f32;
    if n > 0.0 {
        glyph_w(glyph_h) * ((n - 1.0) * ADVANCE + 1.0)
    } else {
        0.0
    }
}

/// `glyph_h`, shrunk so `s` fits in `max_w`.
pub fn fit(s: &str, glyph_h: f32, max_w: f32) -> f32 {
    let tw = text_w(s, glyph_h);
    if tw <= max_w || tw <= 0.0 {
        glyph_h
    } else {
        glyph_h * max_w / tw
    }
}

#[cfg(test)]
mod metric_tests {
    use super::*;

    #[test]
    fn glyph_pixel_aspect_is_constant_and_1280_is_unchanged() {
        let px_aspect = |w: f32, h: f32| {
            let [sw, sh] = glyph_scale_for(w, h);
            (GLYPH_ASPECT * sw * w) / (sh * h)
        };
        let want = GLYPH_ASPECT * DESIGN_ASPECT;
        for (w, h) in [(390.0, 844.0), (1280.0, 800.0), (1920.0, 1080.0), (800.0, 800.0)] {
            assert!((px_aspect(w, h) - want).abs() < 1e-4, "{w}x{h}");
        }
        assert_eq!(glyph_scale_for(1280.0, 800.0), [1.0, 1.0]);
        // 16:9 keeps full height; 390x844 gets min(w, h)-sized type.
        assert_eq!(glyph_scale_for(1920.0, 1080.0)[1], 1.0);
        assert!((glyph_scale_for(390.0, 844.0)[1] - 390.0 / 844.0).abs() < 1e-6);
    }
}
