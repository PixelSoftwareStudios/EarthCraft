# Controller navigation

[English](CONTROLLERS.md) | [Português (Brasil)](CONTROLLERS.pt-BR.md)

Settings → Controllers enables gamepads recognized by
[SDL3](https://wiki.libsdl.org/SDL3/CategoryGamepad). Connect/disconnect without
restarting. Choose a device, adjust its Explore profile, watch normalized inputs,
and restore that profile's defaults. Mouse/keyboard remain available.

| Default input | Action |
| --- | --- |
| Left stick X/Y | Pan the globe |
| Right stick X / Y | Heading / camera tilt |
| Left/right analog triggers | Zoom out/in |
| South | Add a placemark at camera center |
| East | Cancel camera movement/ruler dragging |
| North | Reset north and tilt |
| West | Toggle measurements and reveal their sidebar controls |
| Select / Start | Places / Settings |

Each navigation action selects one of six normalized axes or Unbound, with its
own deadzone, inversion, gain and response exponent (1 is linear). Shared axes are
allowed explicitly. Every recognized normalized button can be assigned an action.
Press-to-bind listens for ten seconds, supports cancellation, requires an axis
to return to neutral before a threshold crossing, and asks before replacing an
occupied button. Labels use normalized positions, not assumed console branding.

Navigation is time-based, bounded after slow frames, and applied once per egui
frame. Earth uses the selected Earth-model camera math; other Explorer bodies
use their existing orbit controls. Tilt applies to Earth only. Controller tool commands
reuse the same dispatcher as menus. Unplugging releases state; text focus,
settings/dialogs/popups, unfocused windows and update restart suspend navigation.
Return bound axes/buttons to neutral after reconnecting, switching devices or leaving
an editor to prevent accidental motion. A bounded event drain and repaint timer
keep hotplugging available while the globe is idle.

Profiles persist locally in settings.json using SDL model GUID, vendor/product
IDs and name. Matching legacy profiles migrate by vendor/product identity. Different models retain independent profiles; identical models
without distinct backend identities share a profile. This is not a physical
serial-number identity. Unknown/malformed profiles fall back independently and
numeric values are bounded. No network or executable profile content is used.

Native builds statically compile only SDL3 gamepad/HID subsystems and require CMake
and a C/C++ compiler. SDL handles Switch USB protocol initialization and calibration.
The monitor samples all axes/buttons with fixed rows instead of showing transient events.
Linux/FreeBSD still need OS input-device access. Snap requests the joystick interface; Flatpak requests input devices.
Platform sandbox/device permissions can still require user setup. Adapter errors
appear in Settings. Packaging and actual hardware behavior require platform CI
and physical-device checks; compiling on macOS does not certify them.

This first slice does not implement Studio/flight modes, raw HOTAS/HID mappings,
SpaceMouse, rumble, per-workspace profiles or profile file import/export. Those
need their own modes/adapters rather than inactive settings. Automated fixtures
exercise deadzones, response/inversion, button edges/release, explicit conflict
replacement, bounded camera steps and independent profile persistence.

Camera tilt is measured from nadir (0°) toward the horizon, limited to 85°. The camera
position/altitude stays fixed; at high altitude the center ray may look into space.
Middle-drag or the Globe slider also adjusts tilt. R resets heading and tilt; N only
resets heading. Fly-to smoothly returns to nadir. A saved startup view retains tilt.
Legacy five-action profiles retain bindings and gain right-stick-Y tilt.

The SDL switch driver replaces the generic HID interpretation that produced noisy
USB Pro Controller readings. Physical USB/Bluetooth tests remain required; synthetic
normalization and neutral-gate tests do not certify a particular controller/firmware.
