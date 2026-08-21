//! Shared screen-space buttons. Click tests and the GPU draw the same rects.

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(self, nx: f32, ny: f32) -> bool {
        nx >= self.x && nx <= self.x + self.w && ny >= self.y && ny <= self.y + self.h
    }

    pub fn pos(self) -> [f32; 4] {
        [self.x, self.y, self.w, self.h]
    }

    pub fn inset(self, p: f32) -> Self {
        Self {
            x: self.x + p,
            y: self.y + p,
            w: (self.w - p * 2.0).max(0.01),
            h: (self.h - p * 2.0).max(0.01),
        }
    }
}

pub fn title_new() -> Rect {
    // Taller on phone (0.10) so thumbs hit reliably
    Rect::new(0.07, 0.68, 0.36, 0.10)
}

pub fn title_continue() -> Rect {
    Rect::new(0.07, 0.80, 0.36, 0.10)
}

pub fn world_codex() -> Rect {
    Rect::new(0.80, 0.90, 0.18, 0.08)
}

pub fn town_hunt() -> Rect {
    Rect::new(0.06, 0.76, 0.24, 0.10)
}

pub fn town_rest() -> Rect {
    Rect::new(0.32, 0.76, 0.22, 0.10)
}

pub fn town_leave() -> Rect {
    Rect::new(0.56, 0.76, 0.22, 0.10)
}

pub fn result_ok() -> Rect {
    Rect::new(0.10, 0.72, 0.30, 0.10)
}

pub fn scene_yes() -> Rect {
    Rect::new(0.08, 0.76, 0.30, 0.10)
}

pub fn scene_no() -> Rect {
    Rect::new(0.42, 0.76, 0.30, 0.10)
}

pub struct CombatBar {
    pub wait: Rect,
    pub raise: Rect,
    pub guard: Rect,
    pub cut: Rect,
    pub slot: Rect,
    pub forfeit: Rect,
}

/// Combat action bar. Raised slightly and taller so it clears phone home
/// indicators and stays finger-sized on ~375 px screens.
pub fn combat_bar() -> CombatBar {
    CombatBar {
        wait: Rect::new(0.04, 0.895, 0.13, 0.085),
        raise: Rect::new(0.18, 0.895, 0.13, 0.085),
        guard: Rect::new(0.32, 0.895, 0.14, 0.085),
        cut: Rect::new(0.47, 0.895, 0.13, 0.085),
        slot: Rect::new(0.61, 0.895, 0.15, 0.085),
        forfeit: Rect::new(0.82, 0.895, 0.14, 0.085),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_new_is_clickable() {
        let r = title_new();
        assert!(r.contains(0.20, 0.72));
        assert!(!r.contains(0.80, 0.20));
    }

    #[test]
    fn combat_wait_sits_on_the_bar() {
        let b = combat_bar();
        assert!(b.wait.contains(0.08, 0.93));
        assert!(!b.forfeit.contains(0.08, 0.93));
    }
}
