use al80_core::Al80;
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

#[tauri::command]
fn get_device_status() -> DeviceStatus {
    let mut device = match Al80::connect() {
        Ok(device) => device,
        Err(error) => {
            return DeviceStatus::offline(error);
        }
    };

    let devnode =
        device
            .device_info()
            .devnode
            .display()
            .to_string();

    let scan = match device.scan_rate_hz() {
        Ok(value) => value,
        Err(error) => {
            return DeviceStatus::offline(error);
        }
    };

    let rgb = match device.rgb_core_enabled() {
        Ok(value) => value,
        Err(error) => {
            return DeviceStatus::offline(error);
        }
    };

    let overlay = match device.overlay_status() {
        Ok(value) => value,
        Err(error) => {
            return DeviceStatus::offline(error);
        }
    };

    DeviceStatus {
        connected: true,
        devnode: Some(devnode),
        matrix_scan_hz: Some(scan),
        matrix_scan_interval_us:
            Some(1_000_000.0 / scan as f64),
        rgb_core_enabled: Some(rgb),
        overlay_enabled: Some(overlay.enabled),
        overlay_reports_rgb_core:
            Some(overlay.rgb_core_enabled),
        error: None,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(
            tauri::generate_handler![
                get_device_status
            ],
        )
        .run(
            tauri::generate_context!()
        )
        .expect(
            "error while running AL80 Studio"
        );
}
