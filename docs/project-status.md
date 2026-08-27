# Project Status

## Current public state

AL80 Studio is an alpha project.

The repository intentionally distinguishes build validation from physical validation.

## Physically validated foundations

Physical validation history exists for:

- Raw HID device broker model;
- runtime RGB control;
- Snake/custom overlay;
- Creator Scene transport;
- 82-address RGB layout;
- Input Router;
- Input Event Bridge;
- Volume/Mute/HOME LCD behavior;
- typed generic LCD feedback;
- automatic LCD action feedback;
- host library persistence.

## Current build-validated candidate: Live Digital Twin V1

Build validation passed for:

- frontend TypeScript/Vite;
- Tauri Cargo check;
- Rust core tests;
- daemon tests;
- QMK compile;
- release GUI binary;
- diff/scope/safety gates.

Firmware candidate:

```text
SHA256:
ecfeeaf8ec7d0ad71e1ed480e7d296f49da4fe259cf2be6a64c7812fccd2d46f

Size:
48828 bytes
```

GUI candidate:

```text
SHA256:
ad96f337461da5d108814cdb46fc56116bdd3dcadbe657dad40db638a2e680e6
```

Physical validation is pending because hardware was not available for the final pass.

## Truth boundaries

`0x4D` provides an 82-LED shadow frame only for AL80-authored final frames:

- Snake;
- Creator Scene;
- low-battery safety red.

Native QMK effects outside that path remain fail-closed/unavailable for exact host mirroring.

The LCD candidate reports host-driven logical state, not arbitrary pixel framebuffer readback.
