use super::{CursorMode, OccupiedScreenSpace, WindowsOpenState};
use crate::planet::*;
use crate::saveload::SaveState;
use crate::screen::HoverTile;
use crate::{planet::debug::PlanetDebug, screen::CauseEventKind};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use geom::Coords;
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    TileInfo,
    Sim,
    Map,
    Planet,
    Atmo,
}

pub fn debug_tools_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut planet: ResMut<Planet>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    mut save_state: ResMut<SaveState>,
    params: Res<Params>,
    sim: Res<Sim>,
    hover_tile: Query<&HoverTile>,
    (mut current_panel, mut map_panel, mut last_hover_tile): (
        Local<Panel>,
        Local<MapPanel>,
        Local<Option<Coords>>,
    ),
) {
    if !wos.debug_tools {
        return;
    }

    if !save_state.save_file_metadata.debug_mode_enabled {
        egui::Modal::new("use-debug-mode".into()).show(egui_ctxs.ctx_mut(), |ui| {
            ui.label("Enable Debug Mode?");
            ui.strong("Debug mode flag is saved in the save data.");
            ui.strong("Use of debug mode is at your own risk.");
            if ui.button("Yes").clicked() {
                save_state.save_file_metadata.debug_mode_enabled = true;
            }
            if ui.button("No").clicked() {
                wos.debug_tools = false;
            }
        });
        return;
    }

    // Information about the hovered tile
    let hover_tile = hover_tile.single();
    last_hover_tile.get_or_insert(Coords(0, 0));
    if hover_tile.0.is_some() {
        *last_hover_tile = hover_tile.0;
    }

    let p = hover_tile.0.unwrap_or(last_hover_tile.unwrap());

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
                Panel::TileInfo => info_ui(ui, &planet, &sim, p),
                Panel::Sim => sim_ui(ui, &mut planet),
                Panel::Map => map_panel.ui(ui, &mut planet, &mut cursor_mode, &params),
                Panel::Planet => planet_ui(ui, &mut planet),
                Panel::Atmo => atmo_ui(ui, &mut planet),
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn info_ui(ui: &mut egui::Ui, planet: &Planet, sim: &Sim, p: Coords) {
    let tile_debug_info = crate::planet::debug::tile_debug_info(planet, sim, p);
    let tile_logs = crate::planet::debug::tile_logs();

    egui::Grid::new("tile_info_grid").striped(true).show(ui, |ui| {
        for (name, data) in tile_debug_info.iter() {
            ui.label(*name);
            ui.label(data);
            ui.end_row();
        }
        ui.separator();
        ui.separator();
        ui.end_row();
        for (name, data) in tile_logs.iter() {
            ui.label(*name);
            ui.label(data);
            ui.end_row();
        }
    });
}

fn sim_ui(ui: &mut egui::Ui, planet: &mut Planet) {
    if ui.button("max resources").clicked() {
        planet.res.debug_max();
    }
}

#[derive(Default, Debug)]
pub struct MapPanel {
    biome: Biome,
    settlement_age: CivilizationAge,
    animal_id: Option<AnimalId>,
    cause_event: CauseEventKind,
}

impl MapPanel {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        planet: &mut Planet,
        cursor_mode: &mut CursorMode,
        params: &Params,
    ) {
        let default_civ_animal = *params
            .animals
            .iter()
            .find(|(_, attr)| attr.civ.is_some())
            .unwrap()
            .0;
        if self.animal_id.is_none() {
            self.animal_id = Some(default_civ_animal);
        }

        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("debug_tool_biomes")
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
            egui::ComboBox::from_id_salt("debug_tool_civ_ages")
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
            egui::ComboBox::from_id_salt("debug_tool_civ_animals")
                .selected_text(&*self.animal_id.unwrap())
                .show_ui(ui, |ui| {
                    for animal_id in params
                        .animals
                        .iter()
                        .filter_map(|(id, attr)| attr.civ.as_ref().map(|_| id))
                    {
                        ui.selectable_value(
                            self.animal_id.as_mut().unwrap(),
                            *animal_id,
                            AsRef::<str>::as_ref(&animal_id),
                        );
                    }
                });
            if ui.button("Place settlement").clicked()
                || matches!(*cursor_mode, CursorMode::PlaceSettlement(_, _))
            {
                *cursor_mode =
                    CursorMode::PlaceSettlement(self.animal_id.unwrap(), self.settlement_age);
            }
        });
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("debug_tool_cause_event")
                .selected_text(AsRef::<str>::as_ref(&self.cause_event))
                .show_ui(ui, |ui| {
                    for ce in CauseEventKind::iter() {
                        ui.selectable_value(&mut self.cause_event, ce, AsRef::<str>::as_ref(&ce));
                    }
                });
            if ui.button("Cause event").clicked()
                || matches!(*cursor_mode, CursorMode::CauseEvent(_))
            {
                *cursor_mode = CursorMode::CauseEvent(self.cause_event);
            }
        });

        ui.horizontal(|ui| {
            if ui.button("height +100").clicked() {
                *cursor_mode = CursorMode::ChangeHeight(100.0);
            }
            if ui.button("height -100").clicked() {
                *cursor_mode = CursorMode::ChangeHeight(-100.0);
            }
        });

        ui.separator();

        if ui.button("delete all civilization").clicked() {
            planet.delete_civilization();
        }

        if ui.button("delete all animals").clicked() {
            planet.delete_animals();
        }

        ui.separator();
        if ui.button("copy height map").clicked() {
            let s = planet.height_map_as_string();
            ui.output_mut(|o| o.commands.push(egui::OutputCommand::CopyText(s)));
        }
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
                .text(t!(gas_kind))
                .logarithmic(true),
        );
        planet.atmo.set_mass(gas_kind, value);
    }
    ui.label(format!("cloud {:.2}", planet.atmo.cloud_amount));
    ui.add(egui::Slider::new(&mut planet.atmo.aerosol, 0.0..=100.0).text("aerosol"));
    ui.separator();
    ui.add(egui::Slider::new(&mut planet.water.water_volume, 0.0..=1.0e+18).text("water volume"));
}
