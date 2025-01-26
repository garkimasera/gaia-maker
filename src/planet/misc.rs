use geom::{Coords, CyclicMode};
use rand::{rngs::SmallRng, Rng, SeedableRng};

pub fn linear_interpolation(table: &[(f32, f32)], x: f32) -> f32 {
    assert!(table.len() > 2);
    let first = table.first().unwrap();
    let last = table.last().unwrap();
    if first.0 >= x {
        return first.1;
    } else if last.0 <= x {
        return last.1;
    }

    for i in 0..(table.len() - 1) {
        let x0 = table[i].0;
        let x1 = table[i + 1].0;
        if x0 < x && x <= x1 {
            let y0 = table[i].1;
            let y1 = table[i + 1].1;
            let a = (y1 - y0) / (x1 - x0);
            let b = (x1 * y0 - x0 * y1) / (x1 - x0);
            return a * x + b;
        }
    }

    panic!("invalid input for interpolation: {}", x)
}

pub fn bisection<F: FnMut(f32) -> f32>(
    mut f: F,
    mut a: f32,
    mut b: f32,
    n_max: usize,
    target_diff: f32,
) -> f32 {
    let mut c = (a + b) / 2.0;

    for _ in 0..n_max {
        if f(c) < 0.0 {
            a = c;
        } else {
            b = c;
        }
        c = (a + b) / 2.0;
        if (b - a) < target_diff * 2.0 {
            return c;
        }
    }
    c
}

pub fn range_to_livability_trapezoid((min, max): (f32, f32), a: f32, x: f32) -> f32 {
    let (min, max) = if min <= max {
        (min, max)
    } else {
        let a = (max + min) / 2.0;
        (a, a)
    };

    let l = max - min;
    let b = l / a;

    let result = if x <= min - b {
        0.0
    } else if x <= min + b {
        x / (2.0 * b) - min / (2.0 * b) + 0.5
    } else if x <= max - b {
        1.0
    } else if x <= max + b {
        -x / (2.0 * b) + max / (2.0 * b) + 0.5
    } else {
        0.0
    };

    debug_assert!(result.is_finite(), "{:?},{}", (min, max, a, x), result);
    // Clamp result because of float precision
    result.clamp(0.0, 1.0)
}

#[rustfmt::skip]
const CALC_CONGESTION_TARGET_TILES: &[((i32, i32), u32)] = &[
    ((-2, -2), 1), ((-1, -2), 1), ((0, -2), 1), ((1, -2), 1), ((2, -2), 1),
    ((-2, -1), 1), ((-1, -1), 2), ((0, -1), 2), ((1, -1), 2), ((2, -1), 1),
    ((-2,  0), 1), ((-1,  0), 2),               ((1,  0), 2), ((2,  0), 1),
    ((-2,  1), 1), ((-1,  1), 2), ((0,  1), 2), ((1,  1), 2), ((2,  1), 1),
    ((-2,  2), 1), ((-1,  2), 1), ((0,  2), 1), ((1,  2), 1), ((2,  2), 1),
];

pub fn calc_congestion_rate<F: FnMut(Coords) -> f32>(p: Coords, size: (u32, u32), mut f: F) -> f32 {
    let mut sum = 0;
    let mut crowded = 0.0;

    for &(dp, a) in CALC_CONGESTION_TARGET_TILES {
        let Some(p) = CyclicMode::X.convert_coords(size, p + dp) else {
            continue;
        };

        crowded += f(p) * a as f32;
        sum += a;
    }

    crowded / sum as f32
}

/// Random sampling [mean - d, mean + d] with constant distribution.
#[derive(Clone, Copy, Debug)]
pub struct ConstantDist {
    mean: f32,
    d: f32,
}

impl rand::distributions::Distribution<f32> for ConstantDist {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        rng.gen_range((self.mean - self.d)..=(self.mean - self.d))
    }
}

impl ConstantDist {
    pub fn new(mean: f32, d: f32) -> Self {
        Self { mean, d }
    }
}

impl From<(f32, f32)> for ConstantDist {
    fn from(a: (f32, f32)) -> Self {
        Self::new(a.0, a.1)
    }
}

/// Random sampling [mean - d, mean + d] with linear distribution.
#[derive(Clone, Copy, Debug)]
pub struct SymmetricalLinearDist {
    mean: f32,
    d: f32,
}

impl SymmetricalLinearDist {
    pub fn new(mean: f32, d: f32) -> Self {
        Self { mean, d }
    }
}

impl From<(f32, f32)> for SymmetricalLinearDist {
    fn from(a: (f32, f32)) -> Self {
        Self::new(a.0, a.1)
    }
}

pub fn get_rng() -> SmallRng {
    let mut thread_rng = rand::thread_rng();
    rand::rngs::SmallRng::from_rng(&mut thread_rng).unwrap()
}

impl rand::distributions::Distribution<f32> for SymmetricalLinearDist {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f32 {
        let r: f32 = rng.gen_range(0.0..=1.0);

        let x = if r < 0.5 {
            (2.0 * r).sqrt() - 1.0
        } else {
            -(2.0 - 2.0 * r).sqrt() + 1.0
        };

        self.mean + self.d * x
    }
}
