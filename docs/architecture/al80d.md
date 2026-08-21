# `al80d` — Single-Owner Device Broker

## Purpose

`al80d` is the target long-running hardware owner for AL80 Studio.

The GUI, volume OSD, CLI, profiles and future extension system should not each
open the keyboard Raw HID interface independently.

Instead:

```text
                    YUNZII AL80
                         |
                       hidraw
                         |
                       al80d
                         |
        +----------------+----------------+
        |                |                |
   AL80 Studio       Volume OSD        CLI / SDK
        |                |                |
        +----------- future plugins ------+
```

## V1 responsibilities

The first daemon foundation contains:

- one persistent `Al80` owner;
- one mutex protecting every keyboard transaction;
- automatic reconnect after a failed hardware transaction;
- event-driven Linux volume monitoring;
- 50 ms normal volume coalescing;
- immediate mute/unmute rendering;
- 3 second return-to-HOME behavior;
- Unix-domain-socket IPC;
- RGB runtime commands;
- overlay commands;
- LCD HOME/volume/mute commands;
- current audio-state query;
- device telemetry query.

## IPC socket

Preferred location:

```text
$XDG_RUNTIME_DIR/al80d.sock
```

Fallback:

```text
/tmp/al80d-$USER.sock
```

V1 uses a deliberately small line-oriented protocol so no new dependency is
required while the reverse-engineered hardware interface is still evolving.

Examples:

```text
PING
STATUS
RGB ON
RGB OFF
OVERLAY STATUS
OVERLAY ON
OVERLAY OFF
LCD HOME
LCD VOLUME 42
LCD MUTE 42
AUDIO CURRENT
```

Each request receives one line beginning with either:

```text
OK
ERR
```

## Ownership model

Only `al80d` should open the AL80 Raw HID interface once migration is
complete.

The current Python OSD and direct Tauri `Al80::connect()` calls are retained
temporarily as validated rollback paths until daemon hardware validation and
GUI IPC migration pass.

## Open-source direction

The socket is an internal Core V1 boundary, not yet a stable public API.

A future public SDK should version capabilities explicitly so community
effects, LCD widgets and other extensions can determine which stock or
extended-firmware functions are available.

## Safety

The daemon foundation does not add:

- EEPROM writes;
- persistent LCD media upload;
- firmware flashing;
- bootloader entry;
- QMK source modification.

RGB, overlay and LCD OSD operations remain volatile.
