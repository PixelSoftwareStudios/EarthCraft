# Controller navigation

[English](CONTROLLERS.md) | [Português (Brasil)](CONTROLLERS.pt-BR.md)

Settings → Controllers enables standard gamepads/joysticks recognized by
[gilrs 0.11.2](https://docs.rs/gilrs/0.11.2/gilrs/). Connect/disconnect without
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

Profiles persist locally in settings.json using gilrs model UUID, vendor/product
IDs and name. Different models retain independent profiles; identical models
without distinct backend identities share a profile. This is not a physical
serial-number identity. Unknown/malformed profiles fall back independently and
numeric values are bounded. No network or executable profile content is used.

Linux builds need pkg-config and libudev development files, with libudev at
runtime; FreeBSD uses libudev-devd/libinotify and appropriate evdev/webcamd device
access. Snap requests the joystick interface; Flatpak requests input devices.
Platform sandbox/device permissions can still require user setup. Adapter errors
appear in Settings. Packaging and actual hardware behavior require platform CI
and physical-device checks; compiling on macOS does not certify them.

This first slice does not implement Studio/flight modes, raw HOTAS/HID mappings,
SpaceMouse, rumble, per-workspace profiles or profile file import/export. Those
need their own modes/adapters rather than inactive settings. Automated fixtures
exercise deadzones, response/inversion, button edges/release, explicit conflict
replacement, bounded camera steps and independent profile persistence.

The lockfile retains libc 0.2.189: FreeBSD CI exposed nix 0.31.3 referring to
NOTE_PCTRLMASK, removed by libc 0.2.190. This is a targeted compatibility pin in
the application lockfile; reevaluate it when nix/gilrs publishes a compatible
update. No dependency source patch or application unsafe code is required.

Tilt ranges from nadir (0°) to 85° toward the horizon without moving the camera.
Middle-drag or the Globe slider also adjusts tilt. R resets heading and tilt; N only
resets heading. Fly-to smoothly returns to nadir. Saved startup views retain tilt.
Legacy five-action profiles retain bindings and gain right-stick-Y tilt.
The live monitor uses fixed button rows. Physical USB Pro Controller mapping
and interactive navigation still require hardware review.
