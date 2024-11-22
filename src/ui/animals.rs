use super::{EguiTextures, OccupiedScreenSpace, WindowsOpenState};
use crate::{planet::*, screen::CursorMode, text::WithUnitDisplay};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use compact_str::{format_compact, CompactString};

#[derive(Debug)]
pub struct State {
    ordered_ids: Vec<CompactString>,
    current: CompactString,
}

pub fn animals_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut _planet: ResMut<Planet>,
    mut cursor_mode: ResMut<CursorMode>,
    params: Res<Params>,
    textures: Res<EguiTextures>,
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
                    contents(ui, state, &params, &textures, &mut cursor_mode);
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
    params: &Params,
    textures: &EguiTextures,
    cursor_mode: &mut CursorMode,
) {
    ui.horizontal(|ui| {
        ui.add(
            egui::Image::new(textures.get(format_compact!("animals/{}", state.current)))
                .shrink_to_fit(),
        );
        ui.heading(t!(state.current));
    });

    let attr = params.animals.get(&state.current).unwrap();

    egui::Grid::new("table_atmo").striped(true).show(ui, |ui| {
        ui.label(t!("cost"));
        ui.horizontal(|ui| {
            ui.label(WithUnitDisplay::GenePoint(attr.cost).to_string());
            ui.add(egui::Image::new(textures.get("ui/icon-gene")).shrink_to_fit());
        });
        ui.end_row();

        ui.label(t!("livable-temperature"));
        ui.label(format!(
            "{}°C - {}°C",
            attr.temp.0 - KELVIN_CELSIUS,
            attr.temp.1 - KELVIN_CELSIUS
        ));
        ui.end_row();
    });

    ui.separator();
    if ui.button(t!("spawn")).clicked() {
        *cursor_mode = CursorMode::SpawnAnimal(state.current.clone());
    }
}

fn select_panel(ui: &mut egui::Ui, state: &mut State) {
    egui::ScrollArea::vertical()
        .min_scrolled_height(200.0)
        .show(ui, |ui| {
            ui.set_min_width(80.0);
            ui.set_min_height(180.0);
            ui.vertical(|ui| {
                for id in &state.ordered_ids {
                    ui.selectable_value(&mut state.current, id.clone(), t!(id));
                }
            });
        });
}

impl State {
    fn new(params: &Params) -> Self {
        let mut ids: Vec<_> = params.animals.keys().cloned().collect();
        ids.sort_unstable();
        let current = ids[0].clone();
        Self {
            ordered_ids: ids,
            current,
        }
    }
}
