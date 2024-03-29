use super::{convert_rect, CursorMode, OccupiedScreenSpace, WindowsOpenState};
use crate::planet::*;
use crate::{conf::Conf, sim::DebugTools};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    TileInfo,
    Sim,
    Map,
    Planet,
    Atmosphere,
    Water,
}

pub fn debug_tools_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut planet: ResMut<Planet>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut debug_tools: ResMut<DebugTools>,
    conf: Res<Conf>,
    mut current_panel: Local<Panel>,
    mut map_panel: Local<MapPanel>,
) {
    if !wos.debug_tools {
        return;
    }

    let rect = egui::Window::new("Debug Tools")
        .open(&mut wos.debug_tools)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, panel.as_ref());
                }
            });
            ui.separator();

            match *current_panel {
                Panel::TileInfo => info_ui(ui),
                Panel::Sim => sim_ui(ui, &mut debug_tools),
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
        .push(convert_rect(rect, conf.ui.scale_factor));
}

fn info_ui(ui: &mut egui::Ui) {
    let tile_logs = crate::planet::debug_log::tile_logs();

    egui::Grid::new("star_system_buildings")
        .striped(true)
        .show(ui, |ui| {
            for (name, data) in tile_logs.iter() {
                ui.label(*name);
                ui.label(data);
                ui.end_row();
            }
        });
}

fn sim_ui(ui: &mut egui::Ui, debug_tools: &mut DebugTools) {
    ui.checkbox(&mut debug_tools.sim_every_frame, "sim every frame");
}

#[derive(Default, Debug)]
pub struct MapPanel {
    biome: Biome,
    settlement_age: CivilizationAge,
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
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_source(CivilizationAge::default())
                .selected_text(AsRef::<str>::as_ref(&self.settlement_age))
                .show_ui(ui, |ui| {
                    for age in CivilizationAge::iter() {
                        ui.selectable_value(
                            &mut self.settlement_age,
                            age,
                            AsRef::<str>::as_ref(&age),
                        );
                    }
                });
            if ui.button("Place settlement").clicked()
                || matches!(*cursor_mode, CursorMode::PlaceSettlement(_))
            {
                *cursor_mode = CursorMode::PlaceSettlement(Settlement {
                    age: self.settlement_age,
                });
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
        let mut value = planet.atmo.mass(gas_kind);
        ui.add(
            egui::Slider::new(&mut value, 1.0e+5..=1.0e+11)
                .text(t!(gas_kind.as_ref()))
                .logarithmic(true),
        );
        planet.atmo.set_mass(gas_kind, value);
    }
}

fn water_ui(ui: &mut egui::Ui, planet: &mut Planet) {
    ui.add(egui::Slider::new(&mut planet.water.water_volume, 0.0..=1.0e+18).text("water volume"));
}
