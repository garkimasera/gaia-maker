use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::{
    assets::UiTexture,
    conf::Conf,
    planet::{Planet, Structure, KELVIN_CELSIUS},
    screen::{CursorMode, HoverTile, OccupiedScreenSpace},
    text::Unit,
    GameSpeed,
};

use super::{EguiTextures, WindowsOpenState};

pub fn panels(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    hover_tile: Query<&HoverTile>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut speed: ResMut<GameSpeed>,
    planet: Res<Planet>,
    textures: Res<EguiTextures>,
    conf: Res<Conf>,
) {
    occupied_screen_space.window_rects.clear();

    occupied_screen_space.occupied_left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            sidebar(
                ui,
                &cursor_mode,
                &planet,
                hover_tile.get_single().unwrap(),
                &textures,
                &conf,
            );
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width()
        * conf.scale_factor;

    occupied_screen_space.occupied_top = egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                toolbar(ui, &mut cursor_mode, &mut wos, &mut speed, &textures, &conf);
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height()
        * conf.scale_factor;
}

fn toolbar(
    ui: &mut egui::Ui,
    _cursor_mode: &mut CursorMode,
    wos: &mut WindowsOpenState,
    speed: &mut GameSpeed,
    textures: &EguiTextures,
    conf: &Conf,
) {
    let button = |ui: &mut egui::Ui, icon: UiTexture, s: &str| {
        let (handle, size) = &textures.0[&icon];
        ui.add(egui::ImageButton::new(handle.id(), conf.tex_size(*size)))
            .on_hover_text(t!(s))
            .clicked()
    };

    if button(ui, UiTexture::IconBuild, "build") {
        wos.build = !wos.build;
    }

    if button(ui, UiTexture::IconOrbit, "orbit") {
        wos.orbit = !wos.orbit;
    }

    if button(ui, UiTexture::IconStarSystem, "star-system") {
        wos.star_system = !wos.star_system;
    }

    if button(ui, UiTexture::IconLayers, "layer") {
        wos.layers = !wos.layers;
    }

    if button(ui, UiTexture::IconStat, "statistics") {
        wos.stat = !wos.stat;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    let texture = if *speed == GameSpeed::Paused {
        UiTexture::IconSpeedPausedSelected
    } else {
        UiTexture::IconSpeedPaused
    };
    if button(ui, texture, "speed-paused") {
        *speed = GameSpeed::Paused;
    }

    let texture = if *speed == GameSpeed::Normal {
        UiTexture::IconSpeedNormalSelected
    } else {
        UiTexture::IconSpeedNormal
    };
    if button(ui, texture, "speed-normal") {
        *speed = GameSpeed::Normal;
    }

    let texture = if *speed == GameSpeed::Fast {
        UiTexture::IconSpeedFastSelected
    } else {
        UiTexture::IconSpeedFast
    };
    if button(ui, texture, "speed-fast") {
        *speed = GameSpeed::Fast;
    }

    ui.add(egui::Separator::default().spacing(2.0).vertical());

    if button(ui, UiTexture::IconMessage, "messages") {
        wos.message = !wos.message;
    }

    if button(ui, UiTexture::IconGameMenu, "menu") {
        wos.game_menu = !wos.game_menu;
    }

    if button(ui, UiTexture::IconHelp, "help") {
        wos.help = !wos.help;
    }
}

fn sidebar(
    ui: &mut egui::Ui,
    cursor_mode: &CursorMode,
    planet: &Planet,
    hover_tile: &HoverTile,
    textures: &EguiTextures,
    _conf: &Conf,
) {
    let mut stock: Vec<_> = planet.res.stock.iter().collect();
    stock.sort_by_key(|&(res, _)| res);
    for (kind, v) in stock.into_iter() {
        ui.horizontal(|ui| {
            ui.label(&format!(
                "{}: {}",
                t!(kind.as_ref()),
                kind.display_with_value(*v)
            ));
            let diff = planet.res.diff[kind];
            let sign = if diff > 0.0 { '+' } else { '-' };
            ui.label(
                egui::RichText::new(format!("({}{})", sign, kind.display_with_value(diff.abs())))
                    .small(),
            );
        });
    }

    ui.separator();

    // Information about selected tool
    ui.label(t!("selected-tool"));
    match cursor_mode {
        CursorMode::Normal => {
            ui.label(t!("none"));
        }
        CursorMode::Build(kind) => {
            ui.label(t!(kind.as_ref()));
        }
        CursorMode::Demolition => {
            ui.label(t!("demolition"));
        }
        CursorMode::EditBiome(biome) => {
            ui.label(format!("biome editing: {}", biome.as_ref()));
        }
    }

    ui.separator();

    // Information about the hovered tile
    if let Some(p) = hover_tile.0 {
        let tile = &planet.map[p];

        let (texture, size) = &textures.0[&UiTexture::IconCoordinates];
        ui.horizontal(|ui| {
            ui.image(texture, *size).on_hover_text(t!("coordinates"));
            ui.label(format!("[{}, {}]", p.0, p.1))
                .on_hover_text(t!("coordinates"));

            let (longitude, latitude) = planet.calc_longitude_latitude(p);
            ui.label(format!(
                "{:.0}°, {:.0}°",
                longitude * 180.0 * std::f32::consts::FRAC_1_PI,
                latitude * 180.0 * std::f32::consts::FRAC_1_PI,
            ))
            .on_hover_text(format!("{}, {}", t!("longitude"), t!("latitude")));
        });

        let items: &[(UiTexture, String, &str)] = &[
            (
                UiTexture::IconHeight,
                format!("{:.0} m", planet.height_above_sea_level(p)),
                "height",
            ),
            (
                UiTexture::IconAirTemprature,
                format!("{:.1} °C", tile.temp - KELVIN_CELSIUS),
                "air-temprature",
            ),
            (
                UiTexture::IconRainfall,
                format!("{:.0} mm", tile.rainfall),
                "rainfall",
            ),
            (
                UiTexture::IconFertility,
                format!("{:.0} %", tile.fertility),
                "fertility",
            ),
            (
                UiTexture::IconBiomass,
                format!("{:.1} Mt", tile.biomass),
                "biomass",
            ),
        ];

        for (icon, label, s) in items {
            let (texture, size) = &textures.0[&icon];
            let s = t!(s);
            ui.horizontal(|ui| {
                ui.image(texture, *size).on_hover_text(&s);
                ui.label(label).on_hover_text(s);
            });
        }

        ui.separator();

        ui.label(t!(tile.biome.as_ref()));

        let s = match &tile.structure {
            Structure::None => None,
            Structure::Occupied { by } => {
                Some(crate::info::structure_info(&planet.map[*by].structure))
            }
            other => Some(crate::info::structure_info(other)),
        };

        if let Some(s) = s {
            ui.label(s);
        }
    }
}
