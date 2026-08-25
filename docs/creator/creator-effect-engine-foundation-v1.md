# Creator Effect Engine Foundation V1

## Purpose

AL80 Studio now has a typed host-side effect engine that can render
deterministic 82-LED frames independently from the keyboard transport.

This is deliberately a foundation milestone.

It does **not** add continuous device streaming and does not claim a new
firmware capability.

## Built-in V1 registry

```text
solid
pulse
comet
snake
```

Each effect is a typed registry entry with a stable ID and description.

The renderer accepts:

```text
effect
primary color
secondary color
speed 1..10
tail length 1..32
phase 0..1
LED order
LED count
```

and returns one validated array of RGB hex colors.

## Creator integration

The Creator page exposes an `Effect Lab · preview only` panel.

`Render preview frame`:

```text
typed EffectSpec
  -> creator-effects.ts
  -> deterministic 82-LED frame
  -> existing local Creator Painter buffer
```

The generated frame may then be edited like any other painting.

The preview action itself does not invoke Tauri and does not write to the
keyboard.

## Why preview-only first

The physically validated Creator Scene protocol `0x4A` is an atomic static
scene transport.

Continuous host animation has different timing, queueing and ownership
requirements. Those must be measured and designed rather than inferred.

V1 therefore separates:

```text
effect definition/rendering
from
device transport/runtime scheduling
```

## Safety

```text
EFFECT_ENGINE_LOCATION=HOST
GUI_DIRECT_HID=NO
NEW_TAURI_COMMAND=NO
NEW_DAEMON_COMMAND=NO
DEVICE_STREAMING=NO
DEVICE_WRITE_FROM_PREVIEW=NO
KEYBOARD_PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
```

## Next

Measure safe Creator Scene transport throughput and then design a bounded
Effect Runtime V1 with explicit frame-rate limits, cancellation, single-owner
serialization and fallback to static scenes when streaming is not appropriate.
