# AL80 Live Digital Twin V1

Status: build candidate.

## Scope

Live Digital Twin V1 joins five Studio concerns:

1. preserve Creator page scroll while painting/selecting;
2. pointer-orbit 3D without stealing key painting;
3. read-only firmware-backed RGB telemetry for the AL80-owned final frame;
4. host-driven logical LCD status;
5. Dashboard and Display Studio mirrors that update without full-page rerenders.

## RGB telemetry

Raw HID namespace `0x4D` is query-only.

The firmware keeps an 82 × RGB shadow buffer only for colors authored by
`rgb_matrix_indicators_advanced_user()` in this AL80 keymap. The host reads
eight LEDs per report.

The frame is marked valid only for Snake/Heart overlay, Creator Scene, or
the low-battery red safety frame.

When the AL80 overlay and Creator Scene are both inactive, native QMK RGB
continues normally, but V1 reports `frame_valid=NO` and
`source=NATIVE_UNKNOWN`. Studio must not fabricate a native frame.

## LCD mirror

V1 is a semantic host-state mirror, not arbitrary LCD pixel readback.

`al80d` records the last successfully host-driven semantic state:
`HOME`, `VOLUME`, `MUTE`, or `FEEDBACK`.

`LCD STATUS` returns that state without sending a new LCD frame.

Factory-owned or autonomous LCD content that the host did not drive is
outside V1 and must not be represented as an exact screenshot.

## Single-owner invariant

`al80d` remains the only persistent Raw HID owner. The RGB query uses the
existing `RawHidSession` transaction path and introduces no second reader.

## Creator interaction

In Studio 3D, dragging the background orbits, wheel zooms, RGB keys remain
paintable/selectable, and Re-center restores the default camera.

Full app rerenders preserve same-view workspace scroll. Creator viewport
synchronization no longer recenters on every paint rerender.

## Safety

- EEPROM writes: no
- persistent LCD media writes: no
- automatic QMK flash: no
- firmware profile persistence: no

A firmware flash is required only later for physical validation of `0x4D`,
after this build-only candidate passes.
