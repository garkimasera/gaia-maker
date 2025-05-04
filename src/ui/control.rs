use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use crate::{
    planet::{
        AnimalId, BuildingControlValue, Planet, Requirement, SpaceBuildingKind, StructureKind,
    },
    screen::OccupiedScreenSpace,
};

use super::{UiTextures, WindowsOpenState};

const SLIDER_WIDTH: f32 = 250.0;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, AsRefStr, EnumIter)]
#[strum(serialize_all = "kebab-case")]
pub enum Panel {
    #[default]
    Planet,
    Civilization,
}

pub fn control_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut planet: ResMut<Planet>,
    textures: Res<UiTextures>,
    mut current_panel: Local<Panel>,
    mut current_civ_id: Local<Option<AnimalId>>,
) {
    if !wos.control {
        return;
    }

    let rect = egui::Window::new("control-window")
        .anchor(
            egui::Align2::LEFT_TOP,
            [
                occupied_screen_space.tools_expander_width,
                occupied_screen_space.toolbar_height,
            ],
        )
        .title_bar(false)
        .vscroll(true)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {
                    wos.control = false;
                }
                ui.heading(t!("control"));
            });
            ui.separator();

            ui.horizontal(|ui| {
                for panel in Panel::iter() {
                    ui.selectable_value(&mut *current_panel, panel, t!(panel));
                }
            });
            ui.separator();

            match *current_panel {
                Panel::Planet => planet_control(ui, &textures, &mut planet),
                Panel::Civilization => civ_control(ui, &textures, &planet, &mut current_civ_id),
            }
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn planet_control(ui: &mut egui::Ui, textures: &UiTextures, planet: &mut Planet) {
    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
    ui.spacing_mut().slider_width = SLIDER_WIDTH;

    // Orbital mirror
    ui.horizontal(|ui| {
        ui.heading(t!("orbital-mirror"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/orbital-mirror"));
    });
    let building = planet.space_building_mut(SpaceBuildingKind::OrbitalMirror);
    if building.n > 0 {
        if let BuildingControlValue::IncreaseRate(rate) = &mut building.control {
            ui.add(egui::Slider::new(rate, -100..=100).suffix("%"));
        }
    } else {
        ui.horizontal(|ui| {
            ui.image(textures.get("ui/icon-cross"));
            ui.label(t!("msg/control-need-orbital-mirror"));
        });
    }
    ui.separator();

    // Forestation speed
    ui.horizontal(|ui| {
        ui.heading(t!("forestation-speed"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/forestation-speed"));
    });
    let requirement = Requirement::StructureBuilt {
        kind: StructureKind::FertilizationPlant,
        n: 1,
    };
    if requirement.check(planet) {
        ui.add(egui::Slider::new(&mut planet.state.forestation_speed, 0..=200).suffix("%"));
    } else {
        ui.horizontal(|ui| {
            ui.image(textures.get("ui/icon-cross"));
            ui.label(t!("msg/control-need-fertilization-plant"));
        });
    }
}

fn civ_control(
    ui: &mut egui::Ui,
    _textures: &UiTextures,
    planet: &Planet,
    current_civ_id: &mut Option<AnimalId>,
) {
    if planet.civs.is_empty() {
        ui.label(t!("no-civilization"));
        *current_civ_id = None;
        return;
    }

    let civ_ids: Vec<AnimalId> = planet.civs.keys().copied().collect();
    if current_civ_id.is_none() {
        *current_civ_id = Some(civ_ids[0]);
    }
    let mut selected_civ_id = current_civ_id.unwrap();

    egui::ComboBox::from_id_salt("select-civilization")
        .selected_text(planet.civ_name(selected_civ_id))
        .show_ui(ui, |ui| {
            for &id in &civ_ids {
                ui.selectable_value(&mut selected_civ_id, id, planet.civ_name(id));
            }
        });

    *current_civ_id = Some(selected_civ_id);
}
