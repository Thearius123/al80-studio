# Automatic LCD Action Feedback V1

## Purpose

Automatic LCD Action Feedback V1 connects the validated Input Router Event
Bridge to the already validated typed LCD feedback renderer.

The firmware action remains authoritative. LCD rendering is host-side,
best-effort observability and presentation only.

## Policy

| Action IDs | Automatic LCD policy |
| --- | --- |
| 0 | no feedback |
| 1–3 | existing audio watcher; show actual Volume/Mute state |
| 4–20 | generic `ACTION <id>` |
| 21 | `SNAKE OFF` |
| 22 | `SNAKE ON` |
| 23 | generic `ACTION 23`; resulting toggle state is not invented |
| 24 | `SCENE OFF` |

RGB actions 15–20 deliberately render their action ID rather than inventing
an absolute RGB brightness, hue, or speed value.

## Concurrency

A generic 96×160 frame takes roughly 2.6 seconds to stream on the recovered
LCD bridge.

Therefore the event pump never renders LCD frames directly.

It performs a nonblocking dispatch to one dedicated auto-LCD worker.

Only one automatic frame is allowed in flight. Additional generic feedback
while the worker is busy is dropped rather than queued as stale UI.

Input events themselves are still consumed and logged normally.

## Volume/Mute priority

Volume and Mute actions never use generic automatic frames. The existing
Fedora audio watcher remains authoritative for the displayed percentage and
mute state.

The routed Volume/Mute event bumps `LCD_GENERATION` immediately without
waiting for the device mutex. This preempts an in-flight generic automatic
frame as soon as that frame reaches its next cancellation checkpoint.

The actual audio watcher then bumps generation again before acquiring the
device mutex and renders the real host audio state.

Automatic generic frame streaming checks the current generation before
GUI_EVENT, after the recovered 150 ms settle, and between RGB565 chunks. If
the generation changes after a bridge session has begun, the transfer still
runs the recovered bridge-finish sequence before releasing the device.

The audio watcher's delayed HOME is also generation guarded in V1. A HOME
timer created by an older Volume/Mute OSD cannot overwrite newer generic LCD
activity.

## Safety

- no QMK change;
- no EEPROM write;
- no persistent LCD media;
- no arbitrary strings from firmware;
- action IDs remain allowlisted 0–24;
- single Raw HID reader/owner remains `al80d`;
- firmware input execution remains authoritative;
- automatic LCD delivery is best effort.
