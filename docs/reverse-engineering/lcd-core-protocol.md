# AL80 LCD Core Protocol Primitives

## Status

This document records migration of the known-good Linux volume OSD Raw HID
protocol into the reusable Rust `al80-core` library.

The Python service remains the known-good production reference during this
stage.

## Migration split

The current OSD has two responsibilities:

1. observing Linux audio state;
2. sending LCD protocol transactions to the keyboard.

Core V1 first migrates the proven keyboard protocol primitives. The Linux
audio watcher is migrated after physical A/B validation.

## Transport

The existing Rust hidraw transport is reused.

Known AL80 identity:

```text
VID 0x28E9
PID 0x30AF
Usage Page 0xFF60
Usage 0x61
```

Linux output framing:

```text
byte 0       report ID 0
bytes 1..32  vendor payload
```

LCD raw responses remain unnormalized because the currently known commands
use different ACK offsets.

## HOME

Sequence:

```text
0x40 begin
0x42 end
```

Begin embeds:

```text
A5 5A 0B 00 00 02 00
```

at payload offset 7.

ACK:

```text
response[6] == 0x55
```

## Volume / Mute

Command:

```text
0x43
```

Payload:

```text
payload[0] = 0x43
payload[1] = 0..100
payload[2] = 0 normal / 1 muted
```

ACK:

```text
response[3] == 0x55
```

The different ACK offsets are intentionally preserved.

## CLI validation interface

```text
al80-core lcd home
al80-core lcd volume <0-100>
al80-core lcd mute <0-100>
```

These commands affect only the keyboard display. They do not modify host
audio state.

## Safety

Classification:

```text
VOLATILE_DISPLAY
```

No EEPROM write, persistent LCD media upload, firmware flash, bootloader
operation, or QMK modification is implemented by this stage.

## Next

Physical validation temporarily stops the legacy Python OSD, exercises HOME,
volume and MUTE through Rust, restores HOME, and then restarts the known-good
service.

After parity passes, the `pactl` / `wpctl` event-driven behavior can move into
the broker.
