use bevy::prelude::*;
use geom::Coords;
use strum::{AsRefStr, EnumIter};

use crate::planet::Planet;

#[derive(Clone, Copy, Debug)]
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(prepare_color_materials);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, AsRefStr, EnumIter, Resource)]
#[strum(serialize_all = "kebab-case")]
pub enum OverlayLayerKind {
    #[default]
    None,
    AirTemprature,
    Rainfall,
    Height,
}

pub const N_POINTS: usize = 64;

#[derive(Resource)]
pub struct ColorMaterials {
    pub white_yellow_red: Vec<Handle<ColorMaterial>>,
    pub brown_white: Vec<Handle<ColorMaterial>>,
    pub blue_dark_blue: Vec<Handle<ColorMaterial>>,
}

impl ColorMaterials {
    pub fn get(&self, planet: &Planet, p: Coords, kind: OverlayLayerKind) -> Handle<ColorMaterial> {
        match kind {
            OverlayLayerKind::None => unreachable!(),
            OverlayLayerKind::AirTemprature => {
                let temp = planet.map[p].temp;

                let i = if temp < 263.15 {
                    0
                } else {
                    (((temp - 263.15) / (50.0 / N_POINTS as f32)) as usize).clamp(0, N_POINTS - 1)
                };

                self.white_yellow_red[i].clone()
            }
            OverlayLayerKind::Rainfall => {
                let rainfall = planet.map[p].rainfall;

                let i = if rainfall < 0.0 {
                    0
                } else {
                    ((rainfall / 40.0) as usize).clamp(0, N_POINTS - 1)
                };

                self.white_yellow_red[i].clone()
            }
            OverlayLayerKind::Height => {
                let h = planet.height_above_sea_level(p);

                if h > 0.0 {
                    let i = (h / 3000.0 * 64.0).clamp(0.0, N_POINTS as f32 - 1.0) as usize;
                    self.brown_white[i].clone()
                } else {
                    let i = (-h / 3000.0 * 64.0).clamp(0.0, N_POINTS as f32 - 1.0) as usize;
                    self.blue_dark_blue[i].clone()
                }
            }
        }
    }
}

fn prepare_color_materials(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    let white_yellow_red = (0..N_POINTS)
        .map(|i| {
            let color = if i < N_POINTS / 2 {
                Color::Rgba {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0 - (i as f32 / (N_POINTS / 2) as f32),
                    alpha: 0.4,
                }
            } else {
                Color::Rgba {
                    red: 1.0,
                    green: 1.0 - ((i - N_POINTS / 2) as f32 / (N_POINTS / 2) as f32),
                    blue: 0.0,
                    alpha: 0.4,
                }
            };

            materials.add(ColorMaterial {
                color,
                texture: None,
            })
        })
        .collect::<Vec<_>>();

    let brown_white = GRAD_BROWN_WIHTE
        .into_iter()
        .map(|[r, g, b]| {
            let color = Color::Rgba {
                red: r as f32 / 256.0,
                green: g as f32 / 256.0,
                blue: b as f32 / 256.0,
                alpha: 0.4,
            };
            materials.add(ColorMaterial {
                color,
                texture: None,
            })
        })
        .collect::<Vec<_>>();

    let blue_dark_blue = BLUE_DARK_BLUE
        .into_iter()
        .map(|[r, g, b]| {
            let color = Color::Rgba {
                red: r as f32 / 256.0,
                green: g as f32 / 256.0,
                blue: b as f32 / 256.0,
                alpha: 0.4,
            };
            materials.add(ColorMaterial {
                color,
                texture: None,
            })
        })
        .collect::<Vec<_>>();

    let color_materials = ColorMaterials {
        white_yellow_red,
        brown_white,
        blue_dark_blue,
    };
    commands.insert_resource(color_materials);
}

const GRAD_BROWN_WIHTE: [[u8; 3]; N_POINTS] = [
    [104, 13, 13],
    [108, 32, 32],
    [110, 43, 44],
    [113, 53, 52],
    [116, 60, 61],
    [119, 67, 68],
    [121, 73, 73],
    [124, 79, 79],
    [126, 84, 84],
    [129, 89, 88],
    [132, 93, 92],
    [134, 97, 97],
    [136, 101, 100],
    [138, 105, 105],
    [140, 109, 109],
    [143, 112, 112],
    [145, 115, 116],
    [147, 119, 118],
    [149, 122, 121],
    [151, 124, 125],
    [153, 127, 127],
    [154, 131, 130],
    [156, 133, 133],
    [158, 136, 136],
    [160, 138, 139],
    [161, 141, 141],
    [163, 144, 143],
    [165, 146, 146],
    [167, 148, 148],
    [169, 151, 150],
    [171, 153, 153],
    [172, 155, 155],
    [174, 158, 158],
    [175, 160, 160],
    [177, 162, 162],
    [178, 164, 164],
    [180, 166, 166],
    [182, 168, 168],
    [183, 170, 170],
    [185, 172, 172],
    [186, 174, 174],
    [187, 176, 176],
    [189, 178, 178],
    [190, 180, 180],
    [191, 181, 181],
    [194, 184, 183],
    [195, 185, 186],
    [197, 187, 187],
    [197, 189, 188],
    [199, 190, 190],
    [201, 192, 193],
    [201, 194, 194],
    [203, 196, 196],
    [204, 197, 198],
    [205, 199, 199],
    [207, 201, 201],
    [209, 202, 202],
    [210, 203, 204],
    [211, 205, 205],
    [212, 207, 207],
    [213, 208, 208],
    [214, 210, 210],
    [215, 211, 211],
    [217, 212, 213],
];

const BLUE_DARK_BLUE: [[u8; 3]; N_POINTS] = [
    [72, 138, 206],
    [72, 136, 205],
    [71, 135, 202],
    [70, 134, 201],
    [70, 133, 199],
    [69, 131, 197],
    [69, 130, 196],
    [68, 129, 194],
    [67, 128, 192],
    [66, 127, 190],
    [66, 126, 188],
    [66, 125, 187],
    [65, 123, 185],
    [64, 122, 183],
    [64, 121, 182],
    [63, 119, 180],
    [62, 118, 178],
    [62, 117, 176],
    [61, 116, 174],
    [61, 115, 173],
    [60, 114, 171],
    [59, 112, 169],
    [58, 111, 167],
    [58, 110, 165],
    [57, 109, 164],
    [57, 108, 161],
    [56, 106, 160],
    [55, 105, 158],
    [55, 104, 156],
    [55, 103, 154],
    [54, 102, 153],
    [53, 100, 151],
    [52, 99, 149],
    [52, 98, 147],
    [51, 96, 145],
    [50, 95, 143],
    [50, 94, 142],
    [49, 93, 140],
    [48, 92, 138],
    [47, 91, 136],
    [48, 90, 134],
    [47, 88, 132],
    [46, 87, 131],
    [45, 86, 129],
    [45, 85, 127],
    [44, 84, 126],
    [43, 82, 124],
    [43, 81, 122],
    [42, 80, 120],
    [41, 79, 118],
    [41, 78, 116],
    [40, 76, 115],
    [40, 75, 112],
    [39, 74, 112],
    [39, 73, 109],
    [38, 72, 107],
    [38, 70, 105],
    [36, 69, 104],
    [36, 68, 102],
    [35, 67, 100],
    [35, 65, 99],
    [34, 64, 96],
    [33, 63, 95],
    [33, 63, 94],
];
