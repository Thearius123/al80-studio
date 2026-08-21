# Linux Installation

## Runtime architecture

AL80 Studio uses `al80d` as the single long-running owner of the AL80 Raw HID
interface.

```text
YUNZII AL80
     |
   hidraw
     |
   al80d
     |
  Unix IPC
     |
+----+-------------------+
|                        |
AL80 Studio          Volume/LCD watcher
```

The GUI itself does not open the keyboard Raw HID interface.

## User service

The repository ships:

```text
packaging/systemd/al80d.service
```

A local user installation places:

```text
al80d binary:
~/.local/bin/al80d

systemd unit:
~/.config/systemd/user/al80d.service
```

The service is enabled for the user's normal systemd session.

## Runtime socket

The daemon exposes:

```text
$XDG_RUNTIME_DIR/al80d.sock
```

with a `/tmp` fallback when no runtime directory is available.

## Linux audio integration

`al80d` uses:

```text
pactl subscribe
wpctl get-volume @DEFAULT_AUDIO_SINK@
```

to observe host volume and mute state.

The keyboard LCD shows changes and returns HOME after the idle interval.

## Legacy Python OSD

During development the project used:

```text
al80-volume-osd.service
```

That implementation remains useful as a rollback/reference while migration
matures, but it must not run simultaneously with `al80d` because both would
attempt direct Raw HID ownership.

## Development install

From a built checkout:

```text
cargo build --release --manifest-path core/Cargo.toml
install -Dm755 core/target/release/al80d ~/.local/bin/al80d
install -Dm644 packaging/systemd/al80d.service \
  ~/.config/systemd/user/al80d.service

systemctl --user daemon-reload
systemctl --user enable --now al80d.service
```

## Diagnostics

```text
systemctl --user status al80d.service
journalctl --user -u al80d.service
```

The IPC service can be queried by AL80 Studio and future CLI/SDK clients.

## Safety

Normal daemon operation provides volatile runtime control only.

This installation path does not itself perform:

- QMK flashing
- bootloader entry
- EEPROM writes
- persistent LCD media upload
