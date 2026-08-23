# Input Designer Live Integration V1

## Status

Build candidate on top of frozen Automatic LCD Action Feedback V1.

## Existing hardware path

Input Designer V1 already applied real volatile hardware bindings:

```text
Input Designer
  -> Tauri apply_input_profile
  -> al80d INPUT APPLY
  -> core transactional Input Router apply
  -> firmware 0x4B volatile binding table
```

It also already supported INPUT STATUS, INPUT DUMP, INPUT OFF and INPUT DEFAULTS.

## Added in this milestone

Studio now consumes:

```text
input_event_bridge_host
input_event_firmware
input_event_auto_lcd
```

and the process-local observability snapshot:

```text
INPUT EVENTS
```

The Input Designer exposes event bridge readiness, received/consumed counts,
last action, LCD preemptions and LCD errors.

Counters are process-local observability and may reset when al80d restarts.

## Automatic LCD path

```text
physical knob
  -> Input Router firmware action
  -> typed 0x4C event
  -> al80d single Raw HID reader
  -> automatic LCD policy
```

Volume/Mute use actual Fedora audio state. Typed non-audio actions use the
automatic generic LCD worker. Generic transfers are cancellable so newer audio
feedback can take priority.

## Safety

```text
GUI_DIRECT_HID_ACCESS=NO
SINGLE_RAW_HID_OWNER=al80d
INPUT_BINDINGS=VOLATILE_RAM_ONLY
PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
ARBITRARY_SHELL=NO
```

This milestone does not change firmware, al80d, core Input Router semantics,
or the 0x4C event protocol.
