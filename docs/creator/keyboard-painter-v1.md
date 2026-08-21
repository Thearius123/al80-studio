# Keyboard Painter V1

Creator RGB command `0x4A` was physically validated before this GUI integration.
The validated firmware supports 82 atomic RGB zones: 79 key LEDs and 3 accent LEDs.

Architecture:

```text
Keyboard Painter -> Tauri -> al80d -> al80-core -> Raw HID 0x4A -> AL80
```

V1 supports exact physical layout rendering, click/drag painting, selection mode,
coloring selected keys, fill/off/white tools, undo, the known WASD validation preset,
accent editing, host-side named scene save/load/delete, atomic Apply, and Exit Creator.

Saved scene definitions are host-side only. Keyboard scene state remains volatile RAM.
No EEPROM, firmware flash, persistent RGB write, or persistent LCD media write is used.

Next: Creator Input V1 for knob bindings + LCD feedback, then animated effect/timeline creation.
