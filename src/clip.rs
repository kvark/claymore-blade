//! Short combat body clips. Presentation only.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FightClip {
    Idle,
    Ready,
    Slash,
    Lunge,
    Guard,
    Raise,
    Hurt,
}

impl FightClip {
    pub fn duration(self) -> f32 {
        match self {
            Self::Idle => 0.0,
            Self::Ready => 0.55,
            Self::Slash => 0.42,
            Self::Lunge => 0.36,
            Self::Guard => 0.50,
            Self::Raise => 0.55,
            Self::Hurt => 0.34,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ClipPlay {
    pub unit: String,
    pub clip: FightClip,
    pub t0: f32,
    pub dur: f32,
}

#[derive(Clone, Debug, Default)]
pub struct ClipBank {
    pub plays: Vec<ClipPlay>,
}

impl ClipBank {
    pub fn play(&mut self, unit: &str, clip: FightClip, now: f32) {
        if unit.is_empty() || clip == FightClip::Idle {
            return;
        }
        self.plays.retain(|c| c.unit != unit);
        self.plays.push(ClipPlay {
            unit: unit.to_string(),
            clip,
            t0: now,
            dur: clip.duration(),
        });
    }

    pub fn tick(&mut self, now: f32) {
        self.plays.retain(|c| c.dur <= 0.0 || now - c.t0 < c.dur);
    }

    pub fn of(&self, unit: &str, now: f32) -> (FightClip, f32) {
        let Some(c) = self.plays.iter().find(|c| c.unit == unit) else {
            return (FightClip::Idle, 0.0);
        };
        if c.dur <= 0.0 {
            return (c.clip, 0.0);
        }
        let u = ((now - c.t0) / c.dur).clamp(0.0, 1.0);
        (c.clip, u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_progresses() {
        let mut bank = ClipBank::default();
        bank.play("clare", FightClip::Slash, 0.0);
        let (c, u) = bank.of("clare", 0.0);
        assert_eq!(c, FightClip::Slash);
        assert!(u < 0.01);
        let (c, u) = bank.of("clare", 0.2);
        assert_eq!(c, FightClip::Slash);
        assert!(u > 0.3 && u < 0.7);
        bank.tick(1.0);
        let (c, _) = bank.of("clare", 1.0);
        assert_eq!(c, FightClip::Idle);
    }
}
