# AL80 Volume OSD Protocol — Known-Good Linux Reference

## Purpose

This document records the behavior of the known-good Fedora volume OSD
implementation before migration into AL80 Studio Core.

It is reverse-engineering documentation, not a claim that every byte of the
vendor protocol is fully understood.

## Device identification

Known keyboard:

```text
Vendor ID:  0x28E9
Product ID: 0x30AF
```

The known-good discovery code searches AL80 hidraw interfaces and also checks
the HID report descriptor for the vendor-defined usage represented by:

```text
06 60 FF
09 61
```

## Linux Raw HID open mode

The known-good Python OSD opens the selected device with:

```text
O_RDWR | O_NONBLOCK
```

## Host report framing

Each transaction constructs a 33-byte host buffer:

```text
byte 0       report ID = 0
bytes 1..32  32-byte protocol payload
```

Before transmitting, the known-good implementation drains pending readable
reports.

It then:

1. writes the 33-byte report;
2. polls non-blocking reads;
3. accepts the first received response;
4. uses a bounded timeout;
5. records round-trip latency.

Current transaction timeout:

```text
0.5 seconds
```

Reference polling sleep:

```text
0.0004 seconds
```

## HOME sequence

Known-good HOME payload embedded in the begin command:

```text
A5 5A 0B 00 00 02 00
```

### Begin

```text
payload[0] = 0x40
payload[3] = length of HOME payload
payload[7..] = A5 5A 0B 00 00 02 00
```

The known-good begin command checks:

```text
response[6] == 0x55
```

### End

Second transaction:

```text
payload[0] = 0x42
```

It also checks:

```text
response[6] == 0x55
```

## Volume / mute OSD

Known-good payload:

```text
payload[0] = 0x43
payload[1] = volume percent
payload[2] = muted ? 1 : 0
```

Host-side volume range:

```text
0..100
```

For this command the known-good implementation checks:

```text
response[3] == 0x55
```

### Important reverse-engineering note

HOME and volume currently validate `0x55` at different response offsets.

Do **not** normalize these offsets merely because they look inconsistent.
They must remain command-specific until packet captures or controlled probes
demonstrate a more general response structure.

## Linux audio event pipeline

Audio change notification:

```text
pactl subscribe
```

Current state query:

```text
wpctl get-volume @DEFAULT_AUDIO_SINK@
```

Relevant events include sink and server changes.

## Coalescing

Normal volume changes settle for approximately:

```text
50 ms
```

Mute transitions bypass normal coalescing and are sent immediately.

## Return to HOME

After the last relevant audio change, the implementation waits:

```text
3 seconds
```

and returns the keyboard display to HOME.

## Recovery behavior

The current known-good implementation:

- reconnects when the Raw HID interface disappears;
- retries after errors;
- terminates its `pactl` child cleanly;
- attempts pending volume delivery during teardown;
- attempts to restore HOME during teardown;
- closes the hidraw file descriptor.

## Migration parity tests

The Python implementation remains a known-good reference until equivalent
physical behavior is demonstrated.

Required tests:

- normal volume increase/decrease
- rapid encoder movement
- mute
- unmute
- 0 percent
- 100 percent
- HOME restoration
- disconnect/reconnect
- bounded ACK latency
- no response stealing between features
