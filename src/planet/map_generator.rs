use geom::Array2d;
use noise::{
    utils::{NoiseMapBuilder, SphereMapBuilder},
    Perlin, ScalePoint,
};

#[derive(Clone, Debug)]
pub struct GenConf {
    pub w: u32,
    pub h: u32,
    pub max_height: f32,
    pub height_table: Vec<(f32, f32)>,
}

pub fn generate(conf: GenConf) -> Array2d<f32> {
    let noise_fn = ScalePoint::new(Perlin::new(rand::random())).set_scale(2.0);
    let map_builder = SphereMapBuilder::new(noise_fn)
        .set_size(conf.w as _, conf.h as _)
        .set_bounds(-80.0, 80.0, -180.0, 180.0)
        .build();

    Array2d::from_fn(conf.w, conf.h, |(x, y)| {
        let h = (map_builder.get_value(x as _, y as _) as f32 + 1.0) * 0.5; // Convert to 0.0..1.0

        let h = if conf.height_table.is_empty() {
            h
        } else {
            super::misc::linear_interpolation(&conf.height_table, h)
        };

        h * conf.max_height
    })
}
