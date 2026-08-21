use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

pub const VID: &str = "28e9";
pub const PID: &str = "30af";

pub const CMD_SCAN_RATE: u8 = 0x47;
pub const CMD_RGB_CORE: u8 = 0x48;
pub const CMD_OVERLAY: u8 = 0x49;

pub const STATUS_OK: u8 = 0x55;

const REPORT_BYTES: usize = 32;
const LINUX_WRITE_BYTES: usize = 33;
const O_NONBLOCK_LINUX: i32 = 0x800;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub devnode: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub struct OverlayStatus {
    pub enabled: bool,
    pub rgb_core_enabled: bool,
}

pub struct Al80 {
    file: File,
    info: DeviceInfo,
}

impl DeviceInfo {
    pub fn discover() -> Result<Self, String> {
        let sys = Path::new("/sys/class/hidraw");

        let entries = fs::read_dir(sys)
            .map_err(|e| format!("cannot read {}: {e}", sys.display()))?;

        let mut candidates = Vec::new();

        for entry in entries {
            let entry = match entry {
                Ok(value) => value,
                Err(_) => continue,
            };

            let name = entry.file_name();
            let class_path = entry.path();

            let device_path = match fs::canonicalize(class_path.join("device")) {
                Ok(value) => value,
                Err(_) => continue,
            };

            let device_text = device_path.to_string_lossy().to_ascii_lowercase();

            let identity_matches =
                device_text.contains(&format!("{VID}:{PID}"))
                    || device_text.contains(&format!("0003:{VID}:{PID}"));

            if !identity_matches {
                continue;
            }

            let descriptor = match fs::read(
                class_path
                    .join("device")
                    .join("report_descriptor"),
            ) {
                Ok(value) => value,
                Err(_) => continue,
            };

            let has_usage_page = descriptor
                .windows(3)
                .any(|w| w == [0x06, 0x60, 0xFF]);

            let has_usage =
                descriptor
                    .windows(2)
                    .any(|w| w == [0x09, 0x61])
                    || descriptor
                        .windows(3)
                        .any(|w| w == [0x0A, 0x61, 0x00]);

            if has_usage_page && has_usage {
                candidates.push(PathBuf::from("/dev").join(name));
            }
        }

        candidates.sort();
        candidates.dedup();

        if candidates.len() != 1 {
            return Err(format!(
                "expected exactly one AL80 Raw HID interface, found {}",
                candidates.len()
            ));
        }

        Ok(Self {
            devnode: candidates.remove(0),
        })
    }

    fn open(&self) -> Result<File, String> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(O_NONBLOCK_LINUX)
            .open(&self.devnode)
            .map_err(|e| {
                format!(
                    "cannot open {}: {e}",
                    self.devnode.display()
                )
            })
    }
}

impl Al80 {
    pub fn connect() -> Result<Self, String> {
        let info = DeviceInfo::discover()?;
        let file = info.open()?;

        Ok(Self { file, info })
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn drain(&mut self) -> Result<(), String> {
        let mut buffer = [0u8; 64];

        loop {
            match self.file.read(&mut buffer) {
                Ok(0) => return Ok(()),
                Ok(_) => continue,

                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    return Ok(());
                }

                Err(e) => {
                    return Err(format!("Raw HID drain failed: {e}"));
                }
            }
        }
    }

    fn write_request(
        &mut self,
        request: &[u8; LINUX_WRITE_BYTES],
    ) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(1);
        let mut offset = 0;

        while offset < request.len() {
            match self.file.write(&request[offset..]) {
                Ok(0) => {
                    return Err(
                        "Raw HID write returned zero bytes".to_string()
                    );
                }

                Ok(count) => {
                    offset += count;
                }

                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err("Raw HID write timeout".to_string());
                    }

                    thread::sleep(Duration::from_millis(2));
                }

                Err(e) => {
                    return Err(format!("Raw HID write failed: {e}"));
                }
            }
        }

        Ok(())
    }

    fn transact(
        &mut self,
        command: u8,
        argument: Option<u8>,
    ) -> Result<Vec<u8>, String> {
        self.drain()?;

        let mut request = [0u8; LINUX_WRITE_BYTES];

        // Linux hidraw:
        // request[0] = report ID 0
        // request[1] = QMK Raw HID data[0]
        request[0] = 0;
        request[1] = command;

        if let Some(value) = argument {
            request[2] = value;
        }

        self.write_request(&request)?;

        let deadline = Instant::now() + Duration::from_secs(1);
        let mut buffer = [0u8; 64];

        while Instant::now() < deadline {
            match self.file.read(&mut buffer) {
                Ok(0) => {
                    thread::sleep(Duration::from_millis(2));
                }

                Ok(count) => {
                    let payload: &[u8] =
                        if count >= LINUX_WRITE_BYTES && buffer[0] == 0 {
                            &buffer[
                                1..usize::min(
                                    LINUX_WRITE_BYTES,
                                    count,
                                )
                            ]
                        } else {
                            &buffer[
                                0..usize::min(
                                    REPORT_BYTES,
                                    count,
                                )
                            ]
                        };

                    if payload.len() < 2 {
                        continue;
                    }

                    if payload[0] != command {
                        continue;
                    }

                    if payload[1] != STATUS_OK {
                        return Err(format!(
                            "0x{command:02X} returned status 0x{:02X}",
                            payload[1]
                        ));
                    }

                    return Ok(payload.to_vec());
                }

                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(2));
                }

                Err(e) => {
                    return Err(format!("Raw HID read failed: {e}"));
                }
            }
        }

        Err(format!(
            "timeout waiting for 0x{command:02X} response"
        ))
    }

    pub fn scan_rate_hz(&mut self) -> Result<u32, String> {
        let payload = self.transact(CMD_SCAN_RATE, None)?;

        if payload.len() < 6 {
            return Err(format!(
                "0x47 response too short: {} bytes",
                payload.len()
            ));
        }

        let rate = u32::from_le_bytes([
            payload[2],
            payload[3],
            payload[4],
            payload[5],
        ]);

        if rate == 0 {
            return Err("0x47 returned zero scan rate".to_string());
        }

        Ok(rate)
    }

    pub fn rgb_core_enabled(&mut self) -> Result<bool, String> {
        let payload = self.transact(CMD_RGB_CORE, Some(2))?;

        if payload.len() < 3 {
            return Err("0x48 response too short".to_string());
        }

        Ok(payload[2] != 0)
    }

    pub fn set_rgb_core(
        &mut self,
        enabled: bool,
    ) -> Result<bool, String> {
        let payload = self.transact(
            CMD_RGB_CORE,
            Some(if enabled { 1 } else { 0 }),
        )?;

        if payload.len() < 3 {
            return Err("0x48 response too short".to_string());
        }

        Ok(payload[2] != 0)
    }

    pub fn overlay_status(
        &mut self,
    ) -> Result<OverlayStatus, String> {
        let payload = self.transact(CMD_OVERLAY, Some(2))?;

        if payload.len() < 4 {
            return Err("0x49 response too short".to_string());
        }

        Ok(OverlayStatus {
            enabled: payload[2] != 0,
            rgb_core_enabled: payload[3] != 0,
        })
    }

    pub fn set_overlay(
        &mut self,
        enabled: bool,
    ) -> Result<OverlayStatus, String> {
        let payload = self.transact(
            CMD_OVERLAY,
            Some(if enabled { 1 } else { 0 }),
        )?;

        if payload.len() < 4 {
            return Err("0x49 response too short".to_string());
        }

        Ok(OverlayStatus {
            enabled: payload[2] != 0,
            rgb_core_enabled: payload[3] != 0,
        })
    }
}
