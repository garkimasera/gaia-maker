use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};

use crate::{
    audio::SoundEffectPlayer,
    planet::{
        AnimalId, BuildingControlValue, Params, Planet, Requirement, SpaceBuildingKind,
        StructureKind,
    },
    screen::OccupiedScreenSpace,
};

use super::{HELP_TOOLTIP_WIDTH, UiTextures, WindowsOpenState, help::HelpItem};

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
    (mut planet, params): (ResMut<Planet>, Res<Params>),
    textures: Res<UiTextures>,
    mut current_panel: Local<Panel>,
    mut current_civ_id: Local<Option<AnimalId>>,
    se_player: SoundEffectPlayer,
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
                    if ui
                        .selectable_value(&mut *current_panel, panel, t!(panel))
                        .clicked()
                    {
                        se_player.play("select-item");
                    }
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .auto_shrink(egui::Vec2b::new(false, false))
                .show(ui, |ui| match *current_panel {
                    Panel::Planet => planet_control(ui, &textures, &mut planet, &se_player),
                    Panel::Civilization => civ_control(
                        ui,
                        &textures,
                        &mut planet,
                        &params,
                        &mut current_civ_id,
                        &se_player,
                    ),
                });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn planet_control(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    planet: &mut Planet,
    se_player: &SoundEffectPlayer,
) {
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
            if ui
                .add(egui::Slider::new(rate, -100..=100).suffix("%"))
                .changed()
            {
                se_player.play_if_stopped("slider");
            }
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
        if ui
            .add(egui::Slider::new(&mut planet.state.forestation_speed, 0..=100).suffix("%"))
            .changed()
        {
            se_player.play_if_stopped("slider");
        }
    } else {
        ui.horizontal(|ui| {
            ui.image(textures.get("ui/icon-cross"));
            ui.label(t!("msg/control-need-fertilization-plant"));
        });
    }

    // Animal Evolution
    ui.horizontal(|ui| {
        ui.heading(t!("animal-evolution"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/animal-evolution"));
    });
    if ui
        .add(egui::Slider::new(&mut planet.state.animal_evolution, 0..=200).suffix("%"))
        .changed()
    {
        se_player.play_if_stopped("slider");
    }

    // Civilization Probability
    ui.horizontal(|ui| {
        ui.heading(t!("civ-probability"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/civ-probability"));
    });
    if ui
        .add(egui::Slider::new(&mut planet.state.civ_prob, 0..=200).suffix("%"))
        .changed()
    {
        se_player.play_if_stopped("slider");
    }
}

fn civ_control(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    planet: &mut Planet,
    params: &Params,
    current_civ_id: &mut Option<AnimalId>,
    se_player: &SoundEffectPlayer,
) {
    let x_tooltip = ui.response().rect.right_top().x + 3.0;

    if planet.civs.is_empty() {
        ui.label(t!("no-civilization"));
        *current_civ_id = None;
        return;
    }
    ui.spacing_mut().slider_width = SLIDER_WIDTH;

    let civ_ids: Vec<AnimalId> = planet.civs.keys().copied().collect();
    if current_civ_id.is_none() {
        *current_civ_id = Some(civ_ids[0]);
    }
    let mut selected_civ_id = current_civ_id.unwrap();

    let response = egui::ComboBox::from_id_salt("select-civilization")
        .selected_text(planet.civ_name(selected_civ_id))
        .show_ui(ui, |ui| {
            for &id in &civ_ids {
                if ui
                    .selectable_value(&mut selected_civ_id, id, planet.civ_name(id))
                    .clicked()
                {
                    se_player.play("select-item");
                }
            }
        })
        .response;
    if response.clicked() {
        se_player.play("select-item");
    }

    *current_civ_id = Some(selected_civ_id);
    let Some(c) = planet.civs.get_mut(&selected_civ_id) else {
        *current_civ_id = None;
        return;
    };
    let civ_control = &mut c.civ_control;

    ui.separator();

    // Population growth
    ui.horizontal(|ui| {
        ui.heading(t!("population-growth"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/population-growth"));
    });
    if ui
        .add(egui::Slider::new(&mut civ_control.pop_growth, 0..=200).suffix("%"))
        .changed()
    {
        se_player.play_if_stopped("slider");
    }
    ui.separator();

    // Technology development
    ui.horizontal(|ui| {
        ui.heading(t!("technology-development"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/technology-development"));
    });
    if ui
        .add(egui::Slider::new(&mut civ_control.tech_development, 0..=200).suffix("%"))
        .changed()
    {
        se_player.play_if_stopped("slider");
    }
    ui.separator();

    // Aggressiveness
    ui.horizontal(|ui| {
        ui.heading(t!("aggressiveness"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/aggressiveness"));
    });
    if ui
        .add(egui::Slider::new(&mut civ_control.aggressiveness, 0..=200).suffix("%"))
        .changed()
    {
        se_player.play_if_stopped("slider");
    }
    ui.separator();

    // Energy source weight
    ui.horizontal(|ui| {
        ui.heading(t!("energy-source-weight"));
        ui.image(textures.get("ui/icon-help"))
            .on_hover_text(t!("help/control/energy-source-weight"));
    });
    for (energy_source, weight) in &mut civ_control.energy_weight {
        ui.horizontal(|ui| {
            ui.image(textures.get(format!("ui/icon-energy-source-{}", energy_source.as_ref())))
                .on_hover_text(t!("energy_source", energy_source));
            let response = ui.add(egui::Slider::new(weight, 0..=100).suffix("%"));
            if response.changed() {
                se_player.play_if_stopped("slider");
            }
            if response.hovered() {
                let help_item = HelpItem::EnergySources(*energy_source);
                let pos = egui::pos2(x_tooltip, response.rect.right_top().y);

                egui::containers::show_tooltip_at(
                    &response.ctx,
                    response.layer_id,
                    response.id,
                    pos,
                    |ui| {
                        ui.set_max_width(HELP_TOOLTIP_WIDTH);
                        ui.strong(t!("energy_source", energy_source));
                        ui.separator();
                        help_item.ui(ui, textures, params);
                    },
                );
            }
        });
    }
}
