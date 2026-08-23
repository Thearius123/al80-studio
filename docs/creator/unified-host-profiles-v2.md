# Unified Host Profiles V2

## Purpose

Host Profiles V1 stored only:

```text
RGB ON/OFF
Snake/overlay ON/OFF
```

Creator scenes and Input Designer profiles already existed as separate local
libraries.

Host Profiles V2 compose those safe host-side pieces without introducing a
firmware profile protocol.

## Profile V2 payload

A V2 profile stores:

```text
RGB state
Snake/overlay state
Creator scene snapshot OR Creator OFF
Input profile snapshot OR Input Router OFF
```

Creator colors and Input bindings are copied into the Host Profile. Applying a
profile therefore does not depend on the original Creator scene or Input
profile remaining in its separate library.

## Backward compatibility

V1 profiles remain readable.

Their historical behavior is preserved:

```text
V1 apply:
  RGB
  Snake/overlay
  preserve current Creator state
  preserve current Input Router state
```

V2 profiles explicitly control all composed fields.

## Apply path

```text
Host Profile V2
  |
  + set_rgb_core_runtime
  + set_overlay_runtime
  + apply_creator_scene / disable_creator_scene
  + apply_input_profile / disable_input_router
  |
  v
Tauri allowlisted commands
  |
  v
al80d single Raw HID owner
```

## Safety

```text
HOST_PROFILE_STORAGE=LOCALSTORAGE
FIRMWARE_PROFILE_PROTOCOL=NO
PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
ARBITRARY_KEYCODE=NO
ARBITRARY_HID=NO
GUI_DIRECT_HID=NO
SINGLE_RAW_HID_OWNER=al80d
```

This milestone changes only frontend composition logic. Core, al80d, firmware
and protocol semantics remain unchanged.
