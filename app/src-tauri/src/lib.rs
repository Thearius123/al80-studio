use std::env;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use serde::Serialize;

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
    audio_watch: bool,
    profiles: bool,
    extension_manifest: String,
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
            audio_watch: false,
            profiles: false,
            extension_manifest: "NONE".to_string(),
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

    let mut stream = UnixStream::connect(&path).map_err(|error| {
        format!(
            "cannot connect to al80d at {}: {error}",
            path.display()
        )
    })?;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| {
            format!("cannot configure al80d read timeout: {error}")
        })?;

    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| {
            format!("cannot configure al80d write timeout: {error}")
        })?;

    writeln!(stream, "{request}").map_err(|error| {
        format!("cannot write al80d request: {error}")
    })?;

    stream.flush().map_err(|error| {
        format!("cannot flush al80d request: {error}")
    })?;

    let mut response = String::new();
    let mut reader = BufReader::new(stream);

    reader.read_line(&mut response).map_err(|error| {
        format!("cannot read al80d response: {error}")
    })?;

    let response = response.trim().to_string();

    if response.is_empty() {
        return Err("al80d returned an empty response".to_string());
    }

    if let Some(error) = response.strip_prefix("ERR ") {
        return Err(format!("al80d: {error}"));
    }

    if !response.starts_with("OK") {
        return Err(format!(
            "unexpected al80d response: {response}"
        ));
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

fn field<'a>(
    fields: &'a [(&'a str, &'a str)],
    name: &str,
) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
}

fn parse_on_off(
    value: Option<&str>,
    name: &str,
) -> Result<bool, String> {
    match value {
        Some("ON") => Ok(true),
        Some("OFF") => Ok(false),
        Some(other) => Err(format!(
            "invalid al80d {name} value: {other}"
        )),
        None => Err(format!(
            "missing al80d {name} value"
        )),
    }
}

fn parse_yes_no(
    value: Option<&str>,
    name: &str,
) -> Result<bool, String> {
    match value {
        Some("YES") => Ok(true),
        Some("NO") => Ok(false),
        Some(other) => Err(format!(
            "invalid al80d {name} value: {other}"
        )),
        None => Err(format!(
            "missing al80d {name} value"
        )),
    }
}

fn parse_status(response: &str) -> Result<DeviceStatus, String> {
    let fields = parse_fields(response);

    let connected = parse_yes_no(
        field(&fields, "connected"),
        "connected",
    )?;

    if !connected {
        return Ok(DeviceStatus::offline(
            "al80d reports keyboard offline",
        ));
    }

    let scan = field(&fields, "scan_hz")
        .ok_or_else(|| "missing al80d scan_hz".to_string())?
        .parse::<u32>()
        .map_err(|error| {
            format!("invalid al80d scan_hz: {error}")
        })?;

    if scan == 0 {
        return Err("al80d scan_hz cannot be zero".to_string());
    }

    let rgb = parse_on_off(
        field(&fields, "rgb"),
        "rgb",
    )?;

    let overlay = parse_on_off(
        field(&fields, "overlay"),
        "overlay",
    )?;

    let overlay_rgb = parse_on_off(
        field(&fields, "overlay_rgb"),
        "overlay_rgb",
    )?;

    Ok(DeviceStatus {
        connected: true,
        devnode: field(&fields, "devnode").map(str::to_string),
        matrix_scan_hz: Some(scan),
        matrix_scan_interval_us:
            Some(1_000_000.0 / scan as f64),
        rgb_core_enabled: Some(rgb),
        overlay_enabled: Some(overlay),
        overlay_reports_rgb_core: Some(overlay_rgb),
        error: None,
    })
}

fn parse_capabilities(
    response: &str,
) -> Result<Capabilities, String> {
    let fields = parse_fields(response);

    let api = field(&fields, "api")
        .ok_or_else(|| "missing capability api".to_string())?
        .parse::<u32>()
        .map_err(|error| {
            format!("invalid capability api: {error}")
        })?;

    let scan_hz = field(&fields, "scan_hz")
        .map(|value| value.parse::<u32>())
        .transpose()
        .map_err(|error| {
            format!("invalid capability scan_hz: {error}")
        })?;

    Ok(Capabilities {
        api,
        daemon_version:
            field(&fields, "daemon")
                .unwrap_or("unknown")
                .to_string(),
        firmware_mode:
            field(&fields, "firmware")
                .unwrap_or("UNKNOWN")
                .to_string(),
        matrix_scan: parse_yes_no(
            field(&fields, "matrix_scan"),
            "matrix_scan",
        )?,
        rgb_runtime: parse_yes_no(
            field(&fields, "rgb_runtime"),
            "rgb_runtime",
        )?,
        overlay: parse_yes_no(
            field(&fields, "overlay"),
            "overlay",
        )?,
        lcd_osd: parse_yes_no(
            field(&fields, "lcd_osd"),
            "lcd_osd",
        )?,
        audio_watch: parse_yes_no(
            field(&fields, "audio_watch"),
            "audio_watch",
        )?,
        profiles: parse_yes_no(
            field(&fields, "profiles"),
            "profiles",
        )?,
        extension_manifest:
            field(&fields, "extension_manifest")
                .unwrap_or("NONE")
                .to_string(),
        persistent_write: parse_yes_no(
            field(&fields, "persistent_write"),
            "persistent_write",
        )?,
        eeprom_write: parse_yes_no(
            field(&fields, "eeprom_write"),
            "eeprom_write",
        )?,
        qmk_flash: parse_yes_no(
            field(&fields, "qmk_flash"),
            "qmk_flash",
        )?,
        scan_hz,
        rgb_state: match field(&fields, "rgb_state") {
            Some(value) => Some(parse_on_off(
                Some(value),
                "rgb_state",
            )?),
            None => None,
        },
        overlay_state:
            match field(&fields, "overlay_state") {
                Some(value) => Some(parse_on_off(
                    Some(value),
                    "overlay_state",
                )?),
                None => None,
            },
        overlay_rgb_state:
            match field(&fields, "overlay_rgb_state") {
                Some(value) => Some(parse_on_off(
                    Some(value),
                    "overlay_rgb_state",
                )?),
                None => None,
            },
        error: None,
    })
}

#[tauri::command]
fn get_device_status() -> DeviceStatus {
    match ipc_request("STATUS").and_then(
        |response| parse_status(&response),
    ) {
        Ok(status) => status,
        Err(error) => DeviceStatus::offline(error),
    }
}

#[tauri::command]
fn get_capabilities() -> Capabilities {
    match ipc_request("CAPABILITIES").and_then(
        |response| parse_capabilities(&response),
    ) {
        Ok(capabilities) => capabilities,
        Err(error) => Capabilities::offline(error),
    }
}

#[tauri::command]
fn set_rgb_core_runtime(
    enabled: bool,
) -> Result<bool, String> {
    let request = if enabled {
        "RGB ON"
    } else {
        "RGB OFF"
    };

    let response = ipc_request(request)?;

    match response.as_str() {
        "OK rgb=ON" => Ok(true),
        "OK rgb=OFF" => Ok(false),
        other => Err(format!(
            "unexpected al80d RGB response: {other}"
        )),
    }
}

#[tauri::command]
fn set_overlay_runtime(
    enabled: bool,
) -> Result<bool, String> {
    let request = if enabled {
        "OVERLAY ON"
    } else {
        "OVERLAY OFF"
    };

    let response = ipc_request(request)?;
    let fields = parse_fields(&response);

    parse_on_off(
        field(&fields, "overlay"),
        "overlay",
    )
}

#[tauri::command]
fn lcd_home() -> Result<(), String> {
    let response = ipc_request("LCD HOME")?;

    if response == "OK lcd=HOME" {
        Ok(())
    } else {
        Err(format!(
            "unexpected al80d LCD HOME response: {response}"
        ))
    }
}

#[tauri::command]
fn lcd_preview(
    percent: u8,
    muted: bool,
) -> Result<String, String> {
    if percent > 100 {
        return Err(
            "LCD preview percent must be between 0 and 100"
                .to_string()
        );
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
        .invoke_handler(
            tauri::generate_handler![
                get_device_status,
                get_capabilities,
                set_rgb_core_runtime,
                set_overlay_runtime,
                lcd_home,
                lcd_preview
            ],
        )
        .run(
            tauri::generate_context!()
        )
        .expect(
            "error while running AL80 Studio"
        );
}
