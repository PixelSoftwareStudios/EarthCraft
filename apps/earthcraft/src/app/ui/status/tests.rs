use super::*;

#[test]
fn narrow_labels_keep_the_filename_end_and_fit_the_displayed_font() {
    let ctx = egui::Context::default();
    let _ = crate::app::test_ui::run(&ctx, egui::RawInput::default(), |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            let text = "1 placemark imported. — ~/Downloads/東京.kml";
            let font = status_font(ui);
            assert_eq!(fit_tail(ui, text, 1000.0, &font), text);
            let fitted = fit_tail(ui, text, 110.0, &font);
            assert!(fitted.starts_with("..."));
            assert!(fitted.ends_with("東京.kml"));
            let width =
                ui.fonts_mut(|fonts| fonts.layout_no_wrap(fitted, font, Color32::WHITE).size().x);
            assert!(width <= 110.0);
            assert_eq!(fit_tail(ui, text, 0.0, &status_font(ui)), "");
            assert_eq!(fit_tail(ui, text, 1.0, &status_font(ui)), "");
        });
    });
}

#[test]
fn hover_shows_full_message_only_when_text_or_path_is_shortened() {
    let path = "/long/export/folder/EarthCraft/東京.png";
    let file = FileStatus {
        message: format!("Screenshot saved — {path}"),
        path: path.into(),
    };
    for (message, file, width) in [
        ("Ready".to_owned(), None, 300.0),
        (
            "Screenshot saved — ~/Pictures/EarthCraft/東京.png".to_owned(),
            None,
            90.0,
        ),
        (file.message.clone(), Some(&file), 500.0),
    ] {
        let ctx = egui::Context::default();
        ctx.global_style_mut(|style| {
            style.interaction.tooltip_delay = 0.0;
            style.interaction.show_tooltips_only_when_still = false;
            style.animation_time = 0.0;
        });
        let mut target = egui::Pos2::ZERO;
        let mut maximum_full_messages = 0;
        for frame in 0..6 {
            let output = crate::app::test_ui::run(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(640.0, 300.0),
                    )),
                    time: Some(f64::from(frame) * 0.2),
                    events: if frame > 0 {
                        vec![egui::Event::PointerMoved(target)]
                    } else {
                        vec![]
                    },
                    ..Default::default()
                },
                |ui| {
                    egui::CentralPanel::default().show(ui, |ui| {
                        ui.set_max_width(width);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            target = show(ui, &message, file).rect.center();
                        });
                    });
                },
            );
            let full_messages = output.shapes.iter().filter(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == message)
            }).count();
            maximum_full_messages = maximum_full_messages.max(full_messages);
        }
        // Ready appears once in the label, never again in a needless tooltip.
        // Shortened messages appear in full only in their hover tooltip.
        assert_eq!(maximum_full_messages, 1, "{message} at width {width}");
    }
}
