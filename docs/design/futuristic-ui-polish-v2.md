# AL80 Futuristic UI Polish V2

## Device Twin

V2 replaces breakpoint-based Fit estimates with runtime geometry. It measures
the real stage and recovered keyboard box, then calculates Fit directly.

Controls:
- Top
- 3D
- Fit
- 100%
- Re-center

Top and 3D use the same recovered key / LED map.

## Truthful live mirror

Live known state shows:
- RGB core
- Snake / Overlay
- Creator Scene
- Input Router
- LCD transport
- Auto-LCD readiness

The current protocol does not expose arbitrary physical per-key RGB framebuffer
readback, and V2 never claims that it does.

The canvas becomes an exact session mirror only after AL80 Studio successfully
applies the current Creator frame. Any later local edit invalidates that exact
mirror status until the new frame is applied.

## Sidebar

Desktop navigation stays viewport-fixed/sticky while the main workspace scrolls
independently. Narrow layouts retain normal document scrolling.

## LCD

The V1 validated-now vs future/protocol-work boundary remains intact.

## Safety

CORE_EDIT=NO
TAURI_API_EDIT=NO
DAEMON_EDIT=NO
QMK_EDIT=NO
DEVICE_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
