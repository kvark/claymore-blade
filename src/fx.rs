//! Presentation juice. Never mutates combat math.

use crate::clip::ClipBank;
use crate::rng::Rng;

pub use crate::clip::FightClip;

#[derive(Clone, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub size: f32,
    pub sprite: &'static str,
    pub tint: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Floater {
    pub x: f32,
    pub y: f32,
    pub vy: f32,
    pub life: f32,
    pub max: f32,
    pub text: String,
    pub tint: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Burst {
    pub x: f32,
    pub y: f32,
    pub life: f32,
    pub max: f32,
    pub size: f32,
    pub sprite: &'static str,
    pub tint: [f32; 4],
}

pub struct Fx {
    pub time: f32,
    pub trauma: f32,
    pub hitstop: f32,
    pub flash: f32,
    pub particles: Vec<Particle>,
    pub floaters: Vec<Floater>,
    pub bursts: Vec<Burst>,
    pub clips: ClipBank,
    rng: Rng,
}

impl Default for Fx {
    fn default() -> Self {
        Self {
            time: 0.0,
            trauma: 0.0,
            hitstop: 0.0,
            flash: 0.0,
            particles: Vec::new(),
            floaters: Vec::new(),
            bursts: Vec::new(),
            clips: ClipBank::default(),
            rng: Rng::new(0xC1A4_0001),
        }
    }
}

impl Fx {
    pub fn tick(&mut self, dt: f32) {
        let dt = dt.clamp(0.0, 0.08);
        let sim = if self.hitstop > 0.0 {
            self.hitstop = (self.hitstop - dt).max(0.0);
            dt * 0.15
        } else {
            dt
        };
        self.time += sim;
        self.trauma = (self.trauma - sim * 1.8).max(0.0);
        self.flash = (self.flash - sim * 4.5).max(0.0);
        for p in &mut self.particles {
            p.x += p.vx * sim;
            p.y += p.vy * sim;
            p.vy -= 0.12 * sim;
            p.life -= sim;
        }
        self.particles.retain(|p| p.life > 0.0);
        if self.particles.len() > 80 {
            self.particles.drain(0..self.particles.len() - 80);
        }
        for f in &mut self.floaters {
            f.y += f.vy * sim;
            f.life -= sim;
        }
        self.floaters.retain(|f| f.life > 0.0);
        for b in &mut self.bursts {
            b.life -= sim;
        }
        self.bursts.retain(|b| b.life > 0.0);
        self.clips.tick(self.time);
    }

    pub fn shake(&self, w: f32, h: f32) -> [f32; 2] {
        let t = self.trauma.clamp(0.0, 1.0);
        let mag = t * t;
        if mag < 0.002 {
            return [0.0, 0.0];
        }
        let n1 = ((self.time * 37.1).sin() * 0.6 + (self.time * 53.7).sin() * 0.4) * mag;
        let n2 = ((self.time * 41.9).cos() * 0.6 + (self.time * 29.3).sin() * 0.4) * mag;
        [n1 * w * 0.018, n2 * h * 0.016]
    }

    pub fn add_trauma(&mut self, v: f32) {
        self.trauma = (self.trauma + v).clamp(0.0, 1.0);
    }

    pub fn emit_dust(&mut self, x: f32, y: f32) {
        for _ in 0..4 {
            self.spawn_p(
                x,
                y,
                0.04,
                "kenney/fx/dust.png",
                [0.48, 0.42, 0.34, 0.85],
                0.45,
            );
        }
    }

    pub fn emit_step(&mut self, x: f32, y: f32) {
        self.emit_dust(x, y + 0.02);
        self.spawn_burst(
            x,
            y + 0.03,
            0.05,
            "kenney/fx/dust.png",
            [0.42, 0.38, 0.30, 0.7],
            0.28,
        );
    }

    pub fn emit_hit(&mut self, x: f32, y: f32, dmg: i32, kind: &str) {
        let (sprite, tint, trauma, flash) = match kind {
            "miss" => (
                "kenney/fx/twirl.png",
                [0.55, 0.55, 0.6, 0.7],
                0.08,
                0.0,
            ),
            "block" => (
                "kenney/fx/circle.png",
                [0.72, 0.7, 0.62, 0.9],
                0.22,
                0.08,
            ),
            "death" => (
                "kenney/fx/smoke.png",
                [0.35, 0.28, 0.24, 0.95],
                0.72,
                0.22,
            ),
            _ => (
                "kenney/fx/slash.png",
                [0.82, 0.18, 0.14, 1.0],
                0.42,
                0.16,
            ),
        };
        self.add_trauma(trauma);
        self.flash = (self.flash + flash).min(0.45);
        if kind == "hit" || kind == "death" {
            self.hitstop = self.hitstop.max(0.05);
        }
        self.spawn_burst(x, y, if kind == "death" { 0.16 } else { 0.11 }, sprite, tint, 0.32);
        if kind == "hit" || kind == "death" {
            self.spawn_burst(
                x + 0.01,
                y,
                0.09,
                "kenney/fx/slash2.png",
                [0.9, 0.85, 0.78, 0.85],
                0.22,
            );
            for _ in 0..8 {
                self.spawn_p(x, y, 0.035, "kenney/fx/spark.png", [0.92, 0.55, 0.28, 0.95], 0.4);
            }
        }
        if kind == "death" {
            for _ in 0..10 {
                self.spawn_p(x, y, 0.06, "kenney/fx/smoke.png", [0.28, 0.24, 0.22, 0.8], 0.7);
            }
            self.spawn_burst(x, y, 0.14, "kenney/fx/scorch.png", [0.2, 0.08, 0.06, 0.7], 0.8);
        }
        if dmg > 0 {
            self.floaters.push(Floater {
                x: x - 0.01,
                y: y - 0.04,
                vy: -0.12,
                life: 0.85,
                max: 0.85,
                text: format!("{dmg}"),
                tint: [0.92, 0.22, 0.16, 1.0],
            });
        } else if kind == "miss" {
            self.floaters.push(Floater {
                x: x - 0.02,
                y: y - 0.04,
                vy: -0.08,
                life: 0.6,
                max: 0.6,
                text: "MISS".into(),
                tint: [0.72, 0.7, 0.62, 1.0],
            });
        }
    }

    pub fn emit_raise(&mut self, x: f32, y: f32) {
        self.spawn_burst(x, y, 0.14, "kenney/fx/magic.png", [0.62, 0.16, 0.22, 0.9], 0.45);
        self.spawn_burst(x, y, 0.1, "kenney/fx/light.png", [0.85, 0.55, 0.4, 0.55], 0.3);
        for _ in 0..6 {
            self.spawn_p(x, y, 0.04, "kenney/fx/star.png", [0.78, 0.28, 0.32, 0.9], 0.55);
        }
        self.add_trauma(0.12);
    }

    pub fn emit_guard(&mut self, x: f32, y: f32) {
        self.spawn_burst(x, y, 0.12, "kenney/fx/circle.png", [0.7, 0.68, 0.55, 0.8], 0.4);
        self.add_trauma(0.08);
    }

    pub fn emit_heal(&mut self, x: f32, y: f32, n: i32) {
        self.spawn_burst(x, y, 0.1, "kenney/fx/light.png", [0.45, 0.7, 0.4, 0.7], 0.4);
        self.floaters.push(Floater {
            x,
            y: y - 0.03,
            vy: -0.1,
            life: 0.7,
            max: 0.7,
            text: format!("+{n}"),
            tint: [0.55, 0.82, 0.45, 1.0],
        });
    }

    pub fn emit_win(&mut self) {
        for _ in 0..18 {
            let x = 0.2 + self.rng.next_f32() * 0.6;
            let y = 0.3 + self.rng.next_f32() * 0.4;
            self.spawn_p(x, y, 0.05, "kenney/fx/star.png", [0.82, 0.7, 0.42, 0.9], 1.1);
        }
        self.flash = 0.2;
    }

    pub fn emit_mote(&mut self, x: f32, y: f32) {
        if self.particles.len() > 40 {
            return;
        }
        self.spawn_p(
            x,
            y,
            0.02,
            "kenney/fx/dust.png",
            [0.55, 0.48, 0.38, 0.35],
            2.4,
        );
    }

    fn spawn_p(
        &mut self,
        x: f32,
        y: f32,
        size: f32,
        sprite: &'static str,
        tint: [f32; 4],
        life: f32,
    ) {
        let a = self.rng.next_f32() * std::f32::consts::TAU;
        let s = 0.04 + self.rng.next_f32() * 0.12;
        self.particles.push(Particle {
            x: x + (self.rng.next_f32() - 0.5) * 0.02,
            y: y + (self.rng.next_f32() - 0.5) * 0.02,
            vx: a.cos() * s,
            vy: a.sin() * s * 0.6 - 0.04,
            life,
            max: life,
            size: size * (0.7 + self.rng.next_f32() * 0.6),
            sprite,
            tint,
        });
    }

    fn spawn_burst(
        &mut self,
        x: f32,
        y: f32,
        size: f32,
        sprite: &'static str,
        tint: [f32; 4],
        life: f32,
    ) {
        self.bursts.push(Burst {
            x,
            y,
            life,
            max: life,
            size,
            sprite,
            tint,
        });
    }

    pub fn play_clip(&mut self, unit: &str, clip: FightClip) {
        self.clips.play(unit, clip, self.time);
    }

    pub fn clip_of(&self, unit: &str) -> (FightClip, f32) {
        self.clips.of(unit, self.time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trauma_decays() {
        let mut fx = Fx::default();
        fx.add_trauma(0.8);
        fx.tick(0.5);
        assert!(fx.trauma < 0.8);
        assert!(fx.trauma > 0.0);
    }

    #[test]
    fn hit_spawns_feedback() {
        let mut fx = Fx::default();
        fx.emit_hit(0.5, 0.5, 12, "hit");
        assert!(!fx.particles.is_empty());
        assert!(!fx.floaters.is_empty());
        assert!(fx.trauma > 0.0);
    }

    #[test]
    fn clip_plays_then_expires() {
        let mut fx = Fx::default();
        fx.play_clip("clare", FightClip::Slash);
        assert_eq!(fx.clip_of("clare").0, FightClip::Slash);
        fx.time = 1.0;
        fx.clips.tick(fx.time);
        assert_eq!(fx.clip_of("clare").0, FightClip::Idle);
    }
}
