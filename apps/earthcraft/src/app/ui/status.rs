//! Compact toolbar status text, with full details only when something is hidden.
use crate::app::files::FileStatus;
use eframe::egui::{self, Color32, FontId, TextStyle};

const STATUS_TEXT_SCALE: f32 = 0.9;

pub(super) fn show(ui: &mut egui::Ui, message: &str, file: Option<&FileStatus>) -> egui::Response {
    let compact = match file {
        Some(file) if file.message == message => {
            let full_path = file.path.to_string_lossy();
            message.replace(full_path.as_ref(), &super::paths::short_folder(&full_path))
        }
        _ => message.to_owned(),
    };
    let available = ui.available_width().max(0.0);
    let font = status_font(ui);
    let fitted = fit_tail(ui, &compact, available, &font);
    // Abbreviated paths also hide details that must remain available on hover.
    let truncated = fitted != message;
    let response = ui.add_sized(
        [available, ui.spacing().interact_size.y],
        egui::Label::new(egui::RichText::new(fitted).font(font))
            .truncate()
            .show_tooltip_when_elided(false),
    );
    if truncated {
        response.on_hover_text(message)
    } else {
        response
    }
}

fn status_font(ui: &egui::Ui) -> FontId {
    let mut font = TextStyle::Body.resolve(ui.style());
    font.size *= STATUS_TEXT_SCALE;
    font
}

/// Keep the filename end visible on narrow toolbars, using the displayed font.
fn fit_tail(ui: &egui::Ui, text: &str, width_points: f32, font: &FontId) -> String {
    let fits = |text: &str| {
        ui.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_owned(), font.clone(), Color32::WHITE)
                .size()
                .x
        }) <= width_points
    };
    if fits(text) {
        return text.to_owned();
    }
    if !fits("...") {
        return String::new();
    }
    let boundaries: Vec<_> = text
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(text.len()))
        .collect();
    let mut low = 0;
    let mut high = boundaries.len() - 1;
    while low < high {
        let mid = (low + high) / 2;
        if fits(&format!("...{}", &text[boundaries[mid]..])) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    format!("...{}", &text[boundaries[low]..])
}

#[cfg(test)]
mod tests;
