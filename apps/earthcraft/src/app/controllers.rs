//! Native gamepad adapter. Bindings and navigation math live in core.
use super::{App, menu::Action};
use earthcraft_core::controller::{ControllerAction, ControllerMotion, ControllerState};
use earthcraft_data::settings::controllers::ControllerPreferences;
use eframe::egui;
mod backend;
use backend::Backend;
use std::time::{Duration, Instant};
const ACTIVE_POLL_INTERVAL: Duration = Duration::from_millis(16);
const HOTPLUG_POLL_INTERVAL: Duration = Duration::from_millis(250);
const BIND_TIMEOUT: Duration = Duration::from_secs(10);
const BIND_AXIS_THRESHOLD: f64 = 0.75;
const NEUTRAL_THRESHOLD: f64 = 0.2;
pub(super) const BUTTONS: [&str; 19] = [
    "South",
    "East",
    "North",
    "West",
    "LeftTrigger",
    "RightTrigger",
    "LeftTrigger2",
    "RightTrigger2",
    "Select",
    "Start",
    "LeftThumb",
    "RightThumb",
    "DPadUp",
    "DPadDown",
    "DPadLeft",
    "DPadRight",
    "Mode",
    "C",
    "Z",
];
pub(super) const AXIS_LABEL_IDS: [&str; 6] = [
    "controllers-left-x",
    "controllers-left-y",
    "controllers-right-x",
    "controllers-right-y",
    "controllers-left-trigger",
    "controllers-right-trigger",
];
pub(super) struct Device {
    pub(super) id: u32,
    legacy_suffix: String,
    pub(super) key: String,
    pub(super) name: String,
    pub(super) state: ControllerState,
}
pub(super) struct BindingCapture {
    pub(super) action: usize,
    pub(super) axis: bool,
    deadline: Instant,
    armed: [bool; 6],
}
pub(super) struct Controllers {
    backend: Option<Backend>,
    pub(super) devices: Vec<Device>,
    pub(super) error: Option<String>,
    pub(super) capture: Option<BindingCapture>,
    pub(super) conflict: Option<(usize, usize)>,
    suspended: bool,
    selected: String,
}
impl Controllers {
    pub(super) fn new() -> Self {
        let (backend, error) = match Backend::new() {
            Ok(backend) => (Some(backend), None),
            Err(error) => (
                None,
                Some(format!("Controller initialization failed: {error}")),
            ),
        };
        let devices = Vec::new();
        Self {
            backend,
            devices,
            error,
            capture: None,
            conflict: None,
            suspended: true,
            selected: String::new(),
        }
    }
    pub(super) fn listen(&mut self, action: usize, axis: bool) {
        self.conflict = None;
        self.capture = Some(BindingCapture {
            action,
            axis,
            deadline: Instant::now() + BIND_TIMEOUT,
            armed: [false; 6],
        });
        self.suspended = true;
    }
    pub(super) fn poll(
        &mut self,
        ctx: &egui::Context,
        prefs: &mut ControllerPreferences,
        allowed: bool,
    ) -> (ControllerMotion, Vec<ControllerAction>) {
        if prefs.selected.is_empty()
            && let Some(device) = self.devices.first()
        {
            prefs.selected = device.key.clone();
        }
        if self.selected != prefs.selected {
            self.selected = prefs.selected.clone();
            self.suspended = true;
            self.capture = None;
            self.conflict = None;
        }
        if !allowed || !prefs.enabled {
            self.suspended = true;
        }
        let mut actions = Vec::new();
        if self
            .capture
            .as_ref()
            .is_some_and(|capture| Instant::now() >= capture.deadline)
        {
            self.capture = None;
        }
        if let Some(backend) = &mut self.backend {
            match backend.poll() {
                Err(error) => {
                    self.error = Some(format!("Controller polling failed: {error}"));
                    self.devices.clear();
                    self.suspended = true;
                }
                Ok(mut devices) => {
                    self.error = None;
                    // Migrate a unique legacy model profile instead of discarding custom bindings.
                    if !prefs.selected.is_empty()
                        && !devices.iter().any(|device| device.key == prefs.selected)
                        && let Some(device) = devices
                            .iter()
                            .find(|device| prefs.selected.contains(&device.legacy_suffix))
                    {
                        if let Some(profile) = prefs.profiles.get(&prefs.selected).cloned() {
                            prefs.profiles.entry(device.key.clone()).or_insert(profile);
                        }
                        prefs.selected = device.key.clone();
                        self.suspended = true;
                    }
                    if prefs.selected.is_empty()
                        && let Some(device) = devices.first()
                    {
                        prefs.selected = device.key.clone();
                    }
                    for device in &mut devices {
                        let selected = device.key == prefs.selected;
                        let old = self.devices.iter().find(|old| old.id == device.id);
                        if selected && old.is_none() {
                            self.suspended = true;
                        }
                        let profile = prefs.profile(&device.key);
                        let mut previous = old.map(|old| old.state.clone()).unwrap_or_default();
                        for input in 0..BUTTONS.len() {
                            let down = device.state.buttons[input];
                            let rising = down && !previous.buttons[input];
                            let action = previous.button(input, down, &profile);
                            if selected
                                && rising
                                && self.capture.as_ref().is_some_and(|capture| !capture.axis)
                            {
                                let capture = self.capture.take().unwrap();
                                let profile = prefs.profiles.entry(device.key.clone()).or_default();
                                if !profile.move_button(capture.action, input, false) {
                                    self.conflict = Some((capture.action, input));
                                }
                            } else if selected
                                && allowed
                                && prefs.enabled
                                && !self.suspended
                                && self.capture.is_none()
                                && self.conflict.is_none()
                                && let Some(action) = action
                            {
                                actions.push(action);
                            }
                        }
                        if selected
                            && let Some(capture) = &mut self.capture
                            && capture.axis
                        {
                            for (input, value) in device.state.axes.iter().enumerate() {
                                capture.armed[input] |= value.abs() < NEUTRAL_THRESHOLD;
                                if capture.armed[input] && value.abs() >= BIND_AXIS_THRESHOLD {
                                    prefs.profiles.entry(device.key.clone()).or_default().axes
                                        [capture.action]
                                        .input = Some(input);
                                    self.capture = None;
                                    break;
                                }
                            }
                        }
                    }
                    if !devices.iter().any(|device| device.key == prefs.selected) {
                        self.capture = None;
                        self.suspended = true;
                    }
                    self.devices = devices;
                }
            }
        }
        ctx.request_repaint_after(if self.devices.is_empty() {
            HOTPLUG_POLL_INTERVAL
        } else {
            ACTIVE_POLL_INTERVAL
        });
        let Some(device) = self
            .devices
            .iter()
            .find(|device| device.key == prefs.selected)
        else {
            return (ControllerMotion::default(), actions);
        };
        if self.suspended
            && allowed
            && prefs.enabled
            && self.capture.is_none()
            && self.conflict.is_none()
            && neutral(&device.state, &prefs.profile(&device.key))
        {
            self.suspended = false;
        }
        let motion = if self.suspended || self.capture.is_some() || self.conflict.is_some() {
            ControllerMotion::default()
        } else {
            prefs.profile(&device.key).motion(&device.state)
        };
        (motion, actions)
    }
}
fn neutral(
    state: &ControllerState,
    profile: &earthcraft_core::controller::ControllerProfile,
) -> bool {
    profile.axes.iter().all(|binding| {
        binding.input.is_none_or(|input| {
            state
                .axes
                .get(input)
                .is_none_or(|value| value.abs() < NEUTRAL_THRESHOLD.max(binding.deadzone))
        })
    }) && state
        .buttons
        .iter()
        .zip(profile.buttons)
        .all(|(down, action)| !down || action == ControllerAction::None)
}

impl App {
    pub(super) fn controller_input(&mut self, ctx: &egui::Context, height_points: f32) -> bool {
        if ctx.current_pass_index() != 0 {
            return false;
        }
        let allowed = ctx.input(|input| input.focused)
            && !ctx.egui_wants_keyboard_input()
            && !self.show_settings
            && !self.show_about
            && !self.show_shortcuts
            && !self.explorer.open
            && self.altitude_editor.is_none()
            && self.ruler_point_editor.is_none()
            && !self.flat_earth.open
            && !self.update_restart.is_saving()
            && !egui::Popup::is_any_open(ctx);
        let (motion, actions) = self
            .controllers
            .poll(ctx, &mut self.settings.controllers, allowed);
        for action in actions {
            let menu = match action {
                ControllerAction::North => Some(Action::North),
                ControllerAction::Ruler => Some(Action::Ruler),
                ControllerAction::Places => Some(Action::ShowPlaces),
                ControllerAction::Settings => Some(Action::Settings),
                ControllerAction::Placemark => Some(Action::Placemark),
                ControllerAction::Cancel => {
                    self.fly = None;
                    self.ruler_paths.end_move();
                    self.ruler_drag = None;
                    None
                }
                ControllerAction::Reset => {
                    self.fly = None;
                    self.cam.reset_view();
                    None
                }
                ControllerAction::None => None,
            };
            if let Some(menu) = menu {
                self.run_menu_action(menu, ctx);
            }
        }
        if motion.is_active() && !self.show_settings {
            self.fly = None;
            let dt = super::frame_timing::navigation_delta_secs(ctx);
            motion.apply(
                &mut self.cam,
                dt,
                height_points,
                (self.explorer.active == earthcraft_core::body::Body::Earth)
                    .then_some(self.settings.earth_model),
            );
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use earthcraft_core::controller::ControllerProfile;
    #[test]
    fn neutral_gate_ignores_unbound_axes_and_buttons_but_requires_bound_controls_released() {
        let mut state = ControllerState::default();
        let mut profile = ControllerProfile::default();
        assert!(neutral(&state, &profile));
        state.axes[3] = 1.0;
        assert!(!neutral(&state, &profile));
        profile.axes[5].input = None;
        assert!(neutral(&state, &profile));
        state.buttons[18] = true;
        assert!(neutral(&state, &profile));
        state.buttons[0] = true;
        assert!(!neutral(&state, &profile));
    }
}
