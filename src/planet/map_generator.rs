use geom::Array2d;
use noise::{NoiseFn, Perlin, ScalePoint};

#[derive(Clone, Debug)]
pub struct GenConf {
    pub w: u32,
    pub h: u32,
    pub max_height: f32,
}

pub fn generate(conf: GenConf) -> Array2d<f32> {
    let noise_fn = ScalePoint::new(Perlin::new(rand::random()))
        .set_x_scale(0.1)
        .set_y_scale(0.1);
    Array2d::from_fn(conf.w, conf.h, |(x, y)| {
        (noise_fn.get([x as f64, y as f64]) as f32 + 1.0) * conf.max_height * 0.5
    })
}
