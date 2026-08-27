# Licensing

AL80 Studio uses split licensing because the host software and QMK-derived firmware have different upstream requirements.

## Host application, Rust core and original documentation

Unless a file says otherwise:

```text
MIT OR Apache-2.0
```

See:

- `LICENSE-MIT`
- `LICENSE-APACHE`

## Firmware subtree

Files under:

```text
firmware/qmk/
```

are QMK-derived firmware material and are distributed under:

```text
GPL-2.0-or-later
```

subject to more specific upstream notices in individual files.

See:

- `firmware/qmk/LICENSE-GPL-2.0`

## Third-party code

Third-party dependencies remain under their own licenses.

This repository does not relicense upstream code beyond the permissions of its original license.
