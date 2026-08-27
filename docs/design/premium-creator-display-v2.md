# AL80 Premium Creator + Display V2

## Goal

Make advanced keyboard creation feel approachable enough for a first-time user
while still feeling powerful to an expert.

## Product patterns

### Progressive disclosure

The normal workflow is presented before protocol detail:

```text
Create -> Refine -> Save -> Apply
```

### Device-centered editing

The keyboard itself is the canvas.

The recovered AL80 layout remains authoritative. The 3D mode is only a visual
projection of the same DOM key map; it does not invent a second layout and does
not alter LED addressing.

### Local vs device actions

Local canvas operations and host saves are visually separated from the explicit
`Apply to keyboard` action.

### Display as a creative surface

LCD is renamed `Display Studio` and begins with a realistic 96x160 digital
preview. Existing validated templates are presented first:

```text
Volume
Mute
Action
Profile / Scene
```

Artwork and animation are shown as future capability areas, not silently
enabled or claimed.

## Current hardware/API truth

The existing Creator path supports one atomic 82-color volatile scene. The
current Display path supports Volume/Mute plus typed generic feedback.

Therefore V2 does not claim:

```text
continuous RGB streaming
RGB effect layers on hardware
arbitrary LCD framebuffer from GUI
custom LCD images
GIF/video playback
persistent keyboard writes
```

## Premium interaction model

Creator:

```text
Header / status
Workflow stepper
Command center
Effect Lab
Digital Twin keyboard
Accent zones
Saved scenes
Local/device action dock
```

Display:

```text
Realistic LCD preview
Supported template map
Existing validated hardware preview controls
Future artwork/animation capability cards
```

## Safety

```text
NEW_TAURI_COMMAND=NO
NEW_DAEMON_COMMAND=NO
CORE_CHANGE=NO
QMK_CHANGE=NO
DEVICE_WRITE_DURING_BUILD=NO
EEPROM_WRITE=NO
PERSISTENT_KEYBOARD_WRITE=NO
```

The only new Creator interaction state is the local Top/3D view toggle.

## Next functional design milestones

1. Creator Effect Runtime throughput measurement.
2. Bounded live effect runtime if transport measurements permit it.
3. LCD framebuffer API discovery/validation for custom artwork.
4. Screen Composer V2 with actual pixel/image assets only after that API is
   validated.
5. Creator Project format combining RGB, effects, Inputs, LCD policy and
   capability requirements.
