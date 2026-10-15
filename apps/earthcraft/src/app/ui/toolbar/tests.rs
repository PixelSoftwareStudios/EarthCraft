use super::*;

#[test]
fn ruler_shortcut_and_sidebar_checkbox_share_state_and_camera_requires_an_image() {
    let ctx = egui::Context::default();
    let mut ruler = false;
    let frame = |ruler: &mut bool, available, events| {
        let mut rectangles = [egui::Rect::NOTHING; 4];
        let mut camera_clicked = false;
        let mut placemark_clicked = false;
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
                        let placemark = placemark_button(ui, Lang::En);
                        rectangles[3] = placemark.rect;
                        placemark_clicked = placemark.clicked();
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
        (rectangles, camera_clicked, placemark_clicked)
    };
    let (rectangles, _, _) = frame(&mut ruler, false, vec![]);
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
        let (_, camera, placemark) = frame(ruler, available, vec![button(false)]);
        (camera, placemark)
    };
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler);
    click(&mut ruler, false, rectangles[0]);
    assert!(
        !ruler,
        "clicking the active toolbar ruler switches measuring off"
    );
    click(&mut ruler, false, rectangles[1]);
    assert!(ruler, "the sidebar can enable the same state");
    click(&mut ruler, false, rectangles[0]);
    assert!(
        !ruler,
        "the toolbar can disable measuring enabled in the sidebar"
    );
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler);
    click(&mut ruler, false, rectangles[1]);
    assert!(!ruler, "the sidebar controls the same state");
    click(&mut ruler, false, rectangles[0]);
    assert!(ruler);
    assert_eq!(click(&mut ruler, false, rectangles[3]), (false, true));
    assert!(ruler, "adding a placemark must not change ruler state");
    assert_eq!(click(&mut ruler, false, rectangles[2]), (false, false));
    assert_eq!(click(&mut ruler, true, rectangles[2]), (true, false));
}
