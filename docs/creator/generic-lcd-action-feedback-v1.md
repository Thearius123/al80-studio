# Generic LCD Action Feedback V1

## Status

Build candidate.

This milestone creates a reusable, typed, transient LCD feedback primitive in
the host stack without changing AL80 firmware.

## Existing transport reused

The physically validated extended firmware already exposes:

```text
0x40 begin
0x41 continuation
0x42 finish
```

Generic LCD Feedback V1 therefore adds no new firmware command and requires no
QMK flash.

## Native display geometry

```text
96 x 160
RGB565
row-major
30,720 bytes
```

The Rust renderer uses a built-in 5x7 glyph set and generates the frame in
memory.

## Recovered display sequence

```text
GUI_EVENT
150 ms
ADD_PIC
RGB565 frame
FINISH
```

## Typed feedback kinds

```text
PROFILE         0..99
ACTION          0..24
RGB_BRIGHTNESS  0..100
RGB_HUE         0..255
RGB_SPEED       0..255
SNAKE           ON/OFF
SCENE           ON/OFF
ROUTER          ON/OFF
```

No arbitrary title or arbitrary LCD packet is exposed by the normal GUI/CLI
surface.

## IPC

```text
LCD FEEDBACK <kind> <value>
```

Examples:

```text
LCD FEEDBACK PROFILE 2
LCD FEEDBACK ACTION 23
LCD FEEDBACK RGB_BRIGHTNESS 72
LCD FEEDBACK SNAKE ON
LCD FEEDBACK ROUTER OFF
```

## HOME arbitration

Generic feedback schedules a delayed HOME.

A generation counter prevents that delayed HOME from overwriting newer
Volume/MUTE activity.

## Important V1 boundary

Input Router 0x4B executes its actions in firmware and currently does not emit a
host event for every physical action.

V1 therefore builds the typed LCD primitive and Studio preview but does not
claim automatic firmware-action feedback.

Automatic physical-action feedback requires a later, separately validated event
bridge or a firmware-side typed renderer.

The GUI must not become a second hidraw reader.

## Safety

```text
QMK_SOURCE_MODIFIED=NO
QMK_FLASH=NO
EEPROM_WRITE=NO
PERSISTENT_LCD_MEDIA_WRITE=NO
ARBITRARY_LCD_TEXT_FROM_GUI=NO
ARBITRARY_LCD_PACKET_FROM_GUI=NO
GUI_DIRECT_HID_ACCESS=NO
SINGLE_RAW_HID_OWNER=al80d
```
