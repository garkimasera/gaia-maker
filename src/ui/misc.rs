use bevy_egui::egui::{self, epaint, load::SizedTexture};

use crate::screen::OccupiedScreenSpace;

use super::UiTextures;

pub fn label_with_icon(
    ui: &mut egui::Ui,
    textures: &UiTextures,
    icon: &str,
    s: impl Into<egui::WidgetText>,
) {
    let icon = textures.get(icon);
    ui.add(LabelWithIcon::new(icon, s));
}

pub struct LabelWithIcon {
    icon: SizedTexture,
    text: egui::WidgetText,
}

impl LabelWithIcon {
    pub fn new(icon: impl Into<SizedTexture>, text: impl Into<egui::WidgetText>) -> Self {
        Self {
            icon: icon.into(),
            text: text.into(),
        }
    }
}

impl egui::Widget for LabelWithIcon {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let layout_job =
            self.text
                .into_layout_job(ui.style(), egui::FontSelection::Default, egui::Align::Min);
        let galley = ui.fonts(|fonts| fonts.layout_job(layout_job));

        let icon_size = self.icon.size;
        let galley_size = galley.rect.size();
        let desired_size =
            egui::Vec2::new(icon_size.x + galley_size.x, icon_size.y.max(galley_size.y));

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, ui.is_enabled(), galley.text())
        });

        let (icon_pos_y, galley_pos_y) = if icon_size.y > galley_size.y {
            (0.0, (icon_size.y - galley_size.y) / 2.0)
        } else {
            ((galley_size.y - icon_size.y) / 2.0, 0.0)
        };

        let icon_rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, icon_pos_y), icon_size)
            .translate(rect.left_top().to_vec2());
        let galley_pos = rect.left_top() + egui::Vec2::new(icon_size.x, galley_pos_y);

        if ui.is_rect_visible(response.rect) {
            let painter = ui.painter();

            painter.add(
                epaint::RectShape::filled(icon_rect, 0, egui::Color32::WHITE).with_texture(
                    self.icon.id,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                ),
            );
            painter.add(epaint::TextShape::new(
                galley_pos,
                galley,
                ui.style().visuals.text_color(),
            ));
        }
        response
    }
}

pub fn small_window_frame(ctx: &egui::Context) -> egui::Frame {
    let visuals = &ctx.style().visuals;
    egui::Frame::default()
        .fill(visuals.window_fill.gamma_multiply(0.95))
        .stroke(visuals.window_stroke)
        .inner_margin(egui::Margin::same(4))
}

pub fn constrain_to_rect(
    ctx: &egui::Context,
    occupied_screen_space: &OccupiedScreenSpace,
) -> egui::Rect {
    let rect = ctx.screen_rect();
    egui::Rect::from_two_pos(
        rect.left_top() + egui::vec2(0.0, occupied_screen_space.occupied_top),
        rect.right_bottom(),
    )
}
