# Reverse Engineering

This directory is the protocol notebook for AL80 Studio.

Documents should record:

- observed packet bytes;
- direction;
- report length;
- CRC/checksum;
- ACK behavior;
- timing;
- failure behavior;
- persistence behavior;
- firmware assumptions;
- physical validation evidence.

## Extended command map

```text
0x47  scan/runtime telemetry
0x48  RGB runtime
0x49  overlay / Snake
0x4A  Creator Scene
0x4B  Input Router
0x4C  Input Event Bridge
0x4D  Live RGB telemetry candidate
```

Do not rely on undocumented magic constants in GUI code.
