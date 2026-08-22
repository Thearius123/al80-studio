# Input Designer V1

## Purpose

Input Designer is the normal-user interface for the physically validated AL80
Input Router V1 firmware protocol (`0x4B`).

Users configure the rotary knob visually instead of writing Raw HID packets,
action IDs, or matrix coordinates by hand.

The runtime path is:

```text
Input Designer
    |
Tauri typed commands
    |
al80d -- single Raw HID owner
    |
al80-core typed Input Router API
    |
Raw HID 0x4B
    |
YUNZII AL80
```

The GUI never opens `/dev/hidraw*`.

## What V1 can bind

Input events:

```text
Knob CCW
Knob CW
Knob press
```

Trigger classes:

```text
Always / NONE
Layer / Fn
Physical matrix key held
Modifier mask held
```

The firmware provides 12 binding slots.

Specific triggers are evaluated before `NONE`/Always bindings. Within the same
specificity class, lower slot numbers win. Input Designer therefore exposes
rule ordering as priority rather than hiding firmware semantics.

## Action registry

The GUI reads:

```text
app/public/devices/al80/input-actions.json
```

That registry is presentation metadata for the exact firmware allowlist. It
contains action IDs `0..24`; it cannot create new firmware behavior merely by
adding JSON.

A new action requires all of the following before it may appear as supported:

1. choose a typed action ID in QMK;
2. implement only the intended allowlisted behavior;
3. preserve EEPROM and bootloader boundaries;
4. compile and run firmware-size/safety gates;
5. physically validate the action;
6. freeze QMK evidence;
7. add the same typed action to `al80-core`;
8. update this registry and UI only after validation.

Do not add a generic arbitrary QMK keycode action.

## Physical-key trigger picker

Input Designer reuses the recovered physical layout:

```text
app/public/devices/al80/layout.json
```

The user clicks a visible AL80 key. Studio converts that key to the validated
matrix row/column internally.

Normal users should not need to know that the knob press is matrix `(0,14)` or
manually type coordinates.

The matrix is still validated by firmware when the profile is applied.

## Transactional profile application

The host never writes bindings one-by-one while leaving a partially active
router.

`al80-core::input_router_apply()` performs:

```text
disable router
    |
clear binding table
    |
write every typed binding
    |
enable router
```

If any write fails:

```text
best-effort disable
restore safe defaults
return error
```

The router remains disabled after a failed application. The firmware fallback
still provides normal Volume Down / Volume Up / Mute behavior.

## Volatile keyboard state vs local profiles

The keyboard-side Input Router table is RAM-only.

Normal Input Designer operations do not write EEPROM or a persistent keymap.
After keyboard reboot the router starts OFF and the recovered safe knob
fallback remains available.

Named Input Designer profiles are stored on the host in local storage:

```text
al80-studio.input-profiles.v1
```

This intentionally separates convenient persistence from keyboard flash or
EEPROM persistence.

## Presets

V1 includes reproducible starting points for:

```text
Default Volume
Fn + Snake
Ctrl + Snake
Knob button + wheel
RGB brightness
Media scrub
Page navigation
```

Presets edit the local draft only. Nothing reaches the keyboard until the user
chooses **Apply profile**.

## Read-back

`INPUT DUMP` reads the current 12 firmware slots through al80d and converts the
non-empty slots back into the visual rule model.

This is important for an open-source tool: the UI does not assume its local
copy is authoritative when another typed client has changed volatile state.

## Safety boundary

Normal Input Designer V1 intentionally has no support for:

```text
arbitrary shell commands
arbitrary Rust/JavaScript extension execution
arbitrary QMK keycodes supplied by the host
arbitrary firmware function IDs
EEPROM writes
persistent keymap writes
bootloader entry
QMK flashing
raw memory access
GUI direct hidraw access
```

Firmware installation remains a separate Advanced Mode workflow.

## Contributor map

When modifying Input Designer:

### Host-only visual changes

Normally touch:

```text
app/src/input-designer.ts
app/src/style.css
app/public/devices/al80/input-actions.json
```

### Tauri typed boundary

```text
app/src-tauri/src/lib.rs
```

Tauri validates the public GUI request and forwards a semantic IPC command.
It must not open Raw HID.

### Daemon IPC

```text
core/src/bin/al80d.rs
```

`al80d` is the single hardware owner. New IPC commands must stay typed and
bounded.

### Hardware protocol implementation

```text
core/src/lib.rs
```

This is where packet layouts, status bytes, ACK checks, transaction sequencing,
and protocol limits belong.

### Firmware

```text
keyboards/yunzii/al80/keymaps/al80_rgb_probe/keymap.c
```

Do not modify firmware merely to simplify the GUI. Firmware changes require a
separate checkpoint/build/flash/physical-validation milestone.

## V1 build acceptance

Before host runtime installation:

```text
exact frozen Studio/QMK heads
git-clean baseline
typed Core compile
al80d compile
al80ctl compile
Tauri compile
frontend TypeScript build
release build
source-scope gate
installed-runtime hash unchanged
QMK worktree/head unchanged
no device write
```

Only after those gates pass should the new daemon/UI be installed for a live
physical test.
