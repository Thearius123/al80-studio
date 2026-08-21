# GUI Customization V1

## Purpose

This milestone turns AL80 Studio from a hardware dashboard into the first
capability-aware customization interface.

## Pages

### Dashboard

Shows:

- connection;
- Raw HID devnode owned by `al80d`;
- matrix scan rate;
- RGB state;
- Snake/overlay state;
- single-owner architecture.

### Effects

The first effect card is Snake.

Snake is rendered as available only when `CAPABILITIES` reports:

```text
firmware=EXTENDED
overlay=YES
rgb_runtime=YES
```

The GUI can enable and disable it through:

```text
OVERLAY ON
OVERLAY OFF
```

via Tauri → `al80d` IPC.

### RGB

Provides runtime RGB core ON/OFF.

Future work includes color, brightness, speed and effect parameters.

### LCD

Provides safe volatile previews:

- 25%;
- 50%;
- 75%;
- 100%;
- MUTE;
- HOME.

These controls do not modify host audio state.

### Profiles

The page exists but remains disabled while the daemon reports:

```text
profiles=NO
```

This is deliberate capability gating.

### Diagnostics

Shows the live daemon capability contract and safety boundaries.

## Safety

This GUI milestone does not add:

- EEPROM writes;
- QMK flashing;
- persistent LCD media writes;
- arbitrary extension code execution.

## Extension direction

The Effects page is the beginning of the extension-driven UI.

Future versions should load validated manifests from the extension registry
rather than having effect cards permanently compiled into frontend code.
