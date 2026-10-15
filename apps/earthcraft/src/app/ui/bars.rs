use super::super::App;
use eframe::egui::{self, Rect};
use emath::GuiRounding;
impl App {
    pub(in crate::app) fn top_bar(&mut self, ui: &mut egui::Ui) {
        let lg = self.lang;
        let mut field_rect = Rect::NOTHING;
        const BAR_MARGIN_POINTS: egui::Margin = egui::Margin::symmetric(8, 4);
        egui::Panel::top("top")
            .frame(egui::Frame::side_top_panel(&ui.global_style()).inner_margin(BAR_MARGIN_POINTS))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let sidebar_hint = if self.sidebar_open {
                        lg.text("sidebar-hide-tooltip")
                    } else {
                        lg.text("sidebar-show-tooltip")
                    };
                    ui.toggle_value(&mut self.sidebar_open, "☰")
                        .on_hover_text(sidebar_hint);
                    ui.strong("🌍 EarthCraft");
                    if ui.button(lg.text("top-explore-action")).clicked() {
                        self.explorer.open = true;
                    }
                    for (name, body) in [
                        (lg.text("top-earth"), earthcraft_core::body::Body::Earth),
                        (lg.text("top-moon"), earthcraft_core::body::Body::Moon),
                        (
                            lg.text("top-custom-planet"),
                            earthcraft_core::body::Body::Custom,
                        ),
                    ] {
                        if ui.button(name).clicked() {
                            self.flat_earth.open = false;
                            self.explorer.choose(body, &self.file_jobs);
                            if body == earthcraft_core::body::Body::Custom {
                                self.explorer.open = true;
                            }
                            self.touch();
                        }
                    }
                    ui.separator();

                    field_rect = self.search_bar(ui);
                    ui.separator();
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("⚙")
                            .on_hover_text(lg.text("settings-open-tooltip"))
                            .clicked()
                        {
                            self.show_settings = !self.show_settings;
                        }
                        if super::toolbar::screenshot_button(ui, self.image.is_some(), lg).clicked()
                        {
                            self.save_screenshot();
                        }
                        if super::toolbar::placemark_button(ui, lg).clicked() {
                            self.add_center_placemark();
                            self.sidebar_open = true;
                            self.place_reveal_requested = true;
                            self.ruler_reveal_requested = false;
                        }
                        if super::toolbar::ruler_button(ui, &mut self.ruler, lg).clicked() {
                            self.ruler_reveal_requested = self.ruler;
                            if self.ruler {
                                self.sidebar_open = true;
                                self.place_reveal_requested = false;
                            }
                        }
                        super::status::show(ui, &self.status, self.status_file.as_ref());
                    });
                });
            });
        self.suggestion_popup(ui, field_rect);
    }

    /// Status below the map.
    pub(in crate::app) fn bottom_bar(&mut self, ui: &mut egui::Ui) {
        let lg = self.lang;
        egui::Panel::bottom("bottom").show(ui, |ui| {
            let bounds = ui.available_rect_before_wrap();
            let heading = self
                .renderer
                .displayed_view()
                .map_or(self.cam.heading, |(cam, _)| cam.heading);
            let font = egui::TextStyle::Monospace.resolve(ui.style());
            let heading_width = ui
                .painter()
                .layout_no_wrap("359° NW".into(), font, ui.visuals().text_color())
                .size()
                .x
                + ui.spacing().item_spacing.x * 2.0;
            let split = (bounds.right() - heading_width).max(bounds.left());
            let heading_rect = Rect::from_min_max(egui::pos2(split, bounds.top()), bounds.max);
            let left_rect = Rect::from_min_max(
                bounds.min,
                egui::pos2(split - ui.spacing().item_spacing.x, bounds.bottom()),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(left_rect), |ui| {
                ui.horizontal(|ui| {
                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                    let sidebar_right =
                        egui::containers::panel::PanelState::load(ui, egui::Id::new("side"))
                            .map_or(
                                ui.content_rect().left()
                                    + super::sidebar::DEFAULT_SIDEBAR_WIDTH_POINTS,
                                |state| state.outer_rect.right(),
                            );
                    // Keep the cursor divider on the inner edge of the sidebar separator.
                    const SIDEBAR_SEPARATOR_INSET_POINTS: f32 = 1.0;
                    let sidebar_border_x = (sidebar_right - SIDEBAR_SEPARATOR_INSET_POINTS)
                        .round_to_pixels(ui.pixels_per_point());
                    super::cursor::cursor_readout(ui, self.cursor, sidebar_border_x);
                    ui.monospace(lg.format(
                        "statusbar-camera-center",
                        &[
                            ("latitude", format!("{:.4}", self.cam.lat).into()),
                            ("longitude", format!("{:.4}", self.cam.lon).into()),
                        ],
                    ));
                    ui.separator();
                    let altitude = lg.format(
                        "statusbar-camera-altitude",
                        &[(
                            "distance",
                            self.distance_units().altitude(self.cam.alt).into(),
                        )],
                    );
                    let altitude_response = ui
                        .add(
                            egui::Label::new(egui::RichText::new(altitude).monospace())
                                .sense(egui::Sense::click()),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .on_hover_text(lg.text("altitude-edit-tooltip"));
                    if altitude_response.clicked() {
                        self.altitude_editor = Some(super::altitude::AltitudeEditor::new(
                            self.cam.alt,
                            self.distance_units(),
                            altitude_response.rect.left_top(),
                        ));
                    }
                    ui.separator();
                    let n = self.cache.inflight();
                    if n > 0 {
                        ui.monospace(lg.format(
                            "statusbar-tiles-loading",
                            &[("count", n.to_string().into())],
                        ));
                    } else {
                        ui.monospace("✔ tiles ok");
                    }
                    let action = super::updater::indicator(
                        ui,
                        self.updater.state(),
                        lg,
                        self.update_restart.error.as_deref(),
                        self.update_restart.is_saving(),
                    );
                    match action {
                        super::updater::Action::InstallOrRetry => self.updater.activate(),
                        super::updater::Action::Restart => self.request_update_restart(ui),
                        super::updater::Action::OpenReleases => {
                            ui.open_url(egui::OpenUrl::new_tab(
                                earthcraft_data::updater::RELEASES_URL,
                            ));
                        }
                        super::updater::Action::None => {}
                    }
                });
            });
            ui.scope_builder(egui::UiBuilder::new().max_rect(heading_rect), |ui| {
                if super::compass::heading_readout(ui, heading, lg) {
                    self.cam.heading = 0.0;
                    self.fly = None;
                    self.touch();
                    ui.request_repaint();
                }
            });
        });
    }

    // ───────── Globe / Globo ─────────
}
