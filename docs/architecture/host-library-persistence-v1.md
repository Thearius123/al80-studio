# Host Library Persistence V1

## Goal

Creator Scenes, Input Profiles and Host Profiles must survive GUI rebuilds,
WebView storage changes and normal application restarts.

Before V1 the three libraries used only WebView `localStorage`.

V1 makes Tauri-controlled files the startup source of truth while retaining
the existing localStorage keys as a compatibility mirror.

## Libraries

The Tauri storage API is path-allowlisted:

```text
creator-scenes-v1 -> creator-scenes-v1.json
input-profiles-v1 -> input-profiles-v1.json
host-profiles-v1  -> host-profiles-v1.json
```

Arbitrary paths and arbitrary file names are not accepted.

The Input Designer draft remains localStorage-only because it is transient
editor state rather than a named host library.

## Linux storage

By default:

```text
$XDG_DATA_HOME/al80-studio/host-library-v1/
```

or, when `XDG_DATA_HOME` is unset:

```text
~/.local/share/al80-studio/host-library-v1/
```

Windows and macOS use their normal per-user application-data roots.

## Startup migration

For each library:

```text
host file exists
  -> read host file
  -> mirror into historical localStorage key
  -> run existing typed frontend parser
  -> normalize invalid entries away

host file missing
  -> read historical localStorage through existing typed parser
  -> write normalized array to host file
  -> keep localStorage mirror
```

This provides one-way migration without deleting the old WebView copy.

## Writes

Existing GUI save/delete flows still update localStorage immediately for
compatibility, then write the normalized library to the allowlisted Tauri
host file.

The keyboard is not involved.

## Safety boundary

```text
HOST_FILE_WRITE=YES
KEYBOARD_PERSISTENT_WRITE=NO
EEPROM_WRITE=NO
QMK_FLASH=NO
GUI_DIRECT_HID=NO
ARBITRARY_HOST_PATH=NO
ALLOWLISTED_LIBRARIES=3
MAX_LIBRARY_BYTES=4194304
```

Host Library Persistence V1 changes application-data persistence only.
`al80d`, Raw HID ownership and firmware protocols are unchanged.
