# AL80 Raw HID Protocol

Status: reverse-engineered / experimental

## Device

Normal application USB identity:

- VID: `0x28E9`
- PID: `0x30AF`

Raw HID:

- Usage Page: `0xFF60`
- Usage: `0x61`
- payload size: 32 bytes
- Linux writes may include report ID `0x00`, making 33 bytes total

## Known commands

| Command | Purpose | Current classification |
|---|---|---|
| `0x40` | transfer begin / info | runtime |
| `0x41` | transfer continuation/data | runtime |
| `0x42` | transfer finish/release | runtime |
| `0x43` | volume/mute OSD | runtime |
| `0x44` | LCD UART receive bridge | runtime/read |
| `0x45` | flash reader | READ ONLY |
| `0x46` | C9 pulse | DANGEROUS / HIDDEN |
| `0x47` | matrix scan-rate telemetry | READ ONLY |
| `0x48` | RGB core runtime on/off/query | volatile |
| `0x49` | custom RGB overlay on/off/query | volatile |

## Common status

Observed success marker:

`0x55`

Observed busy/rejected marker:

`0x0F`

## Safety rules

`0x46` must not be exposed as a normal application control.

`0x45` must remain read-only.

Runtime RGB commands `0x48` and `0x49` use non-EEPROM operations.

Unknown LCD media persistence behavior must not be represented as safe
persistent storage until verified.
