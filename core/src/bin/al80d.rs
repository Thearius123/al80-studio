#[cfg(windows)]
use al80_core::windows_ipc::{NamedPipeListener, NamedPipeStream};
use al80_core::auto_lcd_feedback::{auto_lcd_policy, AutoLcdPolicy};
use al80_core::lcd_feedback::LcdFeedback;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use al80_core::input_event_bridge::{
    InputEventKind as BridgeInputEventKind, SequenceObservation, TriggerKind as BridgeTriggerKind,
};
use al80_core::raw_hid_session::HostInputEvent;
use al80_core::{Al80, InputAction, InputBinding, InputEvent, InputTrigger};

const SETTLE: Duration = Duration::from_millis(50);
const HOME_IDLE: Duration = Duration::from_secs(3);

const GENERIC_LCD_IDLE: Duration = Duration::from_millis(2200);

/*
 * LCD_GENERIC_FEEDBACK_V1
 *
 * A delayed generic-HOME must not overwrite a newer Volume/MUTE OSD.
 */
static LCD_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/*
 * LCD_LOGICAL_STATUS_V1
 *
 * Mirrors the last successfully host-driven LCD semantic state.
 * This is not pixel readback from arbitrary LCD firmware content.
 */
#[derive(Debug, Clone)]
struct LcdLogicalState {
    mode: String,
    generation: u64,
    percent: Option<u8>,
    muted: bool,
    kind: Option<String>,
    value: Option<String>,
}

static LCD_LOGICAL_STATE: std::sync::OnceLock<std::sync::Mutex<LcdLogicalState>> =
    std::sync::OnceLock::new();

fn lcd_logical_state() -> &'static std::sync::Mutex<LcdLogicalState> {
    LCD_LOGICAL_STATE.get_or_init(|| {
        std::sync::Mutex::new(LcdLogicalState {
            mode: "HOME".to_string(),
            generation: 0,
            percent: None,
            muted: false,
            kind: None,
            value: None,
        })
    })
}

fn lcd_record_home(generation: u64) {
    if let Ok(mut state) = lcd_logical_state().lock() {
        state.mode = "HOME".to_string();
        state.generation = generation;
        state.percent = None;
        state.muted = false;
        state.kind = None;
        state.value = None;
    }
}

fn lcd_record_volume(generation: u64, volume: VolumeState) {
    if let Ok(mut state) = lcd_logical_state().lock() {
        state.mode = if volume.muted {
            "MUTE".to_string()
        } else {
            "VOLUME".to_string()
        };
        state.generation = generation;
        state.percent = Some(volume.percent);
        state.muted = volume.muted;
        state.kind = None;
        state.value = None;
    }
}

fn lcd_record_feedback(generation: u64, kind: &str, value: &str) {
    if let Ok(mut state) = lcd_logical_state().lock() {
        state.mode = "FEEDBACK".to_string();
        state.generation = generation;
        state.percent = None;
        state.muted = false;
        state.kind = Some(kind.to_string());
        state.value = Some(value.to_string());
    }
}

fn lcd_status_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.' | ':' | '/') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn lcd_status_line() -> Result<String, String> {
    let state = lcd_logical_state()
        .lock()
        .map_err(|_| "LCD logical state mutex poisoned".to_string())?
        .clone();

    Ok(format!(
        "OK lcd=STATUS mode={} generation={} percent={} muted={} kind={} value={}",
        state.mode,
        state.generation,
        state
            .percent
            .map(|value| value.to_string())
            .unwrap_or_else(|| "-".to_string()),
        if state.muted { "YES" } else { "NO" },
        state
            .kind
            .as_deref()
            .map(lcd_status_token)
            .unwrap_or_else(|| "-".to_string()),
        state
            .value
            .as_deref()
            .map(lcd_status_token)
            .unwrap_or_else(|| "-".to_string()),
    ))
}

const INPUT_EVENT_POLL: Duration = Duration::from_millis(5);
const INPUT_EVENT_NONE: u64 = u64::MAX;

static INPUT_EVENTS_CONSUMED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static INPUT_EVENT_LAST_SEQUENCE: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(INPUT_EVENT_NONE);
static INPUT_EVENT_LAST_ACTION: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(INPUT_EVENT_NONE);
static INPUT_EVENT_LAST_FIRMWARE_DROPPED: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/*
 * AL80_AUTOMATIC_LCD_ACTION_FEEDBACK_V1
 *
 * Input event consumption must never block on the ~30 KiB LCD stream.
 * One worker owns automatic generic feedback. While it is busy, newer
 * generic auto-feedback is dropped rather than building stale backlog.
 *
 * Volume/Mute actions never enter this worker. They remain owned by the
 * actual Fedora audio watcher so the display shows real host state.
 */
const AUTO_LCD_QUEUE_CAPACITY: usize = 1;

static AUTO_LCD_BUSY: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static AUTO_LCD_ENQUEUED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static AUTO_LCD_SENT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static AUTO_LCD_CANCELLED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static AUTO_LCD_DROPPED_BUSY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static AUTO_LCD_ERRORS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Debug, Clone, Copy)]
struct AutoLcdRequest {
    action: u8,
    feedback: LcdFeedback,
}

fn lcd_generation_bump() -> u64 {
    LCD_GENERATION.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
}

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

                eprintln!("AL80D_TRANSACTION_RETRY=YES FIRST_ERROR={first_error}");

                let retry_result = {
                    let device = self.ensure_connected().map_err(|reconnect_error| {
                        format!(
                            concat!(
                                "AL80 reconnect failed after transaction ",
                                "error: first={}; reconnect={}"
                            ),
                            first_error, reconnect_error
                        )
                    })?;

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
                            first_error, retry_error
                        ))
                    }
                }
            }
        }
    }
}

type SharedDevice = Arc<Mutex<DeviceOwner>>;

fn lock_device(shared: &SharedDevice) -> Result<MutexGuard<'_, DeviceOwner>, String> {
    shared
        .lock()
        .map_err(|_| "AL80D device mutex poisoned".to_string())
}

#[cfg(unix)]
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

    let percent = percent.ok_or_else(|| format!("cannot parse wpctl output: {}", raw.trim()))?;

    let muted = raw.to_ascii_uppercase().contains("[MUTED]");

    Ok(VolumeState { percent, muted })
}

fn lcd_home_direct(shared: &SharedDevice) -> Result<(), String> {
    let mut owner = lock_device(shared)?;
    owner.operation(|device| device.lcd_home())?;

    let generation = LCD_GENERATION.load(std::sync::atomic::Ordering::SeqCst);
    lcd_record_home(generation);

    Ok(())
}

fn lcd_home(shared: &SharedDevice) -> Result<(), String> {
    lcd_generation_bump();
    lcd_home_direct(shared)
}

fn lcd_home_if_generation(shared: &SharedDevice, generation: u64) -> Result<bool, String> {
    let mut owner = lock_device(shared)?;

    if LCD_GENERATION.load(std::sync::atomic::Ordering::SeqCst) != generation {
        return Ok(false);
    }

    owner.operation(|device| device.lcd_home())?;
    lcd_record_home(generation);

    Ok(true)
}

fn lcd_volume_with_generation(
    shared: &SharedDevice,
    state: VolumeState,
) -> Result<(f64, u64), String> {
    /*
     * Critical priority invariant:
     * bump generation BEFORE waiting for the device mutex.
     * A generic frame holding the mutex observes this new generation
     * between LCD chunks and terminates its transfer early.
     */
    let generation = lcd_generation_bump();

    let mut owner = lock_device(shared)?;

    let ack = owner.operation(|device| device.lcd_volume_osd(state.percent, state.muted))?;
    lcd_record_volume(generation, state);

    Ok((ack, generation))
}

fn lcd_volume(shared: &SharedDevice, state: VolumeState) -> Result<f64, String> {
    let (ack, _) = lcd_volume_with_generation(shared, state)?;

    Ok(ack)
}

fn lcd_generic_feedback(
    shared: &SharedDevice,
    feedback: &LcdFeedback,
) -> Result<al80_core::lcd_feedback::LcdFeedbackTransfer, String> {
    let mut owner = lock_device(shared)?;

    owner.operation(|device| device.lcd_generic_feedback(feedback))
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
            if overlay.rgb_core_enabled {
                "ON"
            } else {
                "OFF"
            },
        ))
    })
}

fn capabilities_line(shared: &SharedDevice) -> Result<String, String> {
    let mut owner = lock_device(shared)?;

    owner.operation(|device| {
        let scan = device.scan_rate_hz()?;
        let rgb = device.rgb_core_enabled()?;
        let overlay = device.overlay_status()?;
        let creator = device.creator_scene_status()?;
        let input = device.input_router_status()?;

        Ok(format!(
            concat!(
                "OK api=1 daemon=0.6.0 ",
                "firmware=EXTENDED ",
                "matrix_scan=YES ",
                "rgb_runtime=YES ",
                "overlay=YES ",
                "lcd_osd=YES ",
                "audio_watch=YES ",
                "profiles=NO ",
                "extension_manifest=V1 ",
                "per_key_rgb=YES ",
                "creator_scene=YES ",
                "rgb_leds=82 ",
                "key_rgb_leds=79 ",
                "accent_rgb_leds=3 ",
                "creator_scene_state={} ",
                "input_router=YES ",
                "input_event_bridge_host=YES ",
                "input_event_firmware=YES ",
                "input_event_auto_lcd=YES ",
                "lcd_feedback=YES ",
                "lcd_feedback_kinds=8 ",
                "input_bindings={} ",
                "input_actions={} ",
                "input_router_state={} ",
                "persistent_write=NO ",
                "eeprom_write=NO ",
                "qmk_flash=NO ",
                "scan_hz={} ",
                "rgb_state={} ",
                "overlay_state={} ",
                "overlay_rgb_state={}"
            ),
            if creator.enabled { "ON" } else { "OFF" },
            input.binding_slots,
            input.max_action,
            if input.enabled { "ON" } else { "OFF" },
            scan,
            if rgb { "ON" } else { "OFF" },
            if overlay.enabled { "ON" } else { "OFF" },
            if overlay.rgb_core_enabled {
                "ON"
            } else {
                "OFF"
            },
        ))
    })
}

fn decode_input_bindings(raw: &str) -> Result<Vec<InputBinding>, String> {
    if raw.is_empty() {
        return Err("Input Router profile is empty".to_string());
    }

    let segments: Vec<&str> = raw.split(';').collect();

    if segments.len() > 12 {
        return Err(format!(
            "Input Router accepts at most 12 bindings, got {}",
            segments.len()
        ));
    }

    let mut bindings = Vec::with_capacity(segments.len());

    for (index, segment) in segments.iter().enumerate() {
        let values: Vec<&str> = segment.split(',').collect();

        if values.len() != 5 {
            return Err(format!(
                "binding {} must contain event,trigger,a,b,action",
                index
            ));
        }

        let parse = |name: &str, value: &str| -> Result<u8, String> {
            value
                .parse::<u8>()
                .map_err(|error| format!("invalid {name} in binding {index}: {error}"))
        };

        let event = InputEvent::try_from(parse("event", values[0])?)?;
        let trigger = InputTrigger::try_from(parse("trigger", values[1])?)?;
        let trigger_a = parse("trigger_a", values[2])?;
        let trigger_b = parse("trigger_b", values[3])?;
        let action = InputAction::from_id(parse("action", values[4])?)?;

        bindings.push(InputBinding::new(
            event, trigger, trigger_a, trigger_b, action,
        )?);
    }

    Ok(bindings)
}

fn input_status_line(enabled: bool, version: u8, slots: u8, actions: u8, fallback: bool) -> String {
    format!(
        "OK input={} version={} slots={} actions={} fallback={}",
        if enabled { "ON" } else { "OFF" },
        version,
        slots,
        actions,
        if fallback { "YES" } else { "NO" },
    )
}

fn decode_creator_scene_hex(raw: &str) -> Result<Vec<[u8; 3]>, String> {
    const LEDS: usize = 82;
    const HEX_PER_LED: usize = 6;
    const EXPECTED: usize = LEDS * HEX_PER_LED;

    if !raw.is_ascii() {
        return Err("Creator Scene payload must be ASCII hex".to_string());
    }

    if raw.len() != EXPECTED {
        return Err(format!(
            concat!(
                "Creator Scene payload must contain ",
                "{} hex characters, got {}"
            ),
            EXPECTED,
            raw.len()
        ));
    }

    if !raw.bytes().all(|value| value.is_ascii_hexdigit()) {
        return Err("Creator Scene payload contains non-hex data".to_string());
    }

    let mut colors = Vec::with_capacity(LEDS);

    for index in 0..LEDS {
        let start = index * HEX_PER_LED;
        let chunk = &raw[start..start + HEX_PER_LED];

        let value = u32::from_str_radix(chunk, 16)
            .map_err(|_| format!("invalid RGB hex at Creator LED {}", index))?;

        colors.push([
            ((value >> 16) & 0xFF) as u8,
            ((value >> 8) & 0xFF) as u8,
            (value & 0xFF) as u8,
        ]);
    }

    Ok(colors)
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

fn handle_request(request: &str, shared: &SharedDevice) -> Result<String, String> {
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
            Ok(format!("OK rgb={}", if state { "ON" } else { "OFF" }))
        }

        ["RGB", "OFF"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.set_rgb_core(false))?;
            Ok(format!("OK rgb={}", if state { "ON" } else { "OFF" }))
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

        ["TELEMETRY", "RGB"] => {
            let mut owner = lock_device(shared)?;

            let live = owner.operation(|device| device.live_rgb_telemetry())?;

            let source = match live.source {
                1 => "SNAKE",
                2 => "CREATOR",
                3 => "LOW_BATTERY",
                _ => "NATIVE_UNKNOWN",
            };

            let mut frame = String::with_capacity(live.colors.len() * 6);

            for color in &live.colors {
                use std::fmt::Write as _;
                let _ = write!(
                    &mut frame,
                    "{:02x}{:02x}{:02x}",
                    color[0], color[1], color[2]
                );
            }

            Ok(format!(
                concat!(
                    "OK telemetry=RGB version={} source={} ",
                    "frame_valid={} rgb={} overlay={} creator={} ",
                    "leds={} frame={}"
                ),
                live.version,
                source,
                if live.frame_valid { "YES" } else { "NO" },
                if live.rgb_core_enabled { "ON" } else { "OFF" },
                if live.overlay_enabled { "ON" } else { "OFF" },
                if live.creator_scene_enabled {
                    "ON"
                } else {
                    "OFF"
                },
                live.colors.len(),
                frame,
            ))
        }

        ["INPUT", "EVENTS"] => input_events_status_line(shared),

        ["INPUT", "STATUS"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.input_router_status())?;

            Ok(input_status_line(
                state.enabled,
                state.version,
                state.binding_slots,
                state.max_action,
                state.fallback_supported,
            ))
        }

        ["INPUT", "OFF"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.input_router_disable())?;

            Ok(input_status_line(
                state.enabled,
                state.version,
                state.binding_slots,
                state.max_action,
                state.fallback_supported,
            ))
        }

        ["INPUT", "DEFAULTS"] => {
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.input_router_apply_defaults())?;

            Ok(input_status_line(
                state.enabled,
                state.version,
                state.binding_slots,
                state.max_action,
                state.fallback_supported,
            ))
        }

        ["INPUT", "APPLY", raw] => {
            let bindings = decode_input_bindings(raw)?;
            let count = bindings.len();
            let mut owner = lock_device(shared)?;
            let state = owner.operation(|device| device.input_router_apply(&bindings))?;

            Ok(format!(
                "OK input={} bindings={} version={} slots={} actions={} fallback={}",
                if state.enabled { "ON" } else { "OFF" },
                count,
                state.version,
                state.binding_slots,
                state.max_action,
                if state.fallback_supported {
                    "YES"
                } else {
                    "NO"
                },
            ))
        }

        ["INPUT", "DUMP"] => {
            let mut owner = lock_device(shared)?;

            owner.operation(|device| {
                let state = device.input_router_status()?;
                let mut encoded = Vec::new();

                for slot in 0u8..12u8 {
                    if let Some(binding) = device.input_router_get_binding(slot)? {
                        encoded.push(format!(
                            "{},{},{},{},{},{}",
                            slot,
                            binding.event as u8,
                            binding.trigger as u8,
                            binding.trigger_a,
                            binding.trigger_b,
                            binding.action.id(),
                        ));
                    }
                }

                Ok(format!(
                    "OK input={} bindings={}",
                    if state.enabled { "ON" } else { "OFF" },
                    encoded.join(";")
                ))
            })
        }

        ["SCENE", "STATUS"] => {
            let mut owner = lock_device(shared)?;

            let state = owner.operation(|device| device.creator_scene_status())?;

            Ok(format!(
                "OK scene={} rgb={} leds={} chunk={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
                state.led_count,
                state.chunk_max,
            ))
        }

        ["SCENE", "OFF"] => {
            let mut owner = lock_device(shared)?;

            let state = owner.operation(|device| device.creator_scene_disable())?;

            Ok(format!(
                "OK scene={} rgb={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
            ))
        }

        ["SCENE", "APPLY", raw] => {
            let colors = decode_creator_scene_hex(raw)?;

            let mut owner = lock_device(shared)?;

            let state = owner.operation(|device| device.creator_scene_apply(&colors))?;

            Ok(format!(
                "OK scene={} rgb={} leds={}",
                if state.enabled { "ON" } else { "OFF" },
                if state.rgb_core_enabled { "ON" } else { "OFF" },
                state.led_count,
            ))
        }

        ["LCD", "STATUS"] => lcd_status_line(),

        ["LCD", "FEEDBACK", kind, value] => {
            let feedback = LcdFeedback::parse(kind, value)?;

            let kind_out = feedback.kind_token().to_string();

            let value_out = feedback.value_token();

            let generation = lcd_generation_bump();

            let transfer = lcd_generic_feedback(shared, &feedback)?;
            lcd_record_feedback(generation, &kind_out, &value_out);

            let delayed_shared = Arc::clone(shared);

            std::thread::spawn(move || {
                std::thread::sleep(GENERIC_LCD_IDLE);

                match lcd_home_if_generation(&delayed_shared, generation) {
                    Ok(true) => {
                        println!("AL80D_GENERIC_LCD_HOME=PASS");
                    }

                    Ok(false) => {
                        println!("AL80D_GENERIC_LCD_HOME=SKIPPED_NEWER_ACTIVITY");
                    }

                    Err(error) => {
                        eprintln!("AL80D_GENERIC_LCD_HOME_ERROR={error}");
                    }
                }
            });

            println!(
                "AL80D_GENERIC_LCD_SEND={}:{} BYTES={} CHUNKS={} MS={:.3}",
                kind_out, value_out, transfer.bytes, transfer.chunks, transfer.elapsed_ms,
            );

            Ok(format!(
                "OK lcd=FEEDBACK kind={} value={} bytes={} chunks={} elapsed_ms={:.3}",
                kind_out, value_out, transfer.bytes, transfer.chunks, transfer.elapsed_ms,
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
            Ok(format!("OK lcd=MUTE percent={} ack_ms={:.3}", percent, ack))
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

fn bridge_event_name(event: BridgeInputEventKind) -> &'static str {
    match event {
        BridgeInputEventKind::KnobCcw => "KNOB_CCW",
        BridgeInputEventKind::KnobCw => "KNOB_CW",
        BridgeInputEventKind::KnobPress => "KNOB_PRESS",
    }
}

fn bridge_trigger_name(trigger: BridgeTriggerKind) -> &'static str {
    match trigger {
        BridgeTriggerKind::None => "NONE",
        BridgeTriggerKind::Layer => "LAYER",
        BridgeTriggerKind::Matrix => "MATRIX",
        BridgeTriggerKind::Mods => "MODS",
    }
}

fn bridge_action_name(action: u8) -> &'static str {
    match action {
        0 => "NONE",
        1 => "VOLUME_DOWN",
        2 => "VOLUME_UP",
        3 => "MUTE",
        4 => "MEDIA_PREVIOUS",
        5 => "MEDIA_NEXT",
        6 => "MEDIA_PLAY_PAUSE",
        7 => "BRIGHTNESS_DOWN",
        8 => "BRIGHTNESS_UP",
        9 => "LEFT",
        10 => "RIGHT",
        11 => "UP",
        12 => "DOWN",
        13 => "PAGE_UP",
        14 => "PAGE_DOWN",
        15 => "RGB_VALUE_DOWN",
        16 => "RGB_VALUE_UP",
        17 => "RGB_HUE_DOWN",
        18 => "RGB_HUE_UP",
        19 => "RGB_SPEED_DOWN",
        20 => "RGB_SPEED_UP",
        21 => "SNAKE_OFF",
        22 => "SNAKE_ON",
        23 => "SNAKE_TOGGLE",
        24 => "CREATOR_SCENE_OFF",
        _ => "INVALID",
    }
}

fn sequence_observation_name(observation: SequenceObservation) -> &'static str {
    match observation {
        SequenceObservation::First(_) => "FIRST",
        SequenceObservation::Consecutive { .. } => "OK",
        SequenceObservation::Duplicate(_) => "DUPLICATE",
        SequenceObservation::Gap { .. } => "GAP",
    }
}

fn input_event_log_line(host_event: HostInputEvent) -> String {
    let event = host_event.event;

    format!(
        concat!(
            "AL80D_INPUT_EVENT={} SEQ={} SEQ_STATE={} ",
            "SLOT={} TRIGGER={} A={} B={} ACTION={} ",
            "ACTION_NAME={} FW_DROPPED={}"
        ),
        bridge_event_name(event.event),
        event.sequence,
        sequence_observation_name(host_event.sequence),
        event.slot,
        bridge_trigger_name(event.trigger),
        event.trigger_a,
        event.trigger_b,
        event.action,
        bridge_action_name(event.action),
        event.dropped_counter
    )
}

fn record_input_event(host_event: HostInputEvent) {
    let event = host_event.event;

    INPUT_EVENTS_CONSUMED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    INPUT_EVENT_LAST_SEQUENCE.store(event.sequence as u64, std::sync::atomic::Ordering::Relaxed);

    INPUT_EVENT_LAST_ACTION.store(event.action as u64, std::sync::atomic::Ordering::Relaxed);

    INPUT_EVENT_LAST_FIRMWARE_DROPPED.store(
        event.dropped_counter as u64,
        std::sync::atomic::Ordering::Relaxed,
    );

    println!("{}", input_event_log_line(host_event));
}

fn auto_lcd_enqueue(sender: &mpsc::SyncSender<AutoLcdRequest>, action: u8) {
    match auto_lcd_policy(action) {
        Ok(AutoLcdPolicy::None) => {
            println!("AL80D_AUTO_LCD_POLICY=NONE ACTION={action}");
        }

        Ok(AutoLcdPolicy::AudioWatcher) => {
            /*
             * Preempt a generic automatic frame immediately, before the
             * actual host audio watcher reaches the device mutex.
             * The watcher remains authoritative for the displayed value.
             */
            let generation = lcd_generation_bump();

            println!(
                concat!(
                    "AL80D_AUTO_LCD_POLICY=AUDIO_WATCHER ACTION={} ",
                    "PREEMPT_GENERATION={}"
                ),
                action, generation,
            );
        }

        Ok(AutoLcdPolicy::Feedback(feedback)) => {
            if AUTO_LCD_BUSY
                .compare_exchange(
                    false,
                    true,
                    std::sync::atomic::Ordering::AcqRel,
                    std::sync::atomic::Ordering::Acquire,
                )
                .is_err()
            {
                AUTO_LCD_DROPPED_BUSY.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                println!("AL80D_AUTO_LCD_DROP=BUSY ACTION={action}");

                return;
            }

            match sender.try_send(AutoLcdRequest { action, feedback }) {
                Ok(()) => {
                    AUTO_LCD_ENQUEUED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    println!(
                        "AL80D_AUTO_LCD_QUEUED=YES ACTION={action} KIND={} VALUE={}",
                        feedback.kind_token(),
                        feedback.value_token(),
                    );
                }

                Err(error) => {
                    AUTO_LCD_BUSY.store(false, std::sync::atomic::Ordering::Release);

                    AUTO_LCD_DROPPED_BUSY.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    eprintln!("AL80D_AUTO_LCD_QUEUE_ERROR={error} ACTION={action}");
                }
            }
        }

        Err(error) => {
            AUTO_LCD_ERRORS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            eprintln!("AL80D_AUTO_LCD_POLICY_ERROR={error}");
        }
    }
}

fn lcd_auto_feedback(
    shared: &SharedDevice,
    feedback: &LcdFeedback,
    generation: u64,
) -> Result<al80_core::lcd_feedback::LcdFeedbackTransfer, String> {
    let mut owner = lock_device(shared)?;

    owner.operation(|device| {
        device.lcd_generic_feedback_until(feedback, || {
            LCD_GENERATION.load(std::sync::atomic::Ordering::SeqCst) == generation
        })
    })
}

fn schedule_auto_lcd_home(shared: &SharedDevice, generation: u64) {
    let delayed_shared = Arc::clone(shared);

    thread::spawn(move || {
        thread::sleep(GENERIC_LCD_IDLE);

        match lcd_home_if_generation(&delayed_shared, generation) {
            Ok(true) => {
                println!("AL80D_AUTO_LCD_HOME=PASS");
            }

            Ok(false) => {
                println!("AL80D_AUTO_LCD_HOME=SKIPPED_NEWER_ACTIVITY");
            }

            Err(error) => {
                eprintln!("AL80D_AUTO_LCD_HOME_ERROR={error}");
            }
        }
    });
}

fn start_auto_lcd_worker(
    shared: SharedDevice,
    receiver: mpsc::Receiver<AutoLcdRequest>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("AL80D_AUTO_LCD_WORKER=READY QUEUE_CAPACITY={AUTO_LCD_QUEUE_CAPACITY}");

        while let Ok(request) = receiver.recv() {
            let generation = lcd_generation_bump();

            let kind = request.feedback.kind_token().to_string();

            let value = request.feedback.value_token();

            match lcd_auto_feedback(&shared, &request.feedback, generation) {
                Ok(transfer) if transfer.cancelled => {
                    AUTO_LCD_CANCELLED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    println!(
                        concat!(
                            "AL80D_AUTO_LCD_CANCELLED=NEWER_ACTIVITY ",
                            "ACTION={} KIND={} VALUE={} CHUNKS={} MS={:.3}"
                        ),
                        request.action, kind, value, transfer.chunks, transfer.elapsed_ms,
                    );
                }

                Ok(transfer) => {
                    AUTO_LCD_SENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    println!(
                        concat!(
                            "AL80D_AUTO_LCD_SEND=PASS ACTION={} ",
                            "KIND={} VALUE={} BYTES={} CHUNKS={} MS={:.3}"
                        ),
                        request.action,
                        kind,
                        value,
                        transfer.bytes,
                        transfer.chunks,
                        transfer.elapsed_ms,
                    );

                    lcd_record_feedback(generation, &kind, &value);
                    schedule_auto_lcd_home(&shared, generation);
                }

                Err(error) => {
                    AUTO_LCD_ERRORS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                    eprintln!(
                        "AL80D_AUTO_LCD_SEND_ERROR={error} ACTION={}",
                        request.action,
                    );
                }
            }

            AUTO_LCD_BUSY.store(false, std::sync::atomic::Ordering::Release);
        }

        AUTO_LCD_BUSY.store(false, std::sync::atomic::Ordering::Release);

        eprintln!("AL80D_AUTO_LCD_WORKER=DISCONNECTED");
    })
}

fn input_events_status_line(shared: &SharedDevice) -> Result<String, String> {
    let (queued, stats) = {
        let mut owner = shared
            .lock()
            .map_err(|_| "device mutex poisoned".to_string())?;

        owner.operation(|device| {
            Ok((
                device.queued_input_events()?,
                device.raw_hid_session_stats(),
            ))
        })?
    };

    let consumed = INPUT_EVENTS_CONSUMED.load(std::sync::atomic::Ordering::Relaxed);

    let last_sequence = INPUT_EVENT_LAST_SEQUENCE.load(std::sync::atomic::Ordering::Relaxed);

    let last_action = INPUT_EVENT_LAST_ACTION.load(std::sync::atomic::Ordering::Relaxed);

    let firmware_dropped =
        INPUT_EVENT_LAST_FIRMWARE_DROPPED.load(std::sync::atomic::Ordering::Relaxed);

    Ok(format!(
        concat!(
            "OK input_event_bridge_host=YES ",
            "firmware_event_emitter=YES ",
            "auto_lcd=YES queue_capacity=8 ",
            "received={} consumed={} queued={} ",
            "malformed={} host_queue_drops={} ",
            "sequence_gaps={} sequence_duplicates={} ",
            "last_sequence={} last_action={} ",
            "firmware_dropped={} ",
            "auto_lcd_enqueued={} auto_lcd_sent={} ",
            "auto_lcd_cancelled={} auto_lcd_dropped_busy={} ",
            "auto_lcd_errors={}"
        ),
        stats.events_received,
        consumed,
        queued,
        stats.malformed_events,
        stats.host_event_queue_drops,
        stats.sequence_gaps,
        stats.sequence_duplicates,
        if last_sequence == INPUT_EVENT_NONE {
            "NONE".to_string()
        } else {
            last_sequence.to_string()
        },
        if last_action == INPUT_EVENT_NONE {
            "NONE".to_string()
        } else {
            last_action.to_string()
        },
        firmware_dropped,
        AUTO_LCD_ENQUEUED.load(std::sync::atomic::Ordering::Relaxed,),
        AUTO_LCD_SENT.load(std::sync::atomic::Ordering::Relaxed,),
        AUTO_LCD_CANCELLED.load(std::sync::atomic::Ordering::Relaxed,),
        AUTO_LCD_DROPPED_BUSY.load(std::sync::atomic::Ordering::Relaxed,),
        AUTO_LCD_ERRORS.load(std::sync::atomic::Ordering::Relaxed,),
    ))
}

fn start_input_event_pump(
    shared: SharedDevice,
    auto_lcd_sender: mpsc::SyncSender<AutoLcdRequest>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("AL80D_INPUT_EVENT_PUMP=READY");
        println!("AL80D_INPUT_EVENT_LCD_POLICY=AUTOMATIC_V1");

        loop {
            let result = {
                let mut owner = match shared.lock() {
                    Ok(owner) => owner,
                    Err(_) => {
                        eprintln!("AL80D_INPUT_EVENT_PUMP_ERROR=device_mutex_poisoned");
                        thread::sleep(RECONNECT);
                        continue;
                    }
                };

                owner.operation(|device| device.pop_input_event())
            };

            match result {
                Ok(Some(event)) => {
                    let action = event.event.action;

                    record_input_event(event);

                    /*
                     * Nonblocking best-effort dispatch.
                     * The authoritative routed action already happened
                     * in firmware before this host event existed.
                     */
                    auto_lcd_enqueue(&auto_lcd_sender, action);
                }

                Ok(None) => {
                    thread::sleep(INPUT_EVENT_POLL);
                }

                Err(error) => {
                    eprintln!("AL80D_INPUT_EVENT_PUMP_ERROR={error}");
                    thread::sleep(RECONNECT);
                }
            }
        }
    })
}

#[cfg(unix)]
fn handle_client(mut stream: UnixStream, shared: SharedDevice) {
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

#[cfg(unix)]
fn start_ipc_server(shared: SharedDevice) -> Result<thread::JoinHandle<()>, String> {
    let path = socket_path();

    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("cannot remove stale socket {}: {e}", path.display()))?;
    }

    let listener =
        UnixListener::bind(&path).map_err(|e| format!("cannot bind {}: {e}", path.display()))?;

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

#[cfg(windows)]
fn handle_client_windows(stream: NamedPipeStream, shared: SharedDevice) {
    let mut reader = BufReader::new(stream);
    let mut request = String::new();

    match reader.read_line(&mut request) {
        Ok(0) => return,
        Ok(_) => {}
        Err(error) => {
            let _ = writeln!(reader.get_mut(), "ERR read_failed={error}");
            return;
        }
    }

    let request = request.trim();

    let response = match handle_request(request, &shared) {
        Ok(response) => response,
        Err(error) => format!("ERR {error}"),
    };

    let _ = writeln!(reader.get_mut(), "{response}");
}

#[cfg(windows)]
fn start_ipc_server(shared: SharedDevice) -> Result<thread::JoinHandle<()>, String> {
    let listener = NamedPipeListener::bind_default()?;
    println!("AL80D_WINDOWS_PIPE={}", listener.name());

    Ok(thread::spawn(move || loop {
        match listener.accept() {
            Ok(stream) => {
                let shared = Arc::clone(&shared);

                thread::spawn(move || {
                    handle_client_windows(stream, shared);
                });
            }

            Err(error) => {
                eprintln!("AL80D_WINDOWS_PIPE_ACCEPT_ERROR={error}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    }))
}

fn start_audio_reader() -> Result<(Child, mpsc::Receiver<String>, thread::JoinHandle<()>), String> {
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

fn run_audio_session(shared: &SharedDevice) -> Result<(), String> {
    println!("AL80D_LCD_SESSION_START=YES");

    lcd_home(shared)?;
    thread::sleep(Duration::from_secs(1));

    let mut observed = read_volume()?;
    let mut pending: Option<VolumeState> = None;
    let mut pending_since: Option<Instant> = None;
    let mut last_sent: Option<VolumeState> = None;
    let mut last_lcd_generation: Option<u64> = None;
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
                let relevant = line.contains("Event 'change' on sink")
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
                                let (ack, generation) =
                                    lcd_volume_with_generation(shared, current)?;

                                last_lcd_generation = Some(generation);

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
                return Err("pactl subscribe reader disconnected".to_string());
            }
        }

        let now = Instant::now();

        if let (Some(state), Some(since)) = (pending, pending_since) {
            if now.duration_since(since) >= SETTLE {
                pending = None;
                pending_since = None;

                if Some(state) != last_sent {
                    let (ack, generation) = lcd_volume_with_generation(shared, state)?;

                    last_lcd_generation = Some(generation);

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
                    let home_applied = match last_lcd_generation {
                        Some(generation) => lcd_home_if_generation(shared, generation)?,

                        None => {
                            /*
                             * Fail closed: an idle HOME without a
                             * generation must never overwrite newer
                             * LCD activity. Startup HOME remains the
                             * only unconditional audio-session HOME.
                             */
                            false
                        }
                    };

                    if home_applied {
                        println!("AL80D_IDLE_HOME=PASS");
                    } else {
                        println!("AL80D_IDLE_HOME=SKIPPED_NEWER_ACTIVITY");
                    }

                    home_sent = true;
                    last_sent = None;
                    last_lcd_generation = None;
                }
            }
        }

        if let Some(status) = child
            .try_wait()
            .map_err(|e| format!("pactl poll failed: {e}"))?
        {
            let _ = reader_thread.join();
            return Err(format!("pactl subscribe exited: {status}"));
        }
    }
}

fn main() {
    println!("AL80D=START");
    println!("AL80D_VERSION=0.6.0");
    println!("AL80D_DEVICE_OWNERSHIP=SINGLE_PROCESS");
    println!("AL80D_AUDIO_WATCH=EVENT_DRIVEN");
    println!("AL80D_HOST_SETTLE_MS=50");
    println!("AL80D_HOME_IDLE_MS=3000");
    println!("AL80D_INPUT_EVENT_BRIDGE_HOST=YES");
    println!("AL80D_INPUT_EVENT_FIRMWARE=YES");
    println!("AL80D_INPUT_EVENT_AUTO_LCD=YES");

    let shared = Arc::new(Mutex::new(DeviceOwner::new()));

    let (auto_lcd_sender, auto_lcd_receiver) = mpsc::sync_channel(AUTO_LCD_QUEUE_CAPACITY);

    let _auto_lcd_worker = start_auto_lcd_worker(Arc::clone(&shared), auto_lcd_receiver);

    let _input_event_pump = start_input_event_pump(Arc::clone(&shared), auto_lcd_sender);

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

#[cfg(test)]
mod input_event_pump_tests {
    use super::*;
    use al80_core::input_event_bridge::{InputRouterEvent, TriggerKind as BridgeTriggerKind};

    fn host_event(
        sequence: u16,
        event: BridgeInputEventKind,
        action: u8,
        observation: SequenceObservation,
    ) -> HostInputEvent {
        HostInputEvent {
            event: InputRouterEvent {
                sequence,
                event,
                slot: 2,
                trigger: BridgeTriggerKind::Layer,
                trigger_a: 1,
                trigger_b: 0,
                action,
                dropped_counter: 3,
            },
            sequence: observation,
        }
    }

    #[test]
    fn action_names_cover_allowlisted_edges() {
        assert_eq!(bridge_action_name(0), "NONE");
        assert_eq!(bridge_action_name(1), "VOLUME_DOWN");
        assert_eq!(bridge_action_name(24), "CREATOR_SCENE_OFF");
        assert_eq!(bridge_action_name(25), "INVALID");
    }

    #[test]
    fn event_names_are_typed() {
        assert_eq!(bridge_event_name(BridgeInputEventKind::KnobCcw), "KNOB_CCW");
        assert_eq!(bridge_event_name(BridgeInputEventKind::KnobCw), "KNOB_CW");
        assert_eq!(
            bridge_event_name(BridgeInputEventKind::KnobPress),
            "KNOB_PRESS"
        );
    }

    #[test]
    fn first_event_log_is_stable_and_typed() {
        let event = host_event(
            42,
            BridgeInputEventKind::KnobCw,
            22,
            SequenceObservation::First(42),
        );

        let line = input_event_log_line(event);

        assert!(line.contains("AL80D_INPUT_EVENT=KNOB_CW"));
        assert!(line.contains("SEQ=42"));
        assert!(line.contains("SEQ_STATE=FIRST"));
        assert!(line.contains("ACTION=22"));
        assert!(line.contains("ACTION_NAME=SNAKE_ON"));
        assert!(line.contains("FW_DROPPED=3"));
    }

    #[test]
    fn gap_event_log_is_nonfatal_observability() {
        let event = host_event(
            103,
            BridgeInputEventKind::KnobPress,
            24,
            SequenceObservation::Gap {
                previous: 100,
                expected: 101,
                current: 103,
            },
        );

        let line = input_event_log_line(event);

        assert!(line.contains("SEQ_STATE=GAP"));
        assert!(line.contains("ACTION_NAME=CREATOR_SCENE_OFF"));
    }
}
