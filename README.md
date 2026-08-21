# AL80 Studio

Open-source advanced control software and firmware tooling for the
YUNZII AL80 keyboard.

## Project goals

AL80 Studio aims to provide advanced control of the AL80 without
requiring users to edit firmware source code manually.

Planned capabilities include:

- automatic AL80 detection
- Raw HID communication
- RGB control
- RGB effect configuration
- LCD widgets and layouts
- volume and mute OSD
- knob configuration
- profiles
- macros and remapping
- diagnostics and performance telemetry
- safe firmware update and recovery tooling

## Development principle

Runtime configuration should use Raw HID whenever possible.

Firmware flashing should only be required when a feature genuinely
requires a firmware change.

## Current hardware baseline

The development keyboard currently runs a tested QMK-based firmware
with:

- USB polling: 1000 Hz
- matrix scan with custom Snake/Heart effect: ~1472 Hz
- eager key-down
- per-key deferred key-up debounce: 3 ms
- NKRO enabled
- optimized Snake/Heart RGB renderer
- non-blocking LCD row transmission
- working rotary encoder volume control
- Raw HID telemetry/control commands

This baseline is treated as known-good and must not be modified
implicitly by application development.

## Status

AL80 Studio is currently in early development.

Target first release:

`v0.1.0-alpha`

Initial platform:

Linux / Fedora

Future platforms:

- Linux
- Windows
- macOS

## Safety

Some reverse-engineered device operations may write persistent device
storage.

AL80 Studio will distinguish:

- read-only operations
- volatile/runtime operations
- persistent operations
- firmware flashing

Dangerous or insufficiently understood operations will not be exposed
as normal UI controls.
