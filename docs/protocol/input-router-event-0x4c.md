# Input Router Event Bridge V1 — Host Demux Contract

This document accompanies the synthetic host demultiplexer for protocol `0x4C`.

## Frozen wire frame

```text
byte[0]  = 0x4C
byte[1]  = 0xE1
byte[2]  = 1
byte[3]  = sequence low
byte[4]  = sequence high
byte[5]  = event: 1 CCW, 2 CW, 3 PRESS
byte[6]  = matched slot: 0..11
byte[7]  = trigger: 0 NONE, 1 LAYER, 2 MATRIX, 3 MODS
byte[8]  = trigger a
byte[9]  = trigger b
byte[10] = action: 0..24
byte[11] = flags = 0
byte[12] = firmware dropped-event counter low
byte[13] = firmware dropped-event counter high
byte[14] = router enabled = 1
byte[15..31] = zero
```

## Host rule

The host may never assume that the first Raw HID report read after a write is
the response to that write.

An exact `0x4C / 0xE1` report is classified as an unsolicited typed event.
Everything else is a response candidate and must still match the pending
request namespace.

The live transport is **not** changed by this synthetic milestone.

## Synthetic validation

The pure Rust model tests:

1. response only;
2. event only;
3. event before response;
4. several events before response;
5. response then event;
6. malformed event rejection;
7. sequence wrap `65535 -> 0`;
8. non-fatal sequence gap;
9. host event-queue overflow while response still completes;
10. disconnect failing a pending request and resetting transient state.

Additional guards test wrong response namespaces and reserved fields.

## Event queue

The synthetic host queue is bounded to 8 events.

A full event queue drops telemetry only. It must never prevent a solicited
response from completing.

## Boundary of this milestone

```text
LIVE_HID_TRANSPORT_CHANGED=NO
LIVE_AL80D_INSTALLED=NO
LIVE_DEVICE_WRITE=NO
QMK_SOURCE_CHANGED=NO
QMK_FLASH=NO
```

The next milestone integrates this classifier into the real single-reader host
transport while preserving `al80d` as the only Raw HID owner.

## Live host transport candidate

The build-only live transport candidate replaces direct transaction-owned reads
with one dedicated Raw HID I/O worker.

```text
Al80 synchronous API
        |
        v
RawHidSession command channel
        |
        v
one worker owns one hidraw File
        |
        +---- writes serialized requests
        |
        +---- continuously reads all reports
                  |
                  +-- 0x4C/E1 -> typed event queue
                  |
                  +-- matching namespace -> pending response
                  |
                  +-- other response -> observability/drop
```

The public synchronous `Al80` methods remain synchronous. Existing callers do
not open another device handle and do not read Raw HID directly.

The worker reads even while there is no command in flight, which is required
for physical Input Router events to reach the host immediately.

This milestone is build-only:

```text
RUNTIME_INSTALL=NO
AL80D_SERVICE_RESTART=NO
DEVICE_WRITE=NO
QMK_SOURCE_MODIFIED=NO
QMK_FLASH=NO
```

## Daemon Event Pump + Observability V1

The host candidate now includes a daemon-side consumer for the typed event
queue exposed by `RawHidSession`.

The daemon polls through `DeviceOwner::operation`, preserving the existing
single-device ownership and reconnect policy.

```text
QMK 0x4C/E1
   |
RawHidSession persistent reader
   |
bounded typed queue
   |
al80d event pump
   |
   +-- typed AL80D_INPUT_EVENT logs
   +-- INPUT EVENTS IPC observability
   +-- no LCD automation yet
```

### Build-stage capability boundary

```text
input_event_bridge_host=YES
input_event_firmware=NO
input_event_auto_lcd=NO
```

This distinction is intentional. Host support must not imply that the frozen
firmware already emits `0x4C/E1` events.

### LCD policy

V1 daemon pump is `OBSERVE_ONLY`.

Volume/Mute continues to use the existing audio watcher and generic LCD
generation guard. Event-driven automatic LCD feedback is a later layer after
the firmware event emitter and physical bridge are validated.

### Poll health

If the persistent Raw HID worker records a terminal I/O error and the event
queue is empty, `pop_input_event()` surfaces the error through
`DeviceOwner::operation`, allowing the existing reconnect-and-retry policy to
replace the stale device session.

