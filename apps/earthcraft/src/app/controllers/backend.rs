//! SDL owns Switch USB initialization/calibration and platform gamepad mappings.
use super::Device;
use earthcraft_core::controller::ControllerState;
use sdl3::{
    gamepad::{Axis, Button, Gamepad},
    joystick::JoystickId,
};
use std::collections::BTreeMap;

const STICK_POSITIVE_MAX: f64 = i16::MAX as f64;
const STICK_NEGATIVE_MAX: f64 = -(i16::MIN as f64);
const TRIGGER_BUTTON_THRESHOLD: f64 = 0.5;
const MAX_EVENTS_PER_POLL: usize = 512;

pub(super) struct Backend {
    opened: BTreeMap<JoystickId, Gamepad>,
    events: sdl3::EventPump,
    subsystem: sdl3::GamepadSubsystem,
    _context: sdl3::Sdl,
}
impl Backend {
    pub(super) fn new() -> Result<Self, String> {
        // Only input is initialized: eframe retains all window/event ownership.
        let context = sdl3::init().map_err(|error| error.to_string())?;
        let subsystem = context.gamepad().map_err(|error| error.to_string())?;
        let events = context.event_pump().map_err(|error| error.to_string())?;
        Ok(Self {
            opened: BTreeMap::new(),
            events,
            subsystem,
            _context: context,
        })
    }
    pub(super) fn poll(&mut self) -> Result<Vec<Device>, String> {
        // Pump/drain SDL's input queue, then sample a coherent state for every axis/button.
        // Reading snapshots also recovers if events were coalesced or a device reconnected.
        for _ in 0..MAX_EVENTS_PER_POLL {
            if self.events.poll_event().is_none() {
                break;
            }
        }
        let ids = self
            .subsystem
            .gamepads()
            .map_err(|error| error.to_string())?;
        self.opened
            .retain(|id, gamepad| ids.contains(id) && gamepad.connected());
        for id in ids {
            if let std::collections::btree_map::Entry::Vacant(entry) = self.opened.entry(id) {
                entry.insert(self.subsystem.open(id).map_err(|error| error.to_string())?);
            }
        }
        self.subsystem.update();
        Ok(self
            .opened
            .iter()
            .map(|(id, gamepad)| {
                let name = gamepad.name().unwrap_or_else(|| "Gamepad".into());
                let vendor = gamepad.vendor_id().unwrap_or_default();
                let product = gamepad.product_id().unwrap_or_default();
                let axes = [
                    Axis::LeftX,
                    Axis::LeftY,
                    Axis::RightX,
                    Axis::RightY,
                    Axis::TriggerLeft,
                    Axis::TriggerRight,
                ]
                .map(|axis| gamepad.axis(axis));
                let mut state = ControllerState {
                    axes: normalize_axes(axes),
                    ..Default::default()
                };
                // Keep existing profile button indices and physical-position names stable.
                let buttons = [
                    Button::South,
                    Button::East,
                    Button::North,
                    Button::West,
                    Button::LeftShoulder,
                    Button::RightShoulder,
                    Button::Misc3,
                    Button::Misc4,
                    Button::Back,
                    Button::Start,
                    Button::LeftStick,
                    Button::RightStick,
                    Button::DPadUp,
                    Button::DPadDown,
                    Button::DPadLeft,
                    Button::DPadRight,
                    Button::Guide,
                    Button::Misc1,
                    Button::Misc2,
                ];
                state.buttons = buttons.map(|button| gamepad.button(button));
                state.buttons[6] = state.axes[4] > TRIGGER_BUTTON_THRESHOLD;
                state.buttons[7] = state.axes[5] > TRIGGER_BUTTON_THRESHOLD;
                Device {
                    id: id.raw(),
                    key: format!(
                        "sdl-{}-{vendor:04x}-{product:04x}-{name}",
                        self.subsystem.guid_for_id(*id)
                    ),
                    legacy_suffix: format!("-{vendor:04x}-{product:04x}-"),
                    name,
                    state,
                }
            })
            .collect())
    }
}
fn normalize_axes(raw: [i16; 6]) -> [f64; 6] {
    std::array::from_fn(|index| {
        let value = f64::from(raw[index]);
        if index >= 4 {
            (value / STICK_POSITIVE_MAX).clamp(0.0, 1.0)
        } else {
            // Core uses positive Y for up, while SDL uses positive Y for down.
            let normalized = value
                / if value < 0.0 {
                    STICK_NEGATIVE_MAX
                } else {
                    STICK_POSITIVE_MAX
                };
            normalized * if index == 1 || index == 3 { -1.0 } else { 1.0 }
        }
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stick_xy_and_digital_switch_triggers_have_independent_zero_rest_values() {
        assert_eq!(normalize_axes([0; 6]), [0.0; 6]);
        assert_eq!(
            normalize_axes([i16::MIN, i16::MAX, i16::MAX, i16::MIN, i16::MAX, 0]),
            [-1.0, -1.0, 1.0, 1.0, 1.0, 0.0]
        );
        assert_eq!(normalize_axes([0, 0, 0, 0, -1, i16::MAX])[4..], [0.0, 1.0]);
    }
}
