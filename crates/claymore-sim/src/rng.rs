/// LCG matching `src/game/sim/rng.ts`.
#[derive(Clone, Debug)]
pub struct Rng {
    s: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self {
            s: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next_f32(&mut self) -> f32 {
        self.s = 1664525u32.wrapping_mul(self.s).wrapping_add(1013904223);
        self.s as f32 / 4294967296.0
    }

    pub fn int(&mut self, min: i32, max: i32) -> i32 {
        let span = (max - min + 1) as f32;
        min + (self.next_f32() * span).floor() as i32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }
}
