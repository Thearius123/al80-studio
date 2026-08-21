# al80-core

Reusable low-level communication library and CLI for AL80 Studio.

## Current platform

Linux / hidraw

## Device discovery

The AL80 is matched using:

- VID `0x28E9`
- PID `0x30AF`
- Raw HID Usage Page `0xFF60`
- Raw HID Usage `0x61`

## CLI

```text
al80-core status
al80-core scan

al80-core rgb status
al80-core rgb on
al80-core rgb off

al80-core overlay status
al80-core overlay on
al80-core overlay off

al80-core help
al80-core version
```

## Safety classification

Query-only commands:

- `status`
- `scan`
- `rgb status`
- `overlay status`

Volatile runtime operations:

- `rgb on`
- `rgb off`
- `overlay on`
- `overlay off`

The runtime RGB/overlay operations use firmware commands designed not
to write RGB EEPROM state.

al80-core currently implements no:

- firmware flashing
- bootloader commands
- persistent LCD media writes
- device flash writes
- dangerous experimental hardware commands
