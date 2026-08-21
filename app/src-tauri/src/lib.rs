use std::sync::Mutex;

use al80_core::Al80;
use serde::Serialize;
use tauri::State;

/// Serializes every AL80 Studio transaction that currently reaches the
/// keyboard.
///
/// Core V1 intentionally keeps connection lifetime short while moving all
/// application-side hardware access behind one abstraction. This gives RGB,
/// telemetry, LCD, knob support, profiles, CLI/API clients, and future
/// extensions one place to acquire device access.
///
/// IMPORTANT:
/// This is the first broker foundation, not yet system-wide exclusive
/// ownership. The legacy Python volume OSD service still opens Raw HID
/// directly until its known-good behavior is migrated into the broker.
#[derive(Default)]
struct DeviceBroker {
    transaction_gate: Mutex<()>,
}

impl DeviceBroker {
    fn with_device<T>(
        &self,
        operation: impl FnOnce(&mut Al80) -> Result<T, String>,
    ) -> Result<T, String> {
        let _transaction = self.transaction_gate.lock().map_err(|_| {
            "AL80 device broker transaction lock poisoned".to_string()
        })?;

        let mut device = Al80::connect()?;
        operation(&mut device)
    }
}

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

fn read_device_status(device: &mut Al80) -> Result<DeviceStatus, String> {
    let devnode = device.device_info().devnode.display().to_string();

    let scan = device.scan_rate_hz()?;
    let rgb = device.rgb_core_enabled()?;
    let overlay = device.overlay_status()?;

    Ok(DeviceStatus {
        connected: true,
        devnode: Some(devnode),
        matrix_scan_hz: Some(scan),
        matrix_scan_interval_us: Some(1_000_000.0 / scan as f64),
        rgb_core_enabled: Some(rgb),
        overlay_enabled: Some(overlay.enabled),
        overlay_reports_rgb_core: Some(overlay.rgb_core_enabled),
        error: None,
    })
}

#[tauri::command]
fn get_device_status(broker: State<'_, DeviceBroker>) -> DeviceStatus {
    match broker.with_device(read_device_status) {
        Ok(status) => status,
        Err(error) => DeviceStatus::offline(error),
    }
}

#[tauri::command]
fn set_rgb_core_runtime(
    enabled: bool,
    broker: State<'_, DeviceBroker>,
) -> Result<bool, String> {
    broker.with_device(|device| device.set_rgb_core(enabled))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DeviceBroker::default())
        .invoke_handler(tauri::generate_handler![
            get_device_status,
            set_rgb_core_runtime
        ])
        .run(tauri::generate_context!())
        .expect("error while running AL80 Studio");
}
