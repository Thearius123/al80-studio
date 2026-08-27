# Security and Hardware Safety

AL80 Studio communicates directly with keyboard firmware and includes reverse-engineered protocol work.

## Reporting

Do not publish:

- account tokens;
- private GitHub credentials;
- secrets from environment files;
- unrelated personal USB/device identifiers;
- private filesystem information unless essential.

## Device safety model

Normal Studio behavior is designed around volatile runtime control.

Higher-risk operations include:

- firmware flashing;
- bootloader/recovery operations;
- future EEPROM writes;
- future persistent LCD media writes.

They must never be hidden behind ordinary styling/profile actions.

## Firmware testing

Before flashing:

1. compile successfully;
2. record candidate firmware SHA256 and size;
3. keep a known-good rollback image;
4. verify the board/bootloader;
5. stop competing HID owners;
6. make one controlled flash;
7. verify normal USB re-enumeration;
8. run a physical regression.

## Raw HID ownership

A second persistent reader can consume replies intended for `al80d`.

Treat the single-owner transport model as a correctness and safety boundary.
