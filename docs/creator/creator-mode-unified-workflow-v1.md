# Creator Mode Unified Workflow V1

## Goal

Creator Mode already had:

```text
82-LED Keyboard Painter
saved Creator scenes
atomic 0x4A Creator Scene apply
Input Designer / 0x4B Input Router
0x4C Input Event Bridge
Automatic LCD Action Feedback
Host Profiles V2
```

V1 adds one Creator-page workflow that composes the current LED painting with
typed Input behavior.

## Creator Input sources

The unified Creator session can use:

```text
Router OFF
Current Input Designer draft
Any saved Input profile
```

The current draft is read from the existing Input Designer model. No arbitrary
keycodes or HID packets are introduced.

## Apply

```text
Apply unified workspace
  |
  + RGB core ON
  + apply current 82-LED Creator Scene
  + apply selected typed Input profile
    OR disable Input Router
  |
  v
existing Tauri allowlisted commands
  |
  v
al80d single Raw HID owner
```

Automatic LCD feedback remains owned by al80d. Creator does not duplicate LCD
streaming logic.

## Exit

```text
Exit unified workspace
  |
  + Creator Scene OFF
  + Input Router OFF
```

Normal RGB/Snake rendering becomes visible again.

## Hardware safety

```text
GUI_DIRECT_HID=NO
SINGLE_RAW_HID_OWNER=al80d
CREATOR_SCENE=VOLATILE_RAM
INPUT_ROUTER=VOLATILE_RAM
PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
ARBITRARY_KEYCODE=NO
```

## Host library persistence boundary

The current host libraries are still backed by WebView `localStorage`:

```text
al80-studio.creator-scenes.v1
al80-studio.input-profiles.v1
al80-studio.host-profiles.v1
```

That is a host-storage limitation, not keyboard persistence. A later
`Host Library Persistence V1` milestone should migrate these libraries to
Tauri-controlled application data with migration/backward compatibility.

Creator Unified Workflow V1 deliberately does not mix that storage migration
with the hardware workflow change.
