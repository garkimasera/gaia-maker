use rand::Rng;

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
