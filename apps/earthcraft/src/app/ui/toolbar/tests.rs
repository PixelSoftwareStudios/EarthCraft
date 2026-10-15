use super::*;

#[test]
fn ruler_shortcut_and_sidebar_checkbox_share_state_and_camera_requires_an_image() {
    let ctx = egui::Context::default();
    let mut ruler = false;
    let frame = |ruler: &mut bool, available, events| {
        let mut rectangles = [egui::Rect::NOTHING; 3];
        let mut camera_clicked = false;
        let _ = crate::app::test_ui::run(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600.0, 400.0),
                )),
                events,
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        rectangles[0] = ruler_button(ui, ruler, Lang::En).rect;
                        let screenshot = screenshot_button(ui, available, Lang::En);
                        assert_eq!(screenshot.enabled(), available);
                        rectangles[2] = screenshot.rect;
                        camera_clicked = screenshot.clicked();
                    });
                    rectangles[1] = ui
                        .checkbox(ruler, Lang::En.text("ruler-enable-toggle"))
                        .rect;
                });
            },
        );
        (rectangles, camera_clicked)
    };
    let (rectangles, _) = frame(&mut ruler, false, vec![]);
    let click = |ruler: &mut bool, available, rect: egui::Rect| {
        let pos = rect.center();
        let _ = frame(ruler, available, vec![egui::Event::PointerMoved(pos)]);
        let button = |pressed| egui::Event::PointerButton {
            pos,
            pressed,
            button: egui::PointerButton::Primary,
            modifiers: egui::Modifiers::NONE,
        };
        let _ = frame(ruler, available, vec![button(true)]);
        frame(ruler, available, vec![button(false)]).1
    };
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler);
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler, "reopening the ruler must not toggle it off");
    click(&mut ruler, false, rectangles[1]);
    assert!(!ruler, "the sidebar controls the same state");
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler);
    assert!(!click(&mut ruler, false, rectangles[2]));
    assert!(click(&mut ruler, true, rectangles[2]));
}
