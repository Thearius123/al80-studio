# AL80 Studio UI/UX Polish V1

## Product intent

AL80 Studio should read like a finished hardware-configuration application,
not a reverse-engineering console.

The visual system therefore prioritizes:

1. **Understandability** — the primary action and current state should be
   obvious before technical details.
2. **Hierarchy** — page, panel, group, control and status levels should look
   intentionally different.
3. **Safety clarity** — preview/local actions must remain visually distinct
   from actions that apply state to the keyboard.
4. **Consistency** — the same control, status and spacing language should be
   shared by Device, RGB, Creator, Inputs and Profiles.
5. **Professional density** — useful information remains available without
   turning every page into a wall of diagnostics.

## Visual language

V1 uses a restrained dark workstation aesthetic:

```text
background       near-black neutral
surface          layered graphite
primary accent   cool indigo
success          mint
warning          amber
danger           soft red
```

The accent is intentionally reserved for active navigation, primary actions,
focus and high-value Creator surfaces.

## Components polished in V1

- application background and chrome
- sidebar/navigation states
- page typography and spacing
- panels/cards
- control grids
- text, select, color and range inputs
- primary/secondary/danger buttons
- badges/status pills
- Creator unified status cells
- Effect Lab visual priority
- keyboard/key hover treatment when matching surfaces are present
- tables and dividers
- scrollbars
- responsive layouts
- keyboard focus rings
- reduced-motion support

## UX boundaries

This milestone is visual only.

```text
LOGIC_CHANGE=NO
NEW_TAURI_COMMAND=NO
NEW_DAEMON_COMMAND=NO
DEVICE_WRITE=NO
DEVICE_STREAMING=NO
KEYBOARD_PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
```

Existing functionality, source-of-truth rules, input routing, Creator Scene
transport and Host Library Persistence remain unchanged.

## Creator hierarchy

The Creator workflow should visually communicate this order:

```text
1. Paint / generate a local frame
2. Inspect and edit it
3. Choose optional input behavior
4. Apply explicitly only when desired
```

Effect Lab remains `preview only`; its primary button is visually prominent
inside the local creation workflow but does not imply a device write.

## Responsive behavior

Desktop remains the primary authoring experience. V1 also ensures that on
narrow windows:

- panels do not overflow,
- controls stack to one column,
- button rows become full-width,
- status cards remain readable,
- Creator actions do not require horizontal scrolling.

## Next design iteration

After physical/visual review, V2 may introduce markup-level information
architecture changes such as:

- explicit page headers,
- advanced diagnostics disclosure,
- persistent action/status rail in Creator,
- live value readouts beside sliders,
- iconography,
- empty-state illustrations,
- saved/unsaved state language,
- compact vs comfortable density.

Those should be based on real V1 screenshots/usage rather than guessed before
the global visual system is validated.
