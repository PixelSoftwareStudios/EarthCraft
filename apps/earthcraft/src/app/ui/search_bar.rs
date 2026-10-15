//! Search input and submission widgets; query/services live in app::search.
use crate::app::App;
use eframe::egui::{self, Key, Rect};

const SEARCH_FIELD_WIDTH_POINTS: f32 = 320.0;

impl App {
    pub(super) fn search_bar(&mut self, ui: &mut egui::Ui) -> Rect {
        let lg = self.lang;
        // Keys for the suggestion list must be consumed before the TextEdit sees them.
        let popup = self.sugg_open && !self.suggestions.is_empty();
        let (mut down, mut up, mut esc) = (false, false, false);
        if popup {
            ui.input_mut(|i| {
                down = i.consume_key(egui::Modifiers::NONE, Key::ArrowDown);
                up = i.consume_key(egui::Modifiers::NONE, Key::ArrowUp);
                esc = i.consume_key(egui::Modifiers::NONE, Key::Escape);
            });
            let n = self.suggestions.len();
            if down {
                self.sugg_sel = Some(self.sugg_sel.map_or(0, |i| (i + 1) % n));
            }
            if up {
                self.sugg_sel = Some(self.sugg_sel.map_or(n - 1, |i| (i + n - 1) % n));
            }
            if esc {
                self.sugg_open = false;
            }
        }

        let r = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text(lg.text("search-input-placeholder"))
                .desired_width(SEARCH_FIELD_WIDTH_POINTS * self.settings.ui_scale.max(1.0)),
        );
        if r.changed() {
            self.on_query_changed();
        }
        if r.gained_focus() && !self.query.trim().is_empty() && !self.suggestions.is_empty() {
            self.sugg_open = true;
        }
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        if enter {
            match (self.sugg_open, self.sugg_sel) {
                (true, Some(i)) => self.choose_suggestion(i),
                _ => {
                    self.sugg_open = false;
                    self.start_search(ui);
                }
            }
        }
        if ui.button(lg.text("search-submit-action")).clicked() {
            self.sugg_open = false;
            self.start_search(ui);
        }
        r.rect
    }
}
