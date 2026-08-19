//! Pointy-top axial hexes.

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Axial {
    pub q: i32,
    pub r: i32,
}

pub const HEX_DIRS: [Axial; 6] = [
    Axial { q: 1, r: 0 },
    Axial { q: 1, r: -1 },
    Axial { q: 0, r: -1 },
    Axial { q: -1, r: 0 },
    Axial { q: -1, r: 1 },
    Axial { q: 0, r: 1 },
];

impl Axial {
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    pub fn key(self) -> (i32, i32) {
        (self.q, self.r)
    }

    pub fn add(self, o: Self) -> Self {
        Self {
            q: self.q + o.q,
            r: self.r + o.r,
        }
    }

    pub fn sub(self, o: Self) -> Self {
        Self {
            q: self.q - o.q,
            r: self.r - o.r,
        }
    }
}

pub fn hex_eq(a: Axial, b: Axial) -> bool {
    a.q == b.q && a.r == b.r
}

pub fn rotate60(h: Axial, times: i32) -> Axial {
    let mut q = h.q;
    let mut r = h.r;
    let t = ((times % 6) + 6) % 6;
    for _ in 0..t {
        let nq = -r;
        let nr = q + r;
        q = nq;
        r = nr;
    }
    Axial { q, r }
}

pub fn cube_round(qf: f32, rf: f32) -> Axial {
    let sf = -qf - rf;
    let mut q = qf.round();
    let mut r = rf.round();
    let s = sf.round();
    let qd = (q - qf).abs();
    let rd = (r - rf).abs();
    let sd = (s - sf).abs();
    if qd > rd && qd > sd {
        q = -r - s;
    } else if rd > sd {
        r = -q - s;
    }
    Axial {
        q: q as i32,
        r: r as i32,
    }
}

pub fn axial_to_world(h: Axial, size: f32) -> (f32, f32) {
    let x = size * 3f32.sqrt() * (h.q as f32 + h.r as f32 / 2.0);
    let z = size * 1.5 * h.r as f32;
    (x, z)
}

pub fn world_to_axial(x: f32, z: f32, size: f32) -> Axial {
    let q = ((3f32.sqrt() / 3.0) * x - (1.0 / 3.0) * z) / size;
    let r = ((2.0 / 3.0) * z) / size;
    cube_round(q, r)
}

pub fn hex_distance(a: Axial, b: Axial) -> i32 {
    (i32::abs(a.q - b.q) + i32::abs(a.q + a.r - b.q - b.r) + i32::abs(a.r - b.r)) / 2
}

pub fn hex_neighbors(h: Axial) -> [Axial; 6] {
    HEX_DIRS.map(|d| h.add(d))
}

pub fn hex_disc(center: Axial, radius: i32) -> Vec<Axial> {
    let mut out = Vec::new();
    for q in -radius..=radius {
        let r_min = i32::max(-radius, -q - radius);
        let r_max = i32::min(radius, -q + radius);
        for r in r_min..=r_max {
            out.push(Axial {
                q: center.q + q,
                r: center.r + r,
            });
        }
    }
    out
}

pub fn hex_ring(center: Axial, radius: i32) -> Vec<Axial> {
    if radius <= 0 {
        return vec![center];
    }
    hex_disc(center, radius)
        .into_iter()
        .filter(|h| hex_distance(center, *h) == radius)
        .collect()
}

pub fn hex_line(a: Axial, b: Axial) -> Vec<Axial> {
    let n = hex_distance(a, b);
    if n == 0 {
        return vec![a];
    }
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            cube_round(
                a.q as f32 + (b.q - a.q) as f32 * t,
                a.r as f32 + (b.r - a.r) as f32 * t,
            )
        })
        .collect()
}

pub fn hex_cone(origin: Axial, facing: i32, range: i32) -> Vec<Axial> {
    let fwd = HEX_DIRS[(((facing % 6) + 6) % 6) as usize];
    let fq = fwd.q;
    let fr = fwd.r;
    let fs = -fq - fr;
    hex_disc(origin, range)
        .into_iter()
        .filter(|h| {
            let d = hex_distance(origin, *h);
            if d == 0 {
                return false;
            }
            let dq = h.q - origin.q;
            let dr = h.r - origin.r;
            let ds = -dq - dr;
            let dot = fq * dq + fr * dr + fs * ds;
            dot as f32 >= d as f32 - 0.5
        })
        .collect()
}

pub fn hex_sweep(origin: Axial, facing: i32) -> Vec<Axial> {
    let f = ((facing % 6) + 6) % 6;
    let dirs = [
        HEX_DIRS[((f + 5) % 6) as usize],
        HEX_DIRS[f as usize],
        HEX_DIRS[((f + 1) % 6) as usize],
    ];
    dirs.into_iter().map(|d| origin.add(d)).collect()
}

pub fn facing_toward(from: Axial, to: Axial) -> i32 {
    let dq = to.q - from.q;
    let dr = to.r - from.r;
    if dq == 0 && dr == 0 {
        return 0;
    }
    let mut best = 0;
    let mut best_dot = i32::MIN;
    for (i, d) in HEX_DIRS.iter().enumerate() {
        let ds = -d.q - d.r;
        let hs = -dq - dr;
        let dot = d.q * dq + d.r * dr + ds * hs;
        if dot > best_dot {
            best_dot = dot;
            best = i as i32;
        }
    }
    best
}

pub fn place_footprint(shape: &[Axial], origin: Axial, facing: i32) -> Vec<Axial> {
    shape
        .iter()
        .map(|h| origin.add(rotate60(*h, facing)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_neighbors() {
        let o = Axial::new(3, 4);
        for n in hex_neighbors(o) {
            assert_eq!(hex_distance(o, n), 1);
        }
        assert_eq!(hex_distance(o, o), 0);
    }

    #[test]
    fn world_roundtrip() {
        let h = Axial::new(4, 2);
        let (x, z) = axial_to_world(h, 1.0);
        assert_eq!(world_to_axial(x, z, 1.0), h);
    }

    #[test]
    fn rotate_full_turn() {
        let h = Axial::new(2, -1);
        assert_eq!(rotate60(h, 6), h);
    }

    #[test]
    fn sweep_is_three() {
        assert_eq!(hex_sweep(Axial::new(0, 0), 0).len(), 3);
    }
}
