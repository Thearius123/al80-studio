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

fn parse_on_off(
    value: Option<&str>,
    field: &str,
) -> Result<bool, String> {
    match value {
        Some("ON") => Ok(true),
        Some("OFF") => Ok(false),
        Some(other) => Err(format!(
            "invalid al80d {field} value: {other}"
        )),
        None => Err(format!(
            "missing al80d {field} value"
        )),
    }
}

fn status_field<'a>(
    fields: &'a [(&'a str, &'a str)],
    name: &str,
) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, value)| *value)
}

fn parse_status(response: &str) -> Result<DeviceStatus, String> {
    let mut parsed = Vec::new();

    for token in response.split_whitespace().skip(1) {
        if let Some((key, value)) = token.split_once('=') {
            parsed.push((key, value));
        }
    }

    let connected = match status_field(&parsed, "connected") {
        Some("YES") => true,
        Some("NO") => false,
        Some(other) => {
            return Err(format!(
                "invalid al80d connected value: {other}"
            ));
        }
        None => {
            return Err(
                "missing al80d connected value".to_string()
            );
        }
    };

    if !connected {
        return Ok(DeviceStatus::offline(
            "al80d reports keyboard offline",
        ));
    }

    let devnode = status_field(&parsed, "devnode")
        .map(str::to_string);

    let scan = status_field(&parsed, "scan_hz")
        .ok_or_else(|| "missing al80d scan_hz".to_string())?
        .parse::<u32>()
        .map_err(|error| {
            format!("invalid al80d scan_hz: {error}")
        })?;

    if scan == 0 {
        return Err("al80d scan_hz cannot be zero".to_string());
    }

    let rgb = parse_on_off(
        status_field(&parsed, "rgb"),
        "rgb",
    )?;

    let overlay = parse_on_off(
        status_field(&parsed, "overlay"),
        "overlay",
    )?;

    let overlay_rgb = parse_on_off(
        status_field(&parsed, "overlay_rgb"),
        "overlay_rgb",
    )?;

    Ok(DeviceStatus {
        connected: true,
        devnode,
        matrix_scan_hz: Some(scan),
        matrix_scan_interval_us:
            Some(1_000_000.0 / scan as f64),
        rgb_core_enabled: Some(rgb),
        overlay_enabled: Some(overlay),
        overlay_reports_rgb_core: Some(overlay_rgb),
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(
            tauri::generate_handler![
                get_device_status,
                set_rgb_core_runtime
            ],
        )
        .run(
            tauri::generate_context!()
        )
        .expect(
            "error while running AL80 Studio"
        );
}
