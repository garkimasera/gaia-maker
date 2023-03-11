use super::{convert_rect, CursorMode, OccupiedScreenSpace, WindowsOpenState};
use crate::conf::Conf;
use crate::planet::*;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    Map,
    Planet,
    Atmosphere,
    Water,
}

pub fn edit_planet_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut planet: ResMut<Planet>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<Conf>,
    mut current_panel: Local<Panel>,
    mut map_panel: Local<MapPanel>,
) {
    if !wos.edit_planet {
        return;
    }

    let rect = egui::Window::new("Planet editing tools")
        .open(&mut wos.edit_planet)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, panel.as_ref());
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Map => map_panel.ui(ui, &mut cursor_mode),
                Panel::Planet => planet_ui(ui, &mut planet),
                Panel::Atmosphere => atmo_ui(ui, &mut planet),
                Panel::Water => water_ui(ui, &mut planet),
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space
        .window_rects
        .push(convert_rect(rect, conf.scale_factor));
}

#[derive(Default, Debug)]
pub struct MapPanel {
    biome: Biome,
}

impl MapPanel {
    fn ui(&mut self, ui: &mut egui::Ui, cursor_mode: &mut CursorMode) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source(Biome::Ocean)
                .selected_text(AsRef::<str>::as_ref(&self.biome))
                .show_ui(ui, |ui| {
                    for b in Biome::iter() {
                        ui.selectable_value(&mut self.biome, b, AsRef::<str>::as_ref(&b));
                    }
                });
            if ui.button("Edit biome").clicked() || matches!(*cursor_mode, CursorMode::EditBiome(_))
            {
                *cursor_mode = CursorMode::EditBiome(self.biome);
            }
        });
    }
}

fn planet_ui(ui: &mut egui::Ui, planet: &mut Planet) {
    ui.add(
        egui::Slider::new(&mut planet.basics.solar_constant, 0.0..=3000.0)
            .text(t!("solar-constant")),
    );
}

fn atmo_ui(ui: &mut egui::Ui, planet: &mut Planet) {
    for gas_kind in GasKind::iter() {
        ui.add(
            egui::Slider::new(
                planet.atmo.mass.get_mut(&gas_kind).unwrap(),
                1.0e+5..=1.0e+11,
            )
            .text(t!(gas_kind.as_ref()))
            .logarithmic(true),
        );
    }
}

fn water_ui(ui: &mut egui::Ui, planet: &mut Planet) {
    ui.add(egui::Slider::new(&mut planet.water.water_volume, 0.0..=1.0e+18).text("water volume"));
}
