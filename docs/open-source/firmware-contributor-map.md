# Firmware Contributor Map

## Goal

This file tells open-source contributors where customization belongs and where
changes require extra review.

## Safe extension surfaces

### Runtime RGB

Primary extended keymap:

```text
keyboards/yunzii/al80/keymaps/al80_rgb_probe/keymap.c
```

Validated commands:

```text
0x48 RGB runtime
0x49 Snake overlay
0x4A Creator Scene
0x4B Input Router candidate
```

### Host protocol

AL80 Studio:

```text
core/src/lib.rs
core/src/bin/al80d.rs
core/src/bin/al80ctl.rs
app/src-tauri/src/lib.rs
```

The Rust core owns packet details.

`al80d` owns the device.

Tauri talks to `al80d`, not hidraw.

### GUI

```text
app/src/main.ts
app/src/style.css
app/public/devices/al80/
```

GUI code should deal with typed capabilities and semantic actions, not raw
USB packets.

## High-risk areas

Changes here should require separate reverse engineering and physical gates:

```text
keyboards/yunzii/al80/al80_smartble.c
keyboards/yunzii/al80/al80_screen.c
keyboards/yunzii/al80/al80_battery.c
bootloader entry logic
flash layout
USB descriptors
AW20216S driver mapping
```

Do not modify them merely to make a UI feature easier.

## Adding an action to Input Router

Required steps:

1. assign a new typed action ID;
2. document exact semantics;
3. implement only the allowlisted behavior;
4. confirm it cannot enter bootloader or persist configuration;
5. compile and run firmware-size gate;
6. physically validate;
7. expose the same action in a shared host registry;
8. add UI only after firmware support is confirmed.

## Adding a Raw HID command

Before choosing an ID:

1. inspect only the AL80 command switch, not global QMK hex literals;
2. confirm no AL80 collision;
3. define request/response bytes;
4. define failure status;
5. define persistence behavior;
6. document rollback;
7. validate physically;
8. freeze the result.

## Do not do this

```text
execute arbitrary shell from manifest
send arbitrary QMK function ID
write arbitrary firmware memory
write EEPROM for normal UI settings
let multiple processes open the same hidraw
flash automatically when toggling an effect
```
