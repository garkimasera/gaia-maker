use super::{OccupiedScreenSpace, UiTextures, WindowsOpenState, misc::label_with_icon};
use crate::{planet::*, screen::CursorMode, text::WithUnitDisplay};
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use compact_str::format_compact;

#[derive(Debug)]
pub struct State {
    ordered_ids: Vec<AnimalId>,
    current: AnimalId,
}

pub fn animals_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut cursor_mode: ResMut<CursorMode>,
    mut planet: ResMut<Planet>,
    params: Res<Params>,
    textures: Res<UiTextures>,
    mut state: Local<Option<State>>,
) {
    if !wos.animals {
        return;
    }
    if state.is_none() {
        *state = Some(State::new(&params));
    }
    let state = state.as_mut().unwrap();

    let rect = egui::Window::new(t!("animal"))
        .open(&mut wos.animals)
        .show(egui_ctxs.ctx_mut(), |ui| {
            ui.horizontal(|ui| {
                select_panel(ui, state);
                ui.separator();
                ui.vertical(|ui| {
                    contents(ui, state, &mut planet, &params, &textures, &mut cursor_mode);
                });
            });
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn contents(
    ui: &mut egui::Ui,
    state: &State,
    planet: &mut Planet,
    params: &Params,
    textures: &UiTextures,
    cursor_mode: &mut CursorMode,
) {
    ui.horizontal(|ui| {
        ui.add(egui::Image::new(
            textures.get(format_compact!("animals/{}", state.current)),
        ));
        ui.heading(t!("animal", state.current));
    });

    let attr = params.animals.get(&state.current).unwrap();

    egui::Grid::new("table_atmo").striped(true).show(ui, |ui| {
        ui.label(t!("size"));
        ui.label(t!(attr.size));
        ui.end_row();

        ui.label(t!("cost"));
        label_with_icon(
            ui,
            textures,
            "ui/icon-gene",
            WithUnitDisplay::GenePoint(attr.cost).to_string(),
        );
        ui.end_row();

        ui.label(t!("habitat"));
        let s = match &attr.habitat {
            AnimalHabitat::Land => t!("land"),
            AnimalHabitat::Sea => t!("sea"),
            AnimalHabitat::Biomes(biomes) => biomes.iter().fold(String::new(), |s, biome| {
                if s.is_empty() {
                    t!(biome)
                } else {
                    format!("{}, {}", s, t!(biome))
                }
            }),
        };
        ui.label(s);
        ui.end_row();

        ui.label(t!("livable-temperature"));
        ui.label(format!(
            "{}°C - {}°C",
            attr.temp.0 - KELVIN_CELSIUS,
            attr.temp.1 - KELVIN_CELSIUS,
        ));
        ui.end_row();

        if let Some(civ) = &attr.civ {
            ui.label(t!("civilize-cost"));
            label_with_icon(
                ui,
                textures,
                "ui/icon-gene",
                WithUnitDisplay::GenePoint(civ.civilize_cost).to_string(),
            );
            ui.end_row();
        }
    });

    ui.separator();
    ui.horizontal(|ui| {
        if ui.button(t!("spawn")).clicked() {
            *cursor_mode = CursorMode::SpawnAnimal(state.current);
        }

        if attr.civ.is_some() {
            ui.scope(|ui| {
                if planet.events.in_progress_civilize_event(state.current) {
                    ui.disable();
                    let _ = ui.button(t!("civilizing-in-progress"));
                } else if planet.civs.contains_key(&state.current) {
                    ui.disable();
                    let _ = ui.button(t!("civilized"));
                } else {
                    let s = if let Err(s) = planet.can_civilize(state.current, params) {
                        ui.disable();
                        t!("msg", s)
                    } else {
                        String::new()
                    };
                    if ui.button(t!("civilize")).on_disabled_hover_text(s).clicked() {
                        planet.civilize_animal(state.current, params);
                    }
                }
            });
        }
    });
}

fn select_panel(ui: &mut egui::Ui, state: &mut State) {
    egui::ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            ui.set_min_width(80.0);
            ui.set_min_height(180.0);
            ui.vertical(|ui| {
                for id in &state.ordered_ids {
                    ui.selectable_value(&mut state.current, *id, t!("animal", id));
                }
            });
        });
}

impl State {
    fn new(params: &Params) -> Self {
        let mut ids: Vec<_> = params.animals.keys().cloned().collect();
        ids.sort_unstable();
        let current = ids[0];
        Self {
            ordered_ids: ids,
            current,
        }
    }
}
