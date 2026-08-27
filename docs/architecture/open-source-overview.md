# Open-Source Architecture Overview

## Firmware

Responsibilities:

- keyboard-local RGB state;
- Creator Scene;
- overlay/Snake;
- Input Router;
- Event Bridge;
- hardware-local safety behavior;
- typed Raw HID commands.

## Rust core

Responsibilities:

- HID discovery;
- packet framing;
- transactions;
- timeouts;
- CRC/protocol handling;
- reconnect behavior;
- typed device operations.

## al80d

`al80d` is the single persistent hardware owner.

It exposes bounded IPC to GUI/CLI clients.

## Tauri

Tauri maps daemon IPC into typed frontend commands.

It must not become a second HID owner.

## Frontend

Responsibilities:

- presentation;
- local drafts;
- Creator editing;
- host libraries;
- Digital Twin visualization;
- explicit Save vs Apply workflow.

## Provenance

Future UI should distinguish:

- Draft;
- Host;
- Device;
- Firmware;
- Live-known;
- Unavailable.

## Persistence

The architecture deliberately separates:

- host library persistence;
- volatile runtime state;
- firmware flash state.

Host persistence is not keyboard persistence.
