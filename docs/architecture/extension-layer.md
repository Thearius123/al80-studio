# Customization and Extension Layer

## Why this layer exists

AL80 Studio now has a stable single-owner hardware runtime:

```text
YUNZII AL80
     |
   al80d
     |
  IPC
     |
AL80 Studio / al80ctl / future SDK
```

The next problem is describing what a connected keyboard can safely do and
how community customizations declare their requirements.

## Capability contract V1

`al80d` exposes:

```text
CAPABILITIES
```

The response is versioned with:

```text
api=1
```

The current extended-firmware runtime advertises:

- matrix scan diagnostics;
- RGB runtime control;
- overlay/Snake support;
- LCD volume OSD;
- event-driven host audio monitoring;
- extension manifest V1.

It also explicitly advertises that normal runtime operations do not provide
EEPROM writes, QMK flashing or persistent LCD media writes.

## `al80ctl`

`al80ctl` is the first reusable command-line client for the daemon.

Examples:

```text
al80ctl status
al80ctl capabilities
al80ctl audio
al80ctl rgb on
al80ctl rgb off
al80ctl overlay status
al80ctl lcd home
al80ctl lcd volume 50
al80ctl lcd mute 50
```

This replaces ad-hoc Python socket snippets during development and gives
contributors a reproducible control surface.

## Extension manifests

V1 manifests are declarative metadata only.

They describe identity, customization kind, required firmware mode, required
capabilities, activation commands when appropriate and safety requirements.

V1 intentionally does not execute arbitrary extension code.

## Snake

The first example manifest describes Snake as an extended-firmware effect
requiring:

```text
rgb_runtime
overlay
```

The actual animation implementation remains firmware-side.

## Next architecture step

The AL80 Studio GUI should consume `CAPABILITIES` and use the extension
registry to render only compatible customization controls.

That leads to:

1. Effects page;
2. RGB customization;
3. LCD widgets;
4. profiles;
5. extension discovery/import;
6. developer tooling for creating new effects.
