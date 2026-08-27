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
