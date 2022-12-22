use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::conf::{Conf, ConfChange};
use crate::planet::Params;
use crate::sim::ManagePlanet;
use crate::text::Lang;
use strum::IntoEnumIterator;

pub fn main_menu(
    mut egui_ctx: ResMut<EguiContext>,
    mut ew_manage_planet: EventWriter<ManagePlanet>,
    params: Res<Params>,
    mut conf: ResMut<Conf>,
    mut ew_conf_change: EventWriter<ConfChange>,
) {
    egui::Window::new(t!("menu"))
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
        .default_width(0.0)
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                if ui.button(t!("new")).clicked() {
                    let size = params.start.default_size;
                    ew_manage_planet.send(ManagePlanet::New(size.0, size.1));
                }
                if ui.button(t!("load")).clicked() {
                    ew_manage_planet.send(ManagePlanet::Load("test.planet".into()));
                }

                ui.separator();

                if let Some(lang) = language_selector(ui, crate::text::get_lang()) {
                    conf.lang = lang;
                    crate::text::set_lang(lang);
                    ew_conf_change.send_default();
                }
            });
        })
        .unwrap();
}

fn language_selector(ui: &mut egui::Ui, before: Lang) -> Option<Lang> {
    let mut selected = before;
    egui::ComboBox::from_label("")
        .selected_text(selected.name())
        .show_ui(ui, |ui| {
            for lang in Lang::iter() {
                ui.selectable_value(&mut selected, lang, lang.name());
            }
        });

    if selected != before {
        Some(selected)
    } else {
        None
    }
}
