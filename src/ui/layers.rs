use super::*;

pub fn layers_window(
    mut egui_ctxs: EguiContexts,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
    mut wos: ResMut<WindowsOpenState>,
    mut current_layer: ResMut<OverlayLayerKind>,
    mut update_draw: ResMut<UpdateDraw>,
    mut display_opts: ResMut<DisplayOpts>,
) {
    if !wos.layers {
        return;
    }

    let rect = egui::Window::new(t!("layers"))
        .open(&mut wos.layers)
        .vscroll(false)
        .default_width(100.0)
        .show(egui_ctxs.ctx_mut(), |ui| {
            layers_menu(ui, &mut current_layer, &mut update_draw, &mut display_opts);
        })
        .unwrap()
        .response
        .rect;
    occupied_screen_space.push_egui_window_rect(rect);
}

fn layers_menu(
    ui: &mut egui::Ui,
    current_layer: &mut OverlayLayerKind,
    update_draw: &mut UpdateDraw,
    display_opts: &mut DisplayOpts,
) {
    let mut new_layer = *current_layer;
    for kind in OverlayLayerKind::iter() {
        if ui.radio_value(&mut new_layer, kind, t!(kind)).clicked() {
            ui.close_menu();
        }
    }
    if new_layer != *current_layer {
        *current_layer = new_layer;
        update_draw.update();
    }
    ui.separator();

    let old = *display_opts;
    ui.checkbox(&mut display_opts.animals, t!("animal"));
    ui.checkbox(&mut display_opts.cities, t!("cities"));
    ui.checkbox(&mut display_opts.city_icons, t!("city-icons"));
    ui.checkbox(&mut display_opts.structures, t!("structures"));
    if *display_opts != old {
        update_draw.update();
    }
}
