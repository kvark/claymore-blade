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
    Rect::new(0.07, 0.70, 0.32, 0.09)
}

pub fn title_continue() -> Rect {
    Rect::new(0.07, 0.81, 0.32, 0.09)
}

pub fn world_codex() -> Rect {
    Rect::new(0.82, 0.92, 0.16, 0.06)
}

pub fn town_hunt() -> Rect {
    Rect::new(0.07, 0.78, 0.22, 0.09)
}

pub fn town_rest() -> Rect {
    Rect::new(0.32, 0.78, 0.20, 0.09)
}

pub fn town_leave() -> Rect {
    Rect::new(0.55, 0.78, 0.20, 0.09)
}

pub fn result_ok() -> Rect {
    Rect::new(0.10, 0.74, 0.28, 0.09)
}

pub fn scene_yes() -> Rect {
    Rect::new(0.10, 0.78, 0.28, 0.09)
}

pub fn scene_no() -> Rect {
    Rect::new(0.42, 0.78, 0.28, 0.09)
}

pub struct CombatBar {
    pub wait: Rect,
    pub raise: Rect,
    pub guard: Rect,
    pub cut: Rect,
    pub slot: Rect,
    pub forfeit: Rect,
}

pub fn combat_bar() -> CombatBar {
    CombatBar {
        wait: Rect::new(0.105, 0.912, 0.10, 0.07),
        raise: Rect::new(0.215, 0.912, 0.10, 0.07),
        guard: Rect::new(0.325, 0.912, 0.11, 0.07),
        cut: Rect::new(0.445, 0.912, 0.10, 0.07),
        slot: Rect::new(0.555, 0.912, 0.12, 0.07),
        forfeit: Rect::new(0.875, 0.912, 0.11, 0.07),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_new_is_clickable() {
        let r = title_new();
        assert!(r.contains(0.20, 0.74));
        assert!(!r.contains(0.80, 0.20));
    }

    #[test]
    fn combat_wait_sits_on_the_bar() {
        let b = combat_bar();
        assert!(b.wait.contains(0.12, 0.94));
        assert!(!b.forfeit.contains(0.12, 0.94));
    }
}
