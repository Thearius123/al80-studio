# Input Designer V1 — Physical Validation

Date: 2026-08-22

## Frozen prerequisites

QMK Input Router V1:

```text
845378cb6e9a222c988c0ef59ffbb58b9883160e
```

Studio baseline before Input Designer:

```text
acb931faf2a6e5cf5647393a54743bf61d259083
```

Input Designer candidate artifacts:

```text
al80d 0.3.0  1fee864238ec20f31e4b7fc109b23773ff8b38f411565f18b070881c5d5e7df7
al80ctl      229b42e69a5c386358615b1dec93fd65d99f4bfeb39401da0d8d3c2e31e49310
GUI          6967e0e7663633cdeb8a15cfaf32c13d1d74e7c66f24693686711b858a60cfc9
```

No QMK flash was performed during this milestone.

## Host integration gates

Validated:

- typed Core API for Raw HID `0x4B`;
- transactional disable → clear → set → enable profile apply;
- safe-default recovery path;
- al80d 0.3.0 typed Input IPC;
- al80ctl Input developer commands;
- Tauri typed commands;
- frontend command allowlist;
- one Raw HID owner: al80d;
- 12 binding slots;
- actions 0–24;
- host profiles stored locally;
- no arbitrary keycode;
- no shell execution;
- no GUI direct hidraw access.

## Physical GUI validation

Passed:

- al80d 0.3.0 Volume/Mute + LCD regression;
- Inputs view opened successfully;
- hardware binding read/dump rendered in the GUI;
- Fn + Snake preset applied from GUI;
- visual physical-key picker selected A;
- Hold A + knob press overrode the Always Mute rule and toggled Snake;
- releasing A restored ordinary knob Mute;
- local profile save/load;
- hardware readback reconstructed the applied custom rules;
- Disable Router activated firmware safe Volume/Mute fallback;
- Restore Defaults restored typed Volume/Mute bindings;
- final Volume/Mute + LCD OSD + HOME + Snake regression.

## Final runtime state

```text
al80d=0.3.0
input_router=OFF
safe_fallback=Volume/Mute
rgb=ON
overlay=ON
creator_scene=OFF
```

The router is intentionally left OFF after validation. Users explicitly apply
a profile from Studio when they want custom routing.

## Safety

```text
QMK_FLASH=NO
EEPROM_WRITE=NO
PERSISTENT_KEYMAP_WRITE=NO
PERSISTENT_LCD_MEDIA_WRITE=NO
ARBITRARY_KEYCODE_FROM_HOST=NO
ARBITRARY_CODE_EXECUTION=NO
GUI_DIRECT_HID_ACCESS=NO
SINGLE_RAW_HID_OWNER=al80d
```
