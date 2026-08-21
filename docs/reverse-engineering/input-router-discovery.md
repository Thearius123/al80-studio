# Input Router Reverse-Engineering Notes

## Recovered baseline

The AL80 exposes one rotary encoder on the recovered board definition.

The validated extended keymap previously used QMK encoder-map mode:

```text
layer 0: KC_VOLD / KC_VOLU
layer 1: KC_VOLD / KC_VOLU
layer 2: KC_VOLD / KC_VOLU
layer 3: KC_VOLD / KC_VOLU
```

The physical knob press is a matrix key:

```text
matrix (0,14)
layer-0 KC_MUTE
```

## Why V1 changes encoder architecture

Static `encoder_map` cannot express arbitrary runtime bindings received from
AL80 Studio.

Input Router V1 therefore disables `ENCODER_MAP_ENABLE` for the extended
`al80_rgb_probe` keymap and moves behavior into:

```text
encoder_update_user()
process_record_user()
```

The fallback implementation reproduces the original physical behavior before
the router is enabled:

```text
CCW -> KC_VOLD
CW  -> KC_VOLU
press -> KC_MUTE
```

## Arbitrary key + knob

`process_record_user()` tracks currently-held matrix positions.

A binding can therefore specify:

```text
trigger=MATRIX
row=X
column=Y
```

The key is still passed through normally in V1.

This is intentional: suppressing a trigger key after it has already emitted
a host key-down event is not safe. A future exclusive-trigger mode would need
a delayed dispatch state machine and its own physical validation.

## Layer / Fn + knob

Layer triggers are tested against QMK layer state.

This supports the common Creator workflow:

```text
Fn + knob -> alternate action
```

without modifying the physical Fn key itself.

## Why actions are IDs instead of keycodes

Allowlisted action IDs create a stable security and compatibility boundary.

They make it possible for:

- firmware;
- Rust core;
- daemon;
- GUI;
- extension manifests;
- documentation

to agree on one typed action model.

They also prevent a third-party manifest from requesting bootloader or
unknown QMK functionality through a generic integer.

<!-- FAILED_PHYSICAL_CANDIDATE_1 -->

## Physical validation attempt 1 — failed safely

The first Input Router V1 candidate passed:

- Raw HID `0x4B` query;
- default binding reads;
- unknown-action rejection;
- reserved-flags rejection;
- out-of-range-slot rejection.

The first host-visible physical gate then failed: while the router was disabled,
rotating the knob did not change host volume.

Validation stopped immediately and the automated rollback restored the frozen
known-good Creator RGB firmware. Normal knob behavior returned after rollback.
The failed candidate was not committed.

### Root cause identified

The callback implementation used QMK's 8-bit helper:

```c
tap_code(KC_VOLD);
tap_code(KC_VOLU);
tap_code(KC_MUTE);
```

The prior `encoder_map` path preserved QMK's full keycode representation.
Consumer/media actions require the 16-bit dispatch helper.

Input Router V1 Fix2 therefore uses:

```c
tap_code16(...)
```

inside both:

```text
al80_input_execute_action()
al80_input_default_event()
```

This correction is intentionally isolated before another physical flash.

### Validation lesson

Passing a Raw HID protocol test does not prove that the resulting host HID
action is correct. New input protocols require both protocol validation and
host-visible physical regression testing.
