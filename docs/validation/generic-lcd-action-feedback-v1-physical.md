# Generic LCD Action Feedback V1 — Physical Validation

Date: 2026-08-22

## Frozen prerequisites

Studio before this milestone:

```text
63ebb8ba8ca44533b2d521357995b27052f5e41d
```

QMK Input Router / extended firmware:

```text
845378cb6e9a222c988c0ef59ffbb58b9883160e
```

Candidate artifacts:

```text
al80d 0.4.0  c9156a5592f0c153664c34e0b01ab7ca40a3d229181fb17ea476baf54376b72f
al80ctl      60a69d7a69ef96502f5ffd652b353ef6b16b0d4c4f86b1ad81366d2f0455b886
GUI          35e15050bb7332efed4e133f7db0561ee2ae47b812f4e94a3a56851652ff2c24
archive      6588c3cefdb48578da3cc711a82271abe713683d935ca6ad474691c29bf6afb1
```

No QMK flash was performed.

## Typed feedback kinds physically validated

```text
PROFILE 2
ACTION 23
RGB_BRIGHTNESS 72
RGB_HUE 180
RGB_SPEED 128
SNAKE ON
SCENE OFF
ROUTER OFF
```

Every preview rendered on the physical 96x160 LCD and returned to HOME.

## Transport

The milestone reuses the existing recovered, physically validated host bridge:

```text
0x40 begin
0x41 continuation
0x42 finish
```

The native frame is:

```text
96 x 160 RGB565
30,720 bytes
```

No persistent LCD media is written.

## HOME arbitration

Validated physically and through daemon logs:

1. generic SNAKE feedback was displayed;
2. a newer Volume event was generated with the knob;
3. Volume OSD replaced the generic feedback;
4. the old generic HOME timer did not overwrite the newer activity;
5. daemon reported:

```text
AL80D_GENERIC_LCD_HOME=SKIPPED_NEWER_ACTIVITY
```

## GUI

Validated:

- new Typed LCD feedback panel;
- typed SNAKE ON preview;
- existing Volume preview;
- existing MUTE preview;
- HOME;
- Input Designer readback regression;
- Creator Painter rendering regression;
- single Raw HID owner remained al80d.

## Important boundary

Input Router 0x4B still executes actions in firmware and does not emit a typed
host event for every physical binding.

Therefore automatic LCD feedback for every firmware-side knob/key action is not
claimed by V1.

A future event bridge must preserve the single-reader invariant through al80d.

## Final runtime

```text
al80d=0.4.0
input_router=OFF
rgb=ON
overlay=ON
creator_scene=OFF
lcd=HOME
```

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
GITHUB_PUSH=NO
```
