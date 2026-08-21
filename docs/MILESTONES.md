# AL80 Studio Milestones

## M1 — Real Raw HID telemetry

Status: PASS

Validated on a physical YUNZII AL80:

- Rust release build
- Linux hidraw discovery
- VID/PID validation
- Usage Page `0xFF60`
- Usage `0x61`
- Raw HID open
- command `0x47`
- status `0x55`
- matrix scan approximately 1471–1472 Hz
- no persistent device writes

## M2 — Reusable core + CLI

Status: PASS

Target:

- reusable Rust library
- `status`
- `scan`
- RGB query/runtime control
- custom overlay query/runtime control
- no firmware flashing
- no persistent device writes

Runtime acceptance for M2 uses query-only commands.
