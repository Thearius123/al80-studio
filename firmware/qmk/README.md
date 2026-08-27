# AL80 QMK Firmware Snapshot

This directory contains the AL80-specific QMK source needed to reproduce the firmware used by AL80 Studio.

It is **not** a complete QMK checkout.

## Setup

```bash
qmk setup qmk/qmk_firmware
```

Copy the AL80 subtree:

```bash
rsync -a \
  firmware/qmk/keyboards/yunzii/al80/ \
  "$HOME/qmk_firmware/keyboards/yunzii/al80/"
```

Compile:

```bash
cd "$HOME/qmk_firmware"
qmk compile -kb yunzii/al80 -km al80_rgb_probe
```

## Flashing

Flashing is an advanced operation.

Before flashing:

- verify the keyboard target;
- keep a known-good rollback image;
- record SHA256 and binary size;
- understand bootloader/recovery.

## License

QMK-derived material in this subtree is GPL-2.0-or-later, subject to upstream notices.

## Snapshot fidelity

The source under `firmware/qmk/keyboards/yunzii/al80/` is intentionally
preserved as an exact source snapshot of the AL80 QMK candidate used by the
project.

Because the upstream/recovered keymap source contains some historical trailing
whitespace, repository publication checks do **not** rewrite that firmware
snapshot solely for formatting. Project-authored host code and documentation
remain subject to strict whitespace checks.

This keeps the public firmware source reproducible instead of silently changing
it during publication.
