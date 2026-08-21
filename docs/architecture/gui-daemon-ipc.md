# AL80 Studio GUI ↔ `al80d` IPC

## Status

AL80 Studio's Tauri backend no longer opens the keyboard Raw HID interface
directly.

The GUI preserves its existing frontend command contract:

```text
get_device_status
set_rgb_core_runtime
```

but those commands now communicate with the single-owner `al80d` process over
its Unix-domain socket.

## Why

Before this migration, both the GUI backend and the volume OSD could open the
same Raw HID interface.

The target architecture is:

```text
YUNZII AL80
     |
   hidraw
     |
   al80d
     |
  Unix IPC
     |
AL80 Studio
```

`al80d` owns device transactions for RGB, overlay, LCD, telemetry and the
event-driven Linux volume OSD.

## Current GUI IPC commands

Dashboard telemetry uses:

```text
STATUS
```

Runtime RGB control uses:

```text
RGB ON
RGB OFF
```

The existing TypeScript frontend did not need to know that the transport
changed.

## Safety boundary

After this migration, Tauri source must not contain:

```text
Al80::connect()
use al80_core::Al80
```

This is enforced by source gates.

The application still keeps the `al80-core` crate dependency temporarily
because dependency cleanup is deliberately separated from the hardware
migration.

## Next

After physical validation proves that GUI + RGB + Snake + knob + LCD coexist
while `al80d` is the only Raw HID owner:

1. freeze the GUI IPC migration;
2. install `al80d` as the user service;
3. retire the Python OSD service;
4. expose LCD/Knob/Profile pages through IPC;
5. build the effect/plugin capability layer.
