use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub average_air_temp: f32,
    pub average_rainfall: f32,
}

impl Default for Stat {
    fn default() -> Self {
        Self {
            average_air_temp: 0.0,
            average_rainfall: 0.0,
        }
    }
}
