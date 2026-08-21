# Input Router Protocol V1 — Raw HID `0x4B`

## Purpose

Input Router V1 turns the AL80 knob into a typed, runtime-configurable input
surface while preserving a deterministic safe fallback.

The router is designed for AL80 Studio Creator Mode and for future community
extensions.

It does **not** accept arbitrary QMK keycodes from the host.

## Safety model

Configuration exists only in firmware RAM.

There are no:

- EEPROM writes;
- persistent keymap writes;
- bootloader actions;
- arbitrary function pointers;
- arbitrary Raw HID command execution.

On keyboard reboot the router returns to disabled state and the knob behaves
like the validated default:

```text
CCW   -> Volume Down
CW    -> Volume Up
Press -> Mute
```

## Command

```text
0x4B
```

### Operations

```text
0 QUERY
1 DISABLE
2 ENABLE
3 CLEAR_BINDINGS
4 SET_BINDING
5 GET_BINDING
6 RESTORE_DEFAULT_BINDINGS
```

## Binding

A V1 binding is:

```text
event
trigger_kind
trigger_a
trigger_b
action
flags
```

Maximum V1 slots:

```text
12
```

### Input events

```text
1 KNOB_CCW
2 KNOB_CW
3 KNOB_PRESS
```

### Trigger kinds

```text
0 NONE
1 LAYER
2 MATRIX_KEY
3 MODIFIERS
```

`NONE` is the base mapping.

Specific triggers are evaluated before base bindings so that, for example,
Fn+knob can override the ordinary volume binding.

### Matrix trigger

```text
trigger_a = matrix row
trigger_b = matrix column
```

V1 observes the trigger key while preserving its normal key behavior.

Exclusive/suppressed trigger keys are intentionally not implemented in V1
because safely delaying or suppressing ordinary key output requires a
separate, explicitly validated input model.

### Modifier trigger

`trigger_a` is the QMK modifier bitmask.

## Allowlisted actions

```text
0  NONE
1  VOLUME_DOWN
2  VOLUME_UP
3  MUTE
4  MEDIA_PREVIOUS
5  MEDIA_NEXT
6  MEDIA_PLAY_PAUSE
7  BRIGHTNESS_DOWN
8  BRIGHTNESS_UP
9  LEFT
10 RIGHT
11 UP
12 DOWN
13 PAGE_UP
14 PAGE_DOWN
15 RGB_VALUE_DOWN
16 RGB_VALUE_UP
17 RGB_HUE_DOWN
18 RGB_HUE_UP
19 RGB_SPEED_DOWN
20 RGB_SPEED_UP
21 SNAKE_OFF
22 SNAKE_ON
23 SNAKE_TOGGLE
24 CREATOR_SCENE_OFF
```

RGB adjustments use QMK `*_noeeprom()` APIs only.

## Knob press

The recovered physical knob switch is:

```text
matrix row 0
matrix column 14
layer-0 keycode KC_MUTE
```

Only that physical matrix position is intercepted by Input Router V1.
Other `KC_MUTE` keys in the keymap remain ordinary keys.

## Future host actions

Profiles, arbitrary app commands and rich LCD labels are deliberately not
encoded as arbitrary firmware actions.

A later host-action channel will expose a typed event ID to `al80d`, which can
map it to safe host-side capabilities.

That separation prevents third-party manifests from turning a knob binding
into arbitrary code execution.

## QMK keycode dispatch width

Input Router actions that emit QMK keycodes use `tap_code16()`, not
`tap_code()`.

This is required for consumer/media/system actions such as Volume and Mute and
keeps a consistent typed dispatch path for the router.

The requirement was discovered during the first physical V1 validation:
protocol `0x4B` worked, but host volume did not change until the keycode
dispatch width was corrected.
