# Contributing to AL80 Studio

AL80 Studio is both a desktop application and a hardware reverse-engineering project. Small-looking changes can affect a real keyboard, so contributions follow stricter gates than a normal UI application.

## Ground rules

### Preserve the single-owner HID model

`al80d` is the persistent Raw HID owner.

Do not add a GUI loop, helper daemon, extension, or background process that independently reads the same AL80 hidraw interface.

### Be exact about validation level

Use precise language:

- implemented;
- build validated;
- physically validated.

Do not upgrade status without evidence.

### Prefer volatile operations

User-facing configuration should be runtime/volatile by default.

EEPROM, persistent LCD media, bootloader operations, and firmware flashing require explicit safety design and recovery notes.

### Typed interfaces over arbitrary execution

Extensions and Creator features should use bounded typed data.

Do not expose arbitrary host shell execution or arbitrary raw firmware writes as convenience APIs.

## Development checks

```bash
cd app
npm ci
npm run build
cd ..

cargo test --manifest-path core/Cargo.toml
cargo check --manifest-path app/src-tauri/Cargo.toml
```

Firmware changes:

```bash
cd ~/qmk_firmware
qmk compile -kb yunzii/al80 -km al80_rgb_probe
```

Do not flash simply to satisfy code review.

## Firmware / protocol PRs

Include:

- command namespace;
- request bytes;
- response bytes;
- ACK/error behavior;
- persistence behavior;
- expected runtime state changes;
- binary size;
- rollback/recovery plan;
- physical validation evidence when available.

## UI PRs

The UI must not invent hardware state.

Distinguish:

- draft;
- host-known;
- device-known;
- firmware-known;
- inferred;
- unavailable.

## Formatting

```bash
cargo fmt --manifest-path core/Cargo.toml
cargo fmt --manifest-path app/src-tauri/Cargo.toml
git diff --check
```

And:

```bash
cd app
npm run build
```

## Commit examples

```text
feat: add typed creator effect frame
fix: preserve creator workspace scroll
docs: document raw hid 0x4d telemetry
test: harden input event demultiplexing
```

## PR checklist

- [ ] No second persistent Raw HID reader.
- [ ] Protocol changes documented.
- [ ] No silent EEPROM/persistent writes.
- [ ] Frontend build passes.
- [ ] Rust tests/checks pass.
- [ ] Firmware compiles if changed.
- [ ] Hardware claims match evidence.
- [ ] UI changes include screenshots when useful.
- [ ] Risky hardware changes include rollback notes.
