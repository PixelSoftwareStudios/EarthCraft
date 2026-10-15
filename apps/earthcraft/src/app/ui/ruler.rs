//! Ruler-only display settings; camera altitude and scale-bar units stay separate.
use earthcraft_core::{language::Lang, units::RulerUnit};
use eframe::egui;
mod points;
pub(in crate::app) use points::PointEditor;

pub(super) fn section<R>(
    ui: &mut egui::Ui,
    lang: Lang,
    reveal: bool,
    body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::collapsing_header::CollapsingResponse<R> {
    let section = egui::CollapsingHeader::new(lang.text("ruler-section-title"))
        .id_salt("ruler-section")
        .default_open(true)
        .open(reveal.then_some(true))
        .show(ui, body);
    if reveal {
        section.header_response.scroll_to_me(Some(egui::Align::TOP));
    }
    section
}

pub(super) fn unit_selector(
    ui: &mut egui::Ui,
    mut selected: RulerUnit,
    us_locale: bool,
    lang: Lang,
) -> RulerUnit {
    ui.horizontal(|ui| {
        ui.label(lang.text("ruler-units-label"));
        egui::ComboBox::from_id_salt("ruler_units")
            .width(ui.available_width())
            .wrap()
            .selected_text(selected.selector_label(lang, us_locale))
            .show_ui(ui, |ui| {
                super::selection::size_menu(
                    ui,
                    RulerUnit::ALL
                        .into_iter()
                        .map(|unit| unit.selector_label(lang, us_locale)),
                );
                for unit in RulerUnit::ALL {
                    super::selection::full_width_value(
                        ui,
                        &mut selected,
                        unit,
                        unit.selector_label(lang, us_locale),
                    )
                    .on_hover_text(unit.help(lang));
                }
            });
    });
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_reopens_and_scrolls_to_controls_and_language_keeps_section_state() {
        let ctx = egui::Context::default();
        ctx.global_style_mut(|style| {
            style.animation_time = 0.0;
            style.scroll_animation = egui::style::ScrollAnimation::none();
        });
        let frame = |lang, reveal, close| {
            let mut controls_visible = false;
            let mut body_open = false;
            let _ = crate::app::test_ui::run(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(300.0, 200.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    egui::CentralPanel::default().show(ui, |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.allocate_space(egui::vec2(100.0, 600.0));
                            let response = section(ui, lang, reveal, |ui| {
                                let control =
                                    ui.checkbox(&mut false, lang.text("ruler-enable-toggle"));
                                controls_visible = ui.clip_rect().contains_rect(control.rect);
                            });
                            body_open = response.body_response.is_some();
                            if close {
                                let mut state = egui::collapsing_header::CollapsingState::load(
                                    ui.ctx(),
                                    response.header_response.id,
                                )
                                .unwrap();
                                state.set_open(false);
                                state.store(ui.ctx());
                            }
                        });
                    });
                },
            );
            (body_open, controls_visible)
        };
        frame(Lang::En, false, true);
        assert!(!frame(Lang::En, false, false).0);
        assert!(!frame(Lang::PtBr, false, false).0);
        assert!(frame(Lang::PtBr, true, false).0);
        // ScrollArea publishes its adjusted offset after layout; draw it next pass.
        assert!(frame(Lang::En, false, false).0);
        assert_eq!(frame(Lang::En, false, false), (true, true));
    }
    #[test]
    fn selector_fits_narrow_sidebar_for_both_languages_and_all_units() {
        let ctx = egui::Context::default();
        for lang in Lang::ALL {
            for unit in RulerUnit::ALL {
                for us_locale in [true, false] {
                    let _ = crate::app::test_ui::run(
                        &ctx,
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(240.0, 200.0),
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            egui::CentralPanel::default().show(ui, |ui| {
                                let bounds = ui.max_rect();
                                assert_eq!(unit_selector(ui, unit, us_locale, lang), unit);
                                assert!(
                                    ui.min_rect().right() <= bounds.right(),
                                    "{lang:?} {unit:?}"
                                );
                            });
                        },
                    );
                }
            }
        }
    }
}
