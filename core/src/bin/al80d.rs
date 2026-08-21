use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use al80_core::Al80;

const SETTLE: Duration = Duration::from_millis(50);
const HOME_IDLE: Duration = Duration::from_secs(3);
const RECONNECT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VolumeState {
    percent: u8,
    muted: bool,
}

struct DeviceOwner {
    device: Option<Al80>,
}

impl DeviceOwner {
    fn new() -> Self {
        Self { device: None }
    }

    fn ensure_connected(&mut self) -> Result<&mut Al80, String> {
        if self.device.is_none() {
            let device = Al80::connect()?;
            println!(
                "AL80D_DEVICE_CONNECTED={}",
                device.device_info().devnode.display()
            );
            self.device = Some(device);
        }

        self.device
            .as_mut()
            .ok_or_else(|| "device owner has no device".to_string())
    }

    fn operation<T>(
        &mut self,
        mut f: impl FnMut(&mut Al80) -> Result<T, String>,
    ) -> Result<T, String> {
        let first_result = {
            let device = self.ensure_connected()?;
            f(device)
        };

        match first_result {
            Ok(value) => Ok(value),

            Err(first_error) => {
                // A long-lived Linux hidraw file descriptor can become stale
                // after USB reset, suspend/resume or device re-enumeration.
                //
                // Existing daemon operations are explicit idempotent runtime
                // controls or reads, so one reconnect-and-retry is safe.
                self.device = None;

                eprintln!(
                    "AL80D_TRANSACTION_RETRY=YES FIRST_ERROR={first_error}"
                );

                let retry_result = {
                    let device = self.ensure_connected().map_err(
                        |reconnect_error| {
                            format!(
                                concat!(
                                    "AL80 reconnect failed after transaction ",
                                    "error: first={}; reconnect={}"
                                ),
                                first_error,
                                reconnect_error
                            )
                        },
                    )?;

                    f(device)
                };

                match retry_result {
                    Ok(value) => {
                        println!("AL80D_TRANSACTION_RECOVERY=PASS");
                        Ok(value)
                    }

                    Err(retry_error) => {
                        self.device = None;

                        Err(format!(
                            concat!(
                                "AL80 transaction failed after reconnect: ",
                                "first={}; retry={}"
                            ),
                            first_error,
                            retry_error
                        ))
                    }
                }
            }
        }
    }
}

type SharedDevice = Arc<Mutex<DeviceOwner>>;

fn lock_device(
    shared: &SharedDevice,
) -> Result<MutexGuard<'_, DeviceOwner>, String> {
    shared
        .lock()
        .map_err(|_| "AL80D device mutex poisoned".to_string())
}

fn socket_path() -> PathBuf {
    if let Ok(runtime) = env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("al80d.sock");
    }

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    PathBuf::from(format!("/tmp/al80d-{user}.sock"))
}

fn read_volume() -> Result<VolumeState, String> {
    let output = Command::new("/usr/bin/wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .map_err(|e| format!("wpctl execution failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "wpctl failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let mut percent = None;

    for token in raw.split_whitespace() {
        if let Ok(value) = token.parse::<f64>() {
            let value = (value * 100.0).round().clamp(0.0, 100.0);
            percent = Some(value as u8);
            break;
        }
    }

    let percent = percent.ok_or_else(|| {
        format!("cannot parse wpctl output: {}", raw.trim())
    })?;

    let muted = raw.to_ascii_uppercase().contains("[MUTED]");

    Ok(VolumeState { percent, muted })
}

fn lcd_home(shared: &SharedDevice) -> Result<(), String> {
    let mut owner = lock_device(shared)?;
    owner.operation(|device| device.lcd_home())
}

fn lcd_volume(
    shared: &SharedDevice,
    state: VolumeState,
) -> Result<f64, String> {
    let mut owner = lock_device(shared)?;
    owner.operation(|device| {
        device.lcd_volume_osd(state.percent, state.muted)
    })
}

fn status_line(shared: &SharedDevice) -> Result<String, String> {
    let mut owner = lock_device(shared)?;

    owner.operation(|device| {
        let devnode = device.device_info().devnode.display().to_string();
        let scan = device.scan_rate_hz()?;
        let rgb = device.rgb_core_enabled()?;
        let overlay = device.overlay_status()?;

        Ok(format!(
            "OK connected=YES devnode={} scan_hz={} rgb={} overlay={} overlay_rgb={}",
            devnode,
            scan,
            if rgb { "ON" } else { "OFF" },
            if overlay.enabled { "ON" } else { "OFF" },
            if overlay.rgb_core_enabled { "ON" } else { "OFF" },
        ))
    })
}

fn capabilities_line(
    shared: &SharedDevice,
) -> Result<String, String> {
    let mut owner = lock_device(shared)?;

    owner.operation(|device| {
        let scan = device.scan_rate_hz()?;
        let rgb = device.rgb_core_enabled()?;
        let overlay = device.overlay_status()?;

        Ok(format!(
            concat!(
                "OK api=1 daemon=0.1.1 ",
                "firmware=EXTENDED ",
                "matrix_scan=YES ",
                "rgb_runtime=YES ",
                "overlay=YES ",
                "lcd_osd=YES ",
                "audio_watch=YES ",
                "profiles=NO ",
                "extension_manifest=V1 ",
                "persistent_write=NO ",
                "eeprom_write=NO ",
                "qmk_flash=NO ",
                "scan_hz={} ",
                "rgb_state={} ",
                "overlay_state={} ",
                "overlay_rgb_state={}"
            ),
            scan,
            if rgb { "ON" } else { "OFF" },
            if overlay.enabled { "ON" } else { "OFF" },
            if overlay.rgb_core_enabled { "ON" } else { "OFF" },
        ))
    })
}

fn parse_percent(value: Option<&str>) -> Result<u8, String> {
    let raw = value.ok_or_else(|| "missing percent".to_string())?;
    let percent = raw
        .parse::<u8>()
        .map_err(|_| format!("invalid percent: {raw}"))?;

    if percent > 100 {
        return Err(format!("percent out of range: {percent}"));
    }

    Ok(percent)
}

fn handle_request(
    request: &str,
    shared: &SharedDevice,
) -> Result<String, String> {
    let fields: Vec<&str> = request.split_whitespace().collect();

    if fields.is_empty() {
        return Err("empty request".to_string());
    }

    match fields.as_slice() {
        ["PING"] => Ok("OK PONG".to_string()),

        ["CAPABILITIES"] => capabilities_line(shared),

        ["STATUS"] => status_line(shared),

        ["RGB", "ON"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.set_rgb_core(true))?;
            Ok(format!(
                "OK rgb={}",
                if state { "ON" } else { "OFF" }
            ))
        }

        ["RGB", "OFF"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.set_rgb_core(false))?;
            Ok(format!(
                "OK rgb={}",
                if state { "ON" } else { "OFF" }
            ))
        }

        ["OVERLAY", "STATUS"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.overlay_status())?;
            Ok(format!(
                "OK overlay={} rgb={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
            ))
        }

        ["OVERLAY", "ON"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.set_overlay(true))?;
            Ok(format!(
                "OK overlay={} rgb={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
            ))
        }

        ["OVERLAY", "OFF"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.set_overlay(false))?;
            Ok(format!(
                "OK overlay={} rgb={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
            ))
        }

        ["LCD", "HOME"] => {
            lcd_home(shared)?;
            Ok("OK lcd=HOME".to_string())
        }

        ["LCD", "VOLUME", value] => {
            let percent = parse_percent(Some(value))?;
            let ack = lcd_volume(
                shared,
                VolumeState {
                    percent,
                    muted: false,
                },
            )?;
            Ok(format!(
                "OK lcd=VOLUME percent={} ack_ms={:.3}",
                percent, ack
            ))
        }

        ["LCD", "MUTE", value] => {
            let percent = parse_percent(Some(value))?;
            let ack = lcd_volume(
                shared,
                VolumeState {
                    percent,
                    muted: true,
                },
            )?;
            Ok(format!(
                "OK lcd=MUTE percent={} ack_ms={:.3}",
                percent, ack
            ))
        }

        ["AUDIO", "CURRENT"] => {
            let state = read_volume()?;
            Ok(format!(
                "OK volume={} muted={}",
                state.percent,
                if state.muted { "YES" } else { "NO" }
            ))
        }

        _ => Err(format!("unknown request: {request}")),
    }
}

fn handle_client(
    mut stream: UnixStream,
    shared: SharedDevice,
) {
    let cloned = match stream.try_clone() {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!("AL80D_CLIENT_CLONE_ERROR={error}");
            return;
        }
    };

    let mut reader = BufReader::new(cloned);
    let mut request = String::new();

    match reader.read_line(&mut request) {
        Ok(0) => return,
        Ok(_) => {}
        Err(error) => {
            let _ = writeln!(stream, "ERR read_failed={error}");
            return;
        }
    }

    let request = request.trim();

    let response = match handle_request(request, &shared) {
        Ok(response) => response,
        Err(error) => format!("ERR {error}"),
    };

    let _ = writeln!(stream, "{response}");
}

fn start_ipc_server(
    shared: SharedDevice,
) -> Result<thread::JoinHandle<()>, String> {
    let path = socket_path();

    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            format!(
                "cannot remove stale socket {}: {e}",
                path.display()
            )
        })?;
    }

    let listener = UnixListener::bind(&path).map_err(|e| {
        format!("cannot bind {}: {e}", path.display())
    })?;

    println!("AL80D_SOCKET={}", path.display());
    println!("AL80D_IPC_READY=YES");

    Ok(thread::spawn(move || {
        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    let shared = Arc::clone(&shared);
                    thread::spawn(move || handle_client(stream, shared));
                }
                Err(error) => {
                    eprintln!("AL80D_ACCEPT_ERROR={error}");
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }))
}

fn start_audio_reader() -> Result<
    (
        Child,
        mpsc::Receiver<String>,
        thread::JoinHandle<()>,
    ),
    String,
> {
    let mut child = Command::new("/usr/bin/pactl")
        .arg("subscribe")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("pactl subscribe failed: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "pactl stdout unavailable".to_string())?;

    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok((child, rx, handle))
}

fn run_audio_session(
    shared: &SharedDevice,
) -> Result<(), String> {
    println!("AL80D_LCD_SESSION_START=YES");

    lcd_home(shared)?;
    thread::sleep(Duration::from_secs(1));

    let mut observed = read_volume()?;
    let mut pending: Option<VolumeState> = None;
    let mut pending_since: Option<Instant> = None;
    let mut last_sent: Option<VolumeState> = None;
    let mut last_change_at: Option<Instant> = None;
    let mut home_sent = true;

    let (mut child, rx, reader_thread) = start_audio_reader()?;

    println!("AL80D_AUDIO_EVENT_MODE=PACTL_SUBSCRIBE");
    println!(
        "AL80D_WATCHER_READY=YES INITIAL_VOLUME={}",
        if observed.muted {
            "MUTE".to_string()
        } else {
            format!("{}%", observed.percent)
        }
    );

    loop {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(line) => {
                let relevant =
                    line.contains("Event 'change' on sink")
                        || line.contains("Event 'change' on server");

                if relevant {
                    let current = read_volume()?;

                    if current != observed {
                        let previous = observed;
                        observed = current;

                        let now = Instant::now();
                        last_change_at = Some(now);
                        home_sent = false;

                        println!(
                            "AL80D_FEDORA_CHANGE={}",
                            if current.muted {
                                "MUTE".to_string()
                            } else {
                                format!("{}%", current.percent)
                            }
                        );

                        if current.muted != previous.muted {
                            pending = None;
                            pending_since = None;

                            if Some(current) != last_sent {
                                let ack = lcd_volume(shared, current)?;
                                println!(
                                    "AL80D_LCD_SEND={} ACK_MS={:.3}",
                                    if current.muted {
                                        "MUTE".to_string()
                                    } else {
                                        format!("{}%", current.percent)
                                    },
                                    ack
                                );
                                last_sent = Some(current);
                            }
                        } else {
                            pending = Some(current);
                            pending_since = Some(now);
                        }
                    }
                }
            }

            Err(mpsc::RecvTimeoutError::Timeout) => {}

            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader_thread.join();
                return Err(
                    "pactl subscribe reader disconnected".to_string()
                );
            }
        }

        let now = Instant::now();

        if let (Some(state), Some(since)) =
            (pending, pending_since)
        {
            if now.duration_since(since) >= SETTLE {
                pending = None;
                pending_since = None;

                if Some(state) != last_sent {
                    let ack = lcd_volume(shared, state)?;
                    println!(
                        "AL80D_LCD_SEND={} ACK_MS={:.3}",
                        if state.muted {
                            "MUTE".to_string()
                        } else {
                            format!("{}%", state.percent)
                        },
                        ack
                    );
                    last_sent = Some(state);
                }
            }
        }

        if !home_sent && pending.is_none() {
            if let Some(changed) = last_change_at {
                if now.duration_since(changed) >= HOME_IDLE {
                    lcd_home(shared)?;
                    println!("AL80D_IDLE_HOME=PASS");
                    home_sent = true;
                    last_sent = None;
                }
            }
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("pactl poll failed: {e}"))?
        {
            let _ = reader_thread.join();
            return Err(format!(
                "pactl subscribe exited: {status}"
            ));
        }
    }
}

fn main() {
    println!("AL80D=START");
    println!("AL80D_VERSION=0.1.1");
    println!("AL80D_DEVICE_OWNERSHIP=SINGLE_PROCESS");
    println!("AL80D_AUDIO_WATCH=EVENT_DRIVEN");
    println!("AL80D_HOST_SETTLE_MS=50");
    println!("AL80D_HOME_IDLE_MS=3000");

    let shared = Arc::new(Mutex::new(DeviceOwner::new()));

    match start_ipc_server(Arc::clone(&shared)) {
        Ok(_ipc) => {}
        Err(error) => {
            eprintln!("AL80D_FATAL={error}");
            std::process::exit(1);
        }
    }

    loop {
        match run_audio_session(&shared) {
            Ok(()) => {}
            Err(error) => {
                eprintln!("AL80D_SESSION_ERROR={error}");

                if let Ok(mut owner) = shared.lock() {
                    owner.device = None;
                }

                thread::sleep(RECONNECT);
            }
        }
    }
}
