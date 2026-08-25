use al80_core::lcd_feedback::LcdFeedback;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceStatus {
    connected: bool,
    devnode: Option<String>,
    matrix_scan_hz: Option<u32>,
    matrix_scan_interval_us: Option<f64>,
    rgb_core_enabled: Option<bool>,
    overlay_enabled: Option<bool>,
    overlay_reports_rgb_core: Option<bool>,
    error: Option<String>,
}

impl DeviceStatus {
    fn offline(error: impl Into<String>) -> Self {
        Self {
            connected: false,
            devnode: None,
            matrix_scan_hz: None,
            matrix_scan_interval_us: None,
            rgb_core_enabled: None,
            overlay_enabled: None,
            overlay_reports_rgb_core: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Capabilities {
    api: u32,
    daemon_version: String,
    firmware_mode: String,
    matrix_scan: bool,
    rgb_runtime: bool,
    overlay: bool,
    lcd_osd: bool,
    lcd_feedback: bool,
    audio_watch: bool,
    profiles: bool,
    extension_manifest: String,
    per_key_rgb: bool,
    creator_scene: bool,
    rgb_leds: u32,
    key_rgb_leds: u32,
    accent_rgb_leds: u32,
    creator_scene_state: Option<bool>,
    input_router: bool,
    input_bindings: u32,
    input_actions: u32,
    input_router_state: Option<bool>,
    input_event_bridge_host: bool,
    input_event_firmware: bool,
    input_event_auto_lcd: bool,
    persistent_write: bool,
    eeprom_write: bool,
    qmk_flash: bool,
    scan_hz: Option<u32>,
    rgb_state: Option<bool>,
    overlay_state: Option<bool>,
    overlay_rgb_state: Option<bool>,
    error: Option<String>,
}

impl Capabilities {
    fn offline(error: impl Into<String>) -> Self {
        Self {
            api: 0,
            daemon_version: "unknown".to_string(),
            firmware_mode: "UNKNOWN".to_string(),
            matrix_scan: false,
            rgb_runtime: false,
            overlay: false,
            lcd_osd: false,
            lcd_feedback: false,
            audio_watch: false,
            profiles: false,
            extension_manifest: "NONE".to_string(),
            per_key_rgb: false,
            creator_scene: false,
            rgb_leds: 0,
            key_rgb_leds: 0,
            accent_rgb_leds: 0,
            creator_scene_state: None,
            input_router: false,
            input_bindings: 0,
            input_actions: 0,
            input_router_state: None,
            input_event_bridge_host: false,
            input_event_firmware: false,
            input_event_auto_lcd: false,
            persistent_write: false,
            eeprom_write: false,
            qmk_flash: false,
            scan_hz: None,
            rgb_state: None,
            overlay_state: None,
            overlay_rgb_state: None,
            error: Some(error.into()),
        }
    }
}

fn socket_path() -> PathBuf {
    if let Ok(runtime) = env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("al80d.sock");
    }

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    PathBuf::from(format!("/tmp/al80d-{user}.sock"))
}

fn ipc_request(request: &str) -> Result<String, String> {
    let path = socket_path();

    let mut stream = UnixStream::connect(&path)
        .map_err(|error| format!("cannot connect to al80d at {}: {error}", path.display()))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("cannot configure al80d read timeout: {error}"))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("cannot configure al80d write timeout: {error}"))?;

    writeln!(stream, "{request}")
        .map_err(|error| format!("cannot write al80d request: {error}"))?;

    stream
        .flush()
        .map_err(|error| format!("cannot flush al80d request: {error}"))?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream);

    reader
        .read_line(&mut response)
        .map_err(|error| format!("cannot read al80d response: {error}"))?;

    let response = response.trim().to_string();

    if response.is_empty() {
        return Err("al80d returned an empty response".to_string());
    }

    if let Some(error) = response.strip_prefix("ERR ") {
        return Err(format!("al80d: {error}"));
    }

    if !response.starts_with("OK") {
        return Err(format!("unexpected al80d response: {response}"));
    }

    Ok(response)
}

fn parse_fields(response: &str) -> Vec<(&str, &str)> {
    response
        .split_whitespace()
        .skip(1)
        .filter_map(|token| token.split_once('='))
        .collect()
}

fn field<'a>(fields: &'a [(&'a str, &'a str)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
}

fn parse_on_off(value: Option<&str>, name: &str) -> Result<bool, String> {
    match value {
        Some("ON") => Ok(true),
        Some("OFF") => Ok(false),
        Some(other) => Err(format!("invalid al80d {name} value: {other}")),
        None => Err(format!("missing al80d {name} value")),
    }
}

fn parse_yes_no(value: Option<&str>, name: &str) -> Result<bool, String> {
    match value {
        Some("YES") => Ok(true),
        Some("NO") => Ok(false),
        Some(other) => Err(format!("invalid al80d {name} value: {other}")),
        None => Err(format!("missing al80d {name} value")),
    }
}

fn parse_status(response: &str) -> Result<DeviceStatus, String> {
    let fields = parse_fields(response);

    let connected = parse_yes_no(field(&fields, "connected"), "connected")?;

    if !connected {
        return Ok(DeviceStatus::offline("al80d reports keyboard offline"));
    }

    let scan = field(&fields, "scan_hz")
        .ok_or_else(|| "missing al80d scan_hz".to_string())?
        .parse::<u32>()
        .map_err(|error| format!("invalid al80d scan_hz: {error}"))?;

    if scan == 0 {
        return Err("al80d scan_hz cannot be zero".to_string());
    }

    let rgb = parse_on_off(field(&fields, "rgb"), "rgb")?;

    let overlay = parse_on_off(field(&fields, "overlay"), "overlay")?;

    let overlay_rgb = parse_on_off(field(&fields, "overlay_rgb"), "overlay_rgb")?;

    Ok(DeviceStatus {
        connected: true,
        devnode: field(&fields, "devnode").map(str::to_string),
        matrix_scan_hz: Some(scan),
        matrix_scan_interval_us: Some(1_000_000.0 / scan as f64),
        rgb_core_enabled: Some(rgb),
        overlay_enabled: Some(overlay),
        overlay_reports_rgb_core: Some(overlay_rgb),
        error: None,
    })
}

fn parse_capabilities(response: &str) -> Result<Capabilities, String> {
    let fields = parse_fields(response);

    let api = field(&fields, "api")
        .ok_or_else(|| "missing capability api".to_string())?
        .parse::<u32>()
        .map_err(|error| format!("invalid capability api: {error}"))?;

    let scan_hz = field(&fields, "scan_hz")
        .map(|value| value.parse::<u32>())
        .transpose()
        .map_err(|error| format!("invalid capability scan_hz: {error}"))?;

    Ok(Capabilities {
        api,
        daemon_version: field(&fields, "daemon").unwrap_or("unknown").to_string(),
        firmware_mode: field(&fields, "firmware").unwrap_or("UNKNOWN").to_string(),
        matrix_scan: parse_yes_no(field(&fields, "matrix_scan"), "matrix_scan")?,
        rgb_runtime: parse_yes_no(field(&fields, "rgb_runtime"), "rgb_runtime")?,
        overlay: parse_yes_no(field(&fields, "overlay"), "overlay")?,
        lcd_osd: parse_yes_no(field(&fields, "lcd_osd"), "lcd_osd")?,
        lcd_feedback: parse_yes_no(field(&fields, "lcd_feedback"), "lcd_feedback")?,
        audio_watch: parse_yes_no(field(&fields, "audio_watch"), "audio_watch")?,
        profiles: parse_yes_no(field(&fields, "profiles"), "profiles")?,
        extension_manifest: field(&fields, "extension_manifest")
            .unwrap_or("NONE")
            .to_string(),
        per_key_rgb: parse_yes_no(field(&fields, "per_key_rgb"), "per_key_rgb")?,
        creator_scene: parse_yes_no(field(&fields, "creator_scene"), "creator_scene")?,
        rgb_leds: field(&fields, "rgb_leds")
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|error| format!("invalid capability rgb_leds: {error}"))?,
        key_rgb_leds: field(&fields, "key_rgb_leds")
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|error| format!("invalid capability key_rgb_leds: {error}"))?,
        accent_rgb_leds: field(&fields, "accent_rgb_leds")
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|error| format!("invalid capability accent_rgb_leds: {error}"))?,
        creator_scene_state: match field(&fields, "creator_scene_state") {
            Some(value) => Some(parse_on_off(Some(value), "creator_scene_state")?),
            None => None,
        },
        input_router: parse_yes_no(field(&fields, "input_router"), "input_router")?,
        input_bindings: field(&fields, "input_bindings")
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|error| format!("invalid capability input_bindings: {error}"))?,
        input_actions: field(&fields, "input_actions")
            .unwrap_or("0")
            .parse::<u32>()
            .map_err(|error| format!("invalid capability input_actions: {error}"))?,
        input_router_state: match field(&fields, "input_router_state") {
            Some(value) => Some(parse_on_off(Some(value), "input_router_state")?),
            None => None,
        },
        input_event_bridge_host: parse_yes_no(
            field(&fields, "input_event_bridge_host"),
            "input_event_bridge_host",
        )?,
        input_event_firmware: parse_yes_no(
            field(&fields, "input_event_firmware"),
            "input_event_firmware",
        )?,
        input_event_auto_lcd: parse_yes_no(
            field(&fields, "input_event_auto_lcd"),
            "input_event_auto_lcd",
        )?,
        persistent_write: parse_yes_no(field(&fields, "persistent_write"), "persistent_write")?,
        eeprom_write: parse_yes_no(field(&fields, "eeprom_write"), "eeprom_write")?,
        qmk_flash: parse_yes_no(field(&fields, "qmk_flash"), "qmk_flash")?,
        scan_hz,
        rgb_state: match field(&fields, "rgb_state") {
            Some(value) => Some(parse_on_off(Some(value), "rgb_state")?),
            None => None,
        },
        overlay_state: match field(&fields, "overlay_state") {
            Some(value) => Some(parse_on_off(Some(value), "overlay_state")?),
            None => None,
        },
        overlay_rgb_state: match field(&fields, "overlay_rgb_state") {
            Some(value) => Some(parse_on_off(Some(value), "overlay_rgb_state")?),
            None => None,
        },
        error: None,
    })
}

const HOST_LIBRARY_MAX_BYTES: usize = 4 * 1024 * 1024;

fn host_library_file_name(library: &str) -> Result<&'static str, String> {
    match library {
        "creator-scenes-v1" => Ok("creator-scenes-v1.json"),
        "input-profiles-v1" => Ok("input-profiles-v1.json"),
        "host-profiles-v1" => Ok("host-profiles-v1.json"),
        _ => Err(format!("Host library is not allowlisted: {library}")),
    }
}

fn host_library_root() -> Result<std::path::PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var_os("LOCALAPPDATA")
            .ok_or_else(|| "LOCALAPPDATA is unavailable".to_string())?;
        return Ok(std::path::PathBuf::from(base)
            .join("AL80 Studio")
            .join("host-library-v1"));
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| "HOME is unavailable".to_string())?;
        return Ok(std::path::PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("AL80 Studio")
            .join("host-library-v1"));
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let base = match std::env::var_os("XDG_DATA_HOME") {
            Some(value) => std::path::PathBuf::from(value),
            None => {
                let home = std::env::var_os("HOME")
                    .ok_or_else(|| "HOME is unavailable".to_string())?;
                std::path::PathBuf::from(home).join(".local").join("share")
            }
        };

        return Ok(base.join("al80-studio").join("host-library-v1"));
    }

    #[allow(unreachable_code)]
    Err("Unsupported platform for Host Library Persistence V1".to_string())
}

fn host_library_path(library: &str) -> Result<std::path::PathBuf, String> {
    let file_name = host_library_file_name(library)?;
    Ok(host_library_root()?.join(file_name))
}

fn validate_host_library_json(json: &str) -> Result<(), String> {
    if json.len() > HOST_LIBRARY_MAX_BYTES {
        return Err(format!(
            "Host library payload exceeds {} bytes",
            HOST_LIBRARY_MAX_BYTES
        ));
    }

    let trimmed = json.trim();

    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return Err("Host library payload must be a JSON array".to_string());
    }

    Ok(())
}

#[tauri::command]
fn read_host_library(library: String) -> Result<Option<String>, String> {
    let path = host_library_path(&library)?;

    match std::fs::read_to_string(&path) {
        Ok(value) => {
            validate_host_library_json(&value)?;
            Ok(Some(value))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "Failed to read host library {}: {error}",
            path.display()
        )),
    }
}

#[tauri::command]
fn write_host_library(library: String, json: String) -> Result<String, String> {
    validate_host_library_json(&json)?;

    let path = host_library_path(&library)?;
    let root = path
        .parent()
        .ok_or_else(|| "Host library path has no parent".to_string())?;

    std::fs::create_dir_all(root).map_err(|error| {
        format!(
            "Failed to create host library directory {}: {error}",
            root.display()
        )
    })?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Host library file name is invalid".to_string())?;

    let temp = root.join(format!(".{file_name}.tmp-{}", std::process::id()));

    std::fs::write(&temp, json.as_bytes()).map_err(|error| {
        format!(
            "Failed to write temporary host library {}: {error}",
            temp.display()
        )
    })?;

    #[cfg(target_os = "windows")]
    if path.exists() {
        std::fs::remove_file(&path).map_err(|error| {
            format!(
                "Failed to replace host library {}: {error}",
                path.display()
            )
        })?;
    }

    std::fs::rename(&temp, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temp);
        format!(
            "Failed to publish host library {}: {error}",
            path.display()
        )
    })?;

    Ok(format!(
        "OK host_library=V1 library={} bytes={}",
        library,
        json.len()
    ))
}

#[tauri::command]
fn host_library_status() -> Result<String, String> {
    let root = host_library_root()?;

    let exists = |library: &str| -> Result<&'static str, String> {
        Ok(if host_library_path(library)?.is_file() {
            "YES"
        } else {
            "NO"
        })
    };

    Ok(format!(
        "OK host_library=V1 creator={} input={} profiles={} root={}",
        exists("creator-scenes-v1")?,
        exists("input-profiles-v1")?,
        exists("host-profiles-v1")?,
        root.display()
    ))
}

#[tauri::command]
fn get_device_status() -> DeviceStatus {
    match ipc_request("STATUS").and_then(|response| parse_status(&response)) {
        Ok(status) => status,
        Err(error) => DeviceStatus::offline(error),
    }
}

#[tauri::command]
fn get_capabilities() -> Capabilities {
    match ipc_request("CAPABILITIES").and_then(|response| parse_capabilities(&response)) {
        Ok(capabilities) => capabilities,
        Err(error) => Capabilities::offline(error),
    }
}

#[tauri::command]
fn set_rgb_core_runtime(enabled: bool) -> Result<bool, String> {
    let request = if enabled { "RGB ON" } else { "RGB OFF" };

    let response = ipc_request(request)?;

    match response.as_str() {
        "OK rgb=ON" => Ok(true),
        "OK rgb=OFF" => Ok(false),
        other => Err(format!("unexpected al80d RGB response: {other}")),
    }
}

#[tauri::command]
fn set_overlay_runtime(enabled: bool) -> Result<bool, String> {
    let request = if enabled { "OVERLAY ON" } else { "OVERLAY OFF" };

    let response = ipc_request(request)?;
    let fields = parse_fields(&response);

    parse_on_off(field(&fields, "overlay"), "overlay")
}

#[tauri::command]
fn run_safe_extension_command(command: String) -> Result<String, String> {
    match command.as_str() {
        "OVERLAY ON" | "OVERLAY OFF" | "RGB ON" | "RGB OFF" | "LCD HOME" => ipc_request(&command),

        _ => Err(format!(
            "extension command is not allowed in Manifest V1: {command}"
        )),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InputBindingRequest {
    event: u8,
    trigger: u8,
    trigger_a: u8,
    trigger_b: u8,
    action: u8,
}

fn validate_input_binding_request(
    index: usize,
    binding: &InputBindingRequest,
) -> Result<(), String> {
    if !(1..=3).contains(&binding.event) {
        return Err(format!(
            "binding {index} has invalid event {}",
            binding.event
        ));
    }

    if binding.trigger > 3 {
        return Err(format!(
            "binding {index} has invalid trigger {}",
            binding.trigger
        ));
    }

    if binding.action > 24 {
        return Err(format!(
            "binding {index} has invalid action {}",
            binding.action
        ));
    }

    match binding.trigger {
        0 => {
            if binding.trigger_a != 0 || binding.trigger_b != 0 {
                return Err(format!("binding {index}: Always trigger requires A=0/B=0"));
            }
        }
        1 => {
            if binding.trigger_a >= 32 || binding.trigger_b != 0 {
                return Err(format!("binding {index}: Layer requires 0..31 and B=0"));
            }
        }
        2 => {
            // Matrix bounds are authoritatively validated by firmware.
            // The normal GUI obtains row/column from the recovered layout.
        }
        3 => {
            if binding.trigger_a == 0 || binding.trigger_b != 0 {
                return Err(format!(
                    "binding {index}: Modifiers requires nonzero mask and B=0"
                ));
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

#[tauri::command]
fn get_input_router_status() -> Result<String, String> {
    ipc_request("INPUT STATUS")
}

#[tauri::command]
fn get_input_router_dump() -> Result<String, String> {
    ipc_request("INPUT DUMP")
}

#[tauri::command]
fn get_input_event_status() -> Result<String, String> {
    ipc_request("INPUT EVENTS")
}

#[tauri::command]
fn disable_input_router() -> Result<String, String> {
    ipc_request("INPUT OFF")
}

#[tauri::command]
fn restore_input_defaults() -> Result<String, String> {
    ipc_request("INPUT DEFAULTS")
}

#[tauri::command]
fn apply_input_profile(bindings: Vec<InputBindingRequest>) -> Result<String, String> {
    if bindings.is_empty() {
        return Err("Input profile must contain at least one binding".to_string());
    }

    if bindings.len() > 12 {
        return Err(format!(
            "Input profile accepts at most 12 bindings, got {}",
            bindings.len()
        ));
    }

    let mut encoded = Vec::with_capacity(bindings.len());

    for (index, binding) in bindings.iter().enumerate() {
        validate_input_binding_request(index, binding)?;

        encoded.push(format!(
            "{},{},{},{},{}",
            binding.event, binding.trigger, binding.trigger_a, binding.trigger_b, binding.action,
        ));
    }

    ipc_request(&format!("INPUT APPLY {}", encoded.join(";")))
}

#[tauri::command]
fn apply_creator_scene(colors: Vec<String>) -> Result<String, String> {
    if colors.len() != 82 {
        return Err(format!(
            "Creator Scene requires 82 colors, got {}",
            colors.len()
        ));
    }
    let mut encoded = String::with_capacity(82 * 6);
    for (index, color) in colors.iter().enumerate() {
        let normalized = color.trim().trim_start_matches('#');
        if normalized.len() != 6 || !normalized.bytes().all(|v| v.is_ascii_hexdigit()) {
            return Err(format!("invalid RGB value at LED {}: {}", index, color));
        }
        encoded.push_str(&normalized.to_ascii_lowercase());
    }
    ipc_request(&format!("SCENE APPLY {encoded}"))
}

#[tauri::command]
fn disable_creator_scene() -> Result<String, String> {
    ipc_request("SCENE OFF")
}

#[tauri::command]
fn get_creator_scene_status() -> Result<String, String> {
    ipc_request("SCENE STATUS")
}

#[tauri::command]
fn lcd_feedback(kind: String, value: String) -> Result<String, String> {
    let feedback = LcdFeedback::parse(&kind, &value)?;

    ipc_request(&format!(
        "LCD FEEDBACK {} {}",
        feedback.kind_token(),
        feedback.value_token(),
    ))
}

#[tauri::command]
fn lcd_home() -> Result<(), String> {
    let response = ipc_request("LCD HOME")?;

    if response == "OK lcd=HOME" {
        Ok(())
    } else {
        Err(format!("unexpected al80d LCD HOME response: {response}"))
    }
}

#[tauri::command]
fn lcd_preview(percent: u8, muted: bool) -> Result<String, String> {
    if percent > 100 {
        return Err("LCD preview percent must be between 0 and 100".to_string());
    }

    let request = if muted {
        format!("LCD MUTE {percent}")
    } else {
        format!("LCD VOLUME {percent}")
    };

    ipc_request(&request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_device_status,
            get_capabilities,
            read_host_library,
            write_host_library,
            host_library_status,
            set_rgb_core_runtime,
            set_overlay_runtime,
            run_safe_extension_command,
            get_input_router_status,
            get_input_router_dump,
            get_input_event_status,
            disable_input_router,
            restore_input_defaults,
            apply_input_profile,
            apply_creator_scene,
            disable_creator_scene,
            get_creator_scene_status,
            lcd_feedback,
            lcd_home,
            lcd_preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running AL80 Studio");
}
