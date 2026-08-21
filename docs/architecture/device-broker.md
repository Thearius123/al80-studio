# AL80 Device Broker Architecture

## Status

This document describes the Core V1 direction of AL80 Studio.

The project is moving from independent callers of the keyboard Raw HID
interface toward one serialized device-access layer.

## Goal

AL80 Studio is intended to become an open-source platform for:

- RGB control and custom effects
- LCD control and widgets
- volume and mute OSD
- knob behavior
- keyboard configuration
- profiles and macros
- diagnostics
- scripting and automation
- community-developed extensions
- reverse-engineering and protocol research

Hardware access must therefore not be scattered throughout the GUI.

## Core rule

AL80 Studio features do not talk directly to Raw HID.

They request a transaction from the device broker.

Conceptually:

```text
                    YUNZII AL80
                         |
                     Raw HID
                         |
                  Device Broker
                         |
       +-----------------+-----------------+
       |                 |                 |
    Telemetry            RGB              LCD
       |                 |                 |
       +------------ Profiles / UI --------+
                         |
                  Future CLI / SDK
```

## Core V1 — broker foundation

The first implementation places a `DeviceBroker` in the Tauri backend.

It serializes application-side hardware transactions with one transaction
gate.

Existing operations routed through the broker:

- device connection
- matrix scan telemetry
- RGB core status
- overlay status
- RGB runtime on/off

Connection lifetime is intentionally still short in this first stage.

The purpose of this stage is to establish the ownership boundary before
migrating additional functionality.

## Important current limitation

Core V1 foundation is **not yet system-wide exclusive ownership**.

The known-good legacy volume OSD service currently opens the AL80 Raw HID
interface directly from Python.

Until that functionality is migrated, hardware validation must account for
that second process explicitly.

Do not claim single-owner Raw HID architecture until the legacy service no
longer performs direct AL80 HID transactions.

## Target architecture

The long-term target is a persistent AL80 broker/daemon that is the single
owner of the relevant Raw HID interface.

Potential clients include:

```text
al80d / device broker
    |
    +-- AL80 Studio GUI
    +-- volume OSD
    +-- CLI
    +-- profile engine
    +-- automation
    +-- plugin / extension API
```

This allows the GUI to close without requiring every background feature to
disappear and prevents independent programs from racing for keyboard replies.

## Safety principles

Reverse-engineering and development should preserve:

- explicit device identification
- serialized transactions
- bounded read timeouts
- response validation
- known-good rollback artifacts
- backups before experimental changes
- no EEPROM writes unless explicitly intended
- no firmware flash during host-side protocol testing
- separate stock-firmware and extended-firmware capabilities
- reproducible experiment notes

## Extension direction

Custom features such as Snake should eventually be exposed as reusable
capabilities rather than one-off local patches.

The architecture should support both:

### Normal mode

No firmware development required:

- runtime RGB settings
- LCD widgets
- volume OSD
- profiles
- macros
- supported stock capabilities

### Extended mode

For contributors and advanced users:

- custom RGB effects
- new LCD scenes
- protocol experimentation
- QMK extensions
- custom commands
- development SDK
- community plugins

The GUI must eventually identify which capabilities are available on the
connected firmware instead of assuming every AL80 has the same extensions.
