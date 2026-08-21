# Input Router V1 — Physical Validation

Date: 2026-08-21

## Candidate history

### Attempt 1 — failed safely

The first Input Router V1 candidate passed Raw HID protocol and negative
security tests, but the first physical Router-OFF fallback gate failed:
rotating the knob did not change host volume.

The validation stopped immediately and automatically restored the frozen
known-good Creator RGB firmware.

Root cause: the callback used QMK `tap_code()` for consumer/media actions.

### Fix2

The action path was changed to QMK `tap_code16()`.

Fix2 candidate:

```text
SHA256: b79009a6d48de8f00a50899aeb39eb5af2789a3bb61765cc00d671b0ae7b11c4
size: 48256 bytes
```

Known-good rollback:

```text
SHA256: 311cc6e5d68402a4393c28e6b89586bc95e23b0e412fd9a366553bf601e710eb
size: 47568 bytes
```

## Fix2 physical gates passed

- router starts OFF after firmware reboot;
- 0x4B protocol query;
- invalid action rejected;
- reserved flags rejected;
- invalid slot rejected;
- Router OFF Volume Down / Volume Up / Mute;
- Router ON default Volume/Mute;
- Fn/layer + knob;
- held knob-button + wheel matrix trigger;
- Left Ctrl + knob modifier trigger;
- volatile RGB brightness through QMK no-EEPROM APIs;
- al80d reconnect;
- normal Volume/Mute;
- LCD Volume/Mute OSD;
- LCD HOME;
- Snake restoration;
- no daemon session errors.

## Safety conclusion

Input Router V1 configuration is volatile RAM state.

The router boots disabled.

No EEPROM write, persistent keymap write, arbitrary host keycode execution,
arbitrary code execution, or bootloader action is exposed by Raw HID 0x4B.
