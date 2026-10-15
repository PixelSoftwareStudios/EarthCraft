//! Toolbar shortcuts use the sidebar's ruler state and the existing file service.
use earthcraft_core::language::Lang;
use eframe::egui;

const ICON_STROKE_POINTS: f32 = 1.25;

#[derive(Clone, Copy)]
enum Icon {
    Ruler,
    Pin,
    Camera,
}

pub(super) fn ruler_button(ui: &mut egui::Ui, ruler: &mut bool, lang: Lang) -> egui::Response {
    let response = icon_button(ui, Icon::Ruler, *ruler, lang.text("toolbar-ruler-tooltip"));
    if response.clicked() {
        *ruler = !*ruler;
    }
    response
}

pub(super) fn screenshot_button(ui: &mut egui::Ui, available: bool, lang: Lang) -> egui::Response {
    ui.add_enabled_ui(available, |ui| {
        icon_button(
            ui,
            Icon::Camera,
            false,
            lang.text("toolbar-screenshot-tooltip"),
        )
    })
    .inner
}

pub(super) fn placemark_button(ui: &mut egui::Ui, lang: Lang) -> egui::Response {
    icon_button(ui, Icon::Pin, false, lang.text("toolbar-placemark-tooltip"))
}

fn icon_button(ui: &mut egui::Ui, icon: Icon, selected: bool, tooltip: &str) -> egui::Response {
    let id = ui.next_auto_id().with("toolbar-icon");
    let icon_size = ui.text_style_height(&egui::TextStyle::Button);
    let button = egui::Button::new(egui::Atom::custom(id, egui::Vec2::splat(icon_size)))
        .selected(selected)
        .min_size(egui::Vec2::splat(ui.spacing().interact_size.y))
        .atom_ui(ui);
    if let Some(rect) = button.rect(id) {
        let stroke = ui
            .style()
            .interact_selectable(&button.response, selected)
            .fg_stroke;
        paint_icon(
            ui.painter(),
            rect,
            icon,
            egui::Stroke::new(ICON_STROKE_POINTS, stroke.color),
        );
    }
    button.response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, tooltip)
    });
    button.response.on_hover_text(tooltip)
}

fn paint_icon(painter: &egui::Painter, rect: egui::Rect, icon: Icon, stroke: egui::Stroke) {
    let point = |x, y| rect.min + rect.size() * egui::vec2(x, y);
    match icon {
        Icon::Ruler => {
            painter.rect_stroke(
                egui::Rect::from_min_max(point(0.05, 0.25), point(0.95, 0.75)),
                0.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            for index in 1..6 {
                let x = index as f32 / 6.0;
                let bottom = if index % 2 == 0 { 0.6 } else { 0.48 };
                painter.line_segment([point(x, 0.25), point(x, bottom)], stroke);
            }
        }
        Icon::Pin => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(0.5, 0.95),
                    point(0.12, 0.64),
                    point(0.05, 0.4),
                    point(0.18, 0.13),
                    point(0.5, 0.05),
                    point(0.82, 0.13),
                    point(0.95, 0.4),
                    point(0.88, 0.64),
                ],
                stroke,
            ));
            painter.circle_stroke(point(0.5, 0.38), rect.width() * 0.18, stroke);
        }
        Icon::Camera => {
            painter.add(egui::Shape::closed_line(
                vec![
                    point(0.05, 0.3),
                    point(0.25, 0.3),
                    point(0.35, 0.15),
                    point(0.65, 0.15),
                    point(0.75, 0.3),
                    point(0.95, 0.3),
                    point(0.95, 0.85),
                    point(0.05, 0.85),
                ],
                stroke,
            ));
            painter.circle_stroke(point(0.5, 0.56), rect.width() * 0.18, stroke);
        }
    }
}

#[cfg(test)]
mod tests;
