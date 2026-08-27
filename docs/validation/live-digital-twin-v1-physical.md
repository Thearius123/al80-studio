# Live Digital Twin V1 — Physical Validation

Status: **PHYSICALLY VALIDATED**

Validation date: 2026-08-27

Public source baseline:

- commit `634a5aaee4442440957ace33258c0d618ca804f5`
- branch `main`
- firmware SHA-256 `ecfeeaf8ec7d0ad71e1ed480e7d296f49da4fe259cf2be6a64c7812fccd2d46f`
- firmware size `48828` bytes

## Hardware validation

Validated on a physical YUNZII AL80:

- Raw HID `0x4D` telemetry: PASS
- 82-LED telemetry frame: PASS
- firmware Snake source detection: PASS
- moving Snake frames: PASS
- Creator exact 82-LED `#112233` mirror: PASS
- LCD HOME / Volume / Mute logical states: PASS
- Dashboard live Snake mirror: PASS
- Creator 3D orbit and zoom: PASS
- Creator keys remain editable: PASS
- Creator scroll preservation: PASS
- fixed sidebar behavior: PASS
- GUI LCD logical mirror: PASS

## Input / Event Bridge regression

Validation ran while Live Twin polling was active.

Router OFF fallback:

- knob CW -> host volume up: PASS
- knob CCW -> host volume down: PASS
- knob press -> host mute: PASS

Router ON default Event Bridge profile:

- action `1` / `VOLUME_DOWN`: PASS
- action `2` / `VOLUME_UP`: PASS
- action `3` / `MUTE`: PASS
- mute / unmute toggle: PASS
- firmware dropped counter: `0`
- host queue drops: `0`
- sequence gaps: `0`
- sequence duplicates: `0`

LCD Volume/Mute OSD and automatic HOME restoration passed while the Event
Bridge and Live Twin polling were active.

## Safety

- no EEPROM writes
- no persistent LCD media writes
- no second firmware flash during continuation tests
- single-owner Raw HID architecture preserved
- final Input Router state restored to OFF
- final LCD state restored to HOME
- keyboard remained connected
- host audio restored after controlled tests

## DFU note

The STM32duino path can report
`dfu-util: unable to read DFU status after completion (LIBUSB_ERROR_IO)`
after `Download done.` when the keyboard resets/re-enumerates.

This validation did not accept the firmware from that message alone.
Acceptance required normal USB re-enumeration, successful `0x4D` runtime
proof, and the physical behavior tests above.

## Result

`LIVE_DIGITAL_TWIN_V1_PHYSICAL=PASS`
