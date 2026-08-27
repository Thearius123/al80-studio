# Windows support

AL80 Studio is being ported to Windows 10/11 x64 without forking the product.

Linux/Fedora remains the physically validated reference implementation.

## Windows Foundation V1

### Stage A — native compile boundaries

Stage A establishes explicit platform boundaries and native Windows CI.

It does **not** claim usable Windows hardware support yet.

Windows currently fails closed for:

- AL80 HID discovery/transport
- daemon local IPC
- Windows system audio integration

The purpose of Stage A is to prove that the shared frontend, protocol model,
daemon, CLI, and Tauri host can be compiled on a native Windows runner without
changing the known-good Linux behavior.

### Planned stages

1. Named Pipe local IPC on Windows, Unix-domain socket retained on Linux.
2. Windows HID transport for VID `28E9`, PID `30AF`, vendor usage page
   `0xFF60`, usage `0x0061`.
3. Windows Core Audio volume/mute integration.
4. Tauri Windows installer generation.
5. Physical AL80 validation on Windows.
6. Public Windows release artifacts.

## Release truth boundary

Do not call Windows production-ready until physical Windows validation passes
for HID, Live Digital Twin, Creator, LCD, Input Router/Event Bridge, host
audio, reconnect behavior, and installer lifecycle.

### Stage B candidate — Windows Named Pipe IPC

The Stage B candidate replaces the Stage A fail-closed Windows IPC stubs with
a real local Win32 Named Pipe transport.

Design properties:

- Linux keeps the existing Unix-domain socket implementation unchanged.
- Windows uses a per-user `\\.\pipe\al80d-<USERNAME>` endpoint.
- remote Named Pipe clients are rejected.
- `al80d.exe` remains the single IPC server / single HID owner.
- `al80ctl.exe` and the Tauri GUI are clients only.
- the existing line-oriented typed command protocol is preserved.
- connection attempts fail after a bounded three-second connect window.
- native `windows-latest` CI performs a real server/client `PING` / `OK PONG`
  Named Pipe round trip without requiring keyboard hardware.

Stage B is build/IPC validated only after the native Windows CI gate passes.
Physical keyboard control remains pending the Windows HID stage.

### Stage C1 candidate — Windows HID transport

Stage C1 introduces the narrow Raw HID transport boundary required for native
Windows hardware access.

Linux behavior is intentionally preserved:

- Linux discovery still uses `/sys/class/hidraw`.
- Linux opens the same `/dev/hidrawN` node with the existing nonblocking flags.
- the existing `RawHidSession` remains the single I/O worker.
- `al80d` remains the only long-lived device owner.
- GUI and `al80ctl` still communicate only through local IPC.

Windows behavior:

- `hidapi` is a Windows-only dependency using its `windows-native` backend.
- discovery filters VID `0x28E9`, PID `0x30AF`, usage page `0xFF60`,
  usage `0x0061`.
- the selected HID path is preserved exactly for open-by-path.
- writes keep the existing 33-byte framing: report ID `0x00` followed by the
  32-byte AL80 protocol payload.
- Windows `read_timeout` is normalized back into that same internal
  report-ID-prefixed framing so the established Raw HID demultiplexer and
  Input Event Bridge do not gain a second reader.

The native Windows CI gate validates compilation plus framing contracts without
requiring keyboard hardware.

Stage C1 is **not physical Windows validation**. No production-ready Windows
HID claim is made until an actual AL80 is attached to a Windows host and
read-only discovery/status plus controlled volatile protocol transactions pass.

### Stage C2 candidate — read-only enumeration and packaging preview

Stage C2 adds a Windows-only HID enumeration diagnostic that uses the exact
shared C1 VID/PID/usage filter but deliberately never opens the interface and
never starts a Raw HID session.

The native Windows CI now validates:

- read-only HID enumeration through the Windows `hidapi` backend,
- the enumeration path returns a deterministic match count even when a hosted
  runner has no AL80 attached,
- the diagnostic binary contains no device-open or protocol-write path,
- `al80d.exe` and `al80ctl.exe` build in Windows release mode,
- the Tauri application builds in production mode with `--no-bundle`,
- a preview artifact contains the Windows executables and SHA-256 manifest.

This preview artifact is **not an installer or release**. It does not prove
Windows Core Audio, startup integration, physical AL80 I/O, or end-to-end
Windows product behavior.

Physical Windows HID remains pending until the same read-only enumerator is run
on an actual Windows machine with the AL80 attached. Controlled volatile
protocol transactions come only after that read-only physical gate.

### Stage D1 candidate — Windows Core Audio backend foundation

Stage D1 replaces runtime-failing Linux audio-command paths on Windows with a
native Core Audio backend while preserving the daemon audio contract.

The Windows backend obtains the default render endpoint through
`IMMDeviceEnumerator`, activates `IAudioEndpointVolume`, reads the actual
master-volume scalar and mute state, normalizes them into the existing
`VolumeState`, and feeds changes into the established `run_audio_session`
debounce plus LCD generation/HOME-restoration path.

The Windows watcher periodically rebinds the default render endpoint so output
device changes do not permanently pin the daemon to a stale endpoint. GUI and
CLI audio access remain behind daemon IPC.

Linux retains `wpctl get-volume` and `pactl subscribe`.

Stage D1 native Windows CI is hardware-free. Physical Windows host-audio
behavior remains pending until volume/mute changes and LCD feedback are tested
on an actual Windows machine.

Stage D1 does not change HID ownership, firmware, EEPROM, persistent LCD state,
or installer/release status.
