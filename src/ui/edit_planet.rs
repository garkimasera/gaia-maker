use super::{convert_rect, CursorMode, OccupiedScreenSpace, UiConf, WindowsOpenState};
use crate::planet::*;
use crate::sim::ManagePlanet;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    Map,
    Planet,
}

pub fn edit_planet_window(
    mut egui_ctx: ResMut<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut planet: ResMut<Planet>,
    mut cursor_mode: ResMut<CursorMode>,
    mut wos: ResMut<WindowsOpenState>,
    conf: Res<UiConf>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    mut current_panel: Local<Panel>,
    mut map_panel: Local<MapPanel>,
    mut planet_panel: Local<PlanetPanel>,
) {
    if !wos.edit_planet {
        return;
    }

    let rect = egui::Window::new("Planet editing tools")
        .open(&mut wos.edit_planet)
        .vscroll(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, panel.as_ref());
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Map => map_panel.ui(ui, &mut ew_manage_planet, &mut cursor_mode),
                Panel::Planet => planet_panel.ui(ui, &mut planet),
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
    new_w: u32,
    new_h: u32,
    biome: Biome,
}

impl MapPanel {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        ew_manage_planet: &mut EventWriter<ManagePlanet>,
        cursor_mode: &mut CursorMode,
    ) {
        ui.add(egui::Slider::new(&mut self.new_w, 2..=100).text("width"));
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut self.new_h, 2..=100).text("height"));
            if ui.button("New").clicked() {
                ew_manage_planet.send(ManagePlanet::New(self.new_w, self.new_h));
            }
        });

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

#[derive(Default, Debug)]
pub struct PlanetPanel;

impl PlanetPanel {
    fn ui(&mut self, ui: &mut egui::Ui, planet: &mut Planet) {
        ui.add(
            egui::Slider::new(&mut planet.basics.solar_constant, 0.0..=3000.0)
                .text(t!("solar-constant")),
        );
    }
}
