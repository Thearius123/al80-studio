# AL80 Firmware Safety Boundaries

This document defines the rules contributors should follow before modifying
AL80 Extended Firmware.

## Protected principles

### 1. Normal customization should be volatile

Prefer:

```text
RAM state
host-side profile storage
explicit runtime commands
```

Avoid persistent keyboard writes unless a feature has a dedicated design,
rollback plan and physical validation.

### 2. Never add EEPROM writes casually

AL80 Studio runtime protocols currently require:

```text
eeprom_write=NO
persistent_write=NO
```

RGB runtime adjustment must use QMK `_noeeprom` APIs.

### 3. Never hide a firmware flash behind a normal toggle

Firmware installation is an Advanced Mode operation.

Runtime controls must never silently enter the bootloader or flash QMK.

### 4. Preserve the known-good bootloader/application boundary

Build scripts enforce a firmware size safety limit derived from the recovered
factory memory layout.

Do not increase or move that boundary without a separate reverse-engineering
milestone.

### 5. Keep Raw HID commands typed

Current AL80 Studio extended commands:

```text
0x47 scan diagnostics
0x48 RGB runtime
0x49 Snake/overlay
0x4A Creator RGB Scene
0x4B Creator Input Router
```

Do not introduce a generic "execute command" or arbitrary memory write
protocol.

### 6. Single hardware owner

On Linux:

```text
AL80 -> hidraw -> al80d -> IPC clients
```

GUI, plugins and SDK code must not open hidraw directly.

### 7. Physical validation before freeze

A new firmware protocol is not "supported" because it compiles.

Required sequence:

```text
checkpoint
compile
size/collision gates
single controlled flash
protocol query
physical behavior test
regression test
rollback availability
commit/freeze
```

### 8. Preserve low-battery behavior

Creator RGB intentionally leaves the recovered low-battery red indication
above user-created scenes in renderer priority.

Input features must not disable battery safety logic.

## If a candidate fails

Do not repeatedly flash the same candidate.

Return to the frozen known-good firmware only after confirming the keyboard
returned to normal application mode.

Keep SHA256 hashes for both candidate and rollback binary.
