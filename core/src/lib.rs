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
pub const CMD_CREATOR_SCENE: u8 = 0x4A;

pub const CREATOR_LED_COUNT: usize = 82;
pub const CREATOR_CHUNK_MAX: usize = 9;

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

#[derive(Debug, Clone, Copy)]
pub struct CreatorSceneStatus {
    pub enabled: bool,
    pub rgb_core_enabled: bool,
    pub led_count: u8,
    pub chunk_max: u8,
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

    /// Send one complete 32-byte vendor payload through Linux hidraw.
    ///
    /// LCD responses are returned raw because the currently known commands
    /// use command-specific ACK offsets.
    fn transact_raw32(
        &mut self,
        payload: &[u8; REPORT_BYTES],
        timeout: Duration,
    ) -> Result<(Vec<u8>, f64), String> {
        self.drain()?;

        let mut request = [0u8; LINUX_WRITE_BYTES];
        request[0] = 0;
        request[1..].copy_from_slice(payload);

        let started = Instant::now();
        self.write_request(&request)?;

        let deadline = started + timeout;
        let mut buffer = [0u8; 64];

        while Instant::now() < deadline {
            match self.file.read(&mut buffer) {
                Ok(0) => {
                    thread::sleep(Duration::from_micros(400));
                }
                Ok(count) => {
                    let elapsed_ms =
                        started.elapsed().as_secs_f64() * 1000.0;
                    return Ok((buffer[..count].to_vec(), elapsed_ms));
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_micros(400));
                }
                Err(e) => {
                    return Err(format!(
                        "Raw HID LCD read failed: {e}"
                    ));
                }
            }
        }

        Err("timeout waiting for LCD Raw HID response".to_string())
    }

    /// Return the keyboard LCD to its normal HOME screen.
    pub fn lcd_home(&mut self) -> Result<(), String> {
        const GO_HOME: [u8; 7] = [
            0xA5, 0x5A, 0x0B, 0x00, 0x00, 0x02, 0x00,
        ];

        let mut begin = [0u8; REPORT_BYTES];
        begin[0] = 0x40;
        begin[3] = GO_HOME.len() as u8;
        begin[7..7 + GO_HOME.len()].copy_from_slice(&GO_HOME);

        let (response, _) =
            self.transact_raw32(&begin, Duration::from_millis(500))?;

        if response.len() <= 6 || response[6] != STATUS_OK {
            let status = response.get(6).copied();
            return Err(format!(
                "LCD HOME begin ACK invalid: response[6]={}",
                status
                    .map(|value| format!("0x{value:02X}"))
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }

        let mut end = [0u8; REPORT_BYTES];
        end[0] = 0x42;

        let (response, _) =
            self.transact_raw32(&end, Duration::from_millis(500))?;

        if response.len() <= 6 || response[6] != STATUS_OK {
            let status = response.get(6).copied();
            return Err(format!(
                "LCD HOME end ACK invalid: response[6]={}",
                status
                    .map(|value| format!("0x{value:02X}"))
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }

        Ok(())
    }

    /// Show a volatile host volume/mute OSD on the keyboard LCD.
    pub fn lcd_volume_osd(
        &mut self,
        percent: u8,
        muted: bool,
    ) -> Result<f64, String> {
        if percent > 100 {
            return Err(format!(
                "LCD volume percent out of range: {percent}"
            ));
        }

        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = 0x43;
        payload[1] = percent;
        payload[2] = if muted { 1 } else { 0 };

        let (response, elapsed_ms) =
            self.transact_raw32(
                &payload,
                Duration::from_millis(500),
            )?;

        if response.len() <= 3 || response[3] != STATUS_OK {
            let status = response.get(3).copied();
            return Err(format!(
                "LCD volume ACK invalid: response[3]={}",
                status
                    .map(|value| format!("0x{value:02X}"))
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }

        Ok(elapsed_ms)
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

    fn creator_scene_command(
        &mut self,
        payload: &[u8; REPORT_BYTES],
    ) -> Result<Vec<u8>, String> {
        let (response, _) = self.transact_raw32(
            payload,
            Duration::from_millis(800),
        )?;

        let normalized: &[u8] =
            if response.len() >= LINUX_WRITE_BYTES && response[0] == 0 {
                &response[1..usize::min(LINUX_WRITE_BYTES, response.len())]
            } else {
                &response[0..usize::min(REPORT_BYTES, response.len())]
            };

        if normalized.len() < 2 {
            return Err("0x4A response too short".to_string());
        }

        if normalized[0] != CMD_CREATOR_SCENE {
            return Err(format!(
                "unexpected Creator Scene response command 0x{:02X}",
                normalized[0]
            ));
        }

        if normalized[1] != STATUS_OK {
            return Err(format!(
                "0x4A returned status 0x{:02X}",
                normalized[1]
            ));
        }

        Ok(normalized.to_vec())
    }

    pub fn creator_scene_status(
        &mut self,
    ) -> Result<CreatorSceneStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_CREATOR_SCENE;
        payload[1] = 0;
        let response = self.creator_scene_command(&payload)?;

        if response.len() < 9 {
            return Err("0x4A query response too short".to_string());
        }
        if response[3] as usize != CREATOR_LED_COUNT {
            return Err(format!(
                "0x4A reports unexpected LED count {}",
                response[3]
            ));
        }
        if response[4] as usize != CREATOR_CHUNK_MAX {
            return Err(format!(
                "0x4A reports unexpected chunk size {}",
                response[4]
            ));
        }

        Ok(CreatorSceneStatus {
            enabled: response[2] != 0,
            led_count: response[3],
            chunk_max: response[4],
            rgb_core_enabled: response[8] != 0,
        })
    }

    pub fn creator_scene_disable(
        &mut self,
    ) -> Result<CreatorSceneStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_CREATOR_SCENE;
        payload[1] = 1;
        let response = self.creator_scene_command(&payload)?;
        if response.len() < 3 || response[2] != 0 {
            return Err("Creator Scene disable ACK invalid".to_string());
        }
        self.creator_scene_status()
    }

    pub fn creator_scene_apply(
        &mut self,
        colors: &[[u8; 3]],
    ) -> Result<CreatorSceneStatus, String> {
        if colors.len() != CREATOR_LED_COUNT {
            return Err(format!(
                "Creator Scene requires exactly {} RGB values, got {}",
                CREATOR_LED_COUNT,
                colors.len()
            ));
        }

        self.set_rgb_core(true)?;

        let mut clear = [0u8; REPORT_BYTES];
        clear[0] = CMD_CREATOR_SCENE;
        clear[1] = 2;
        self.creator_scene_command(&clear)?;

        for start in (0..CREATOR_LED_COUNT).step_by(CREATOR_CHUNK_MAX) {
            let count = usize::min(
                CREATOR_CHUNK_MAX,
                CREATOR_LED_COUNT - start,
            );
            let mut payload = [0u8; REPORT_BYTES];
            payload[0] = CMD_CREATOR_SCENE;
            payload[1] = 3;
            payload[2] = start as u8;
            payload[3] = count as u8;

            for offset in 0..count {
                let target = 4 + offset * 3;
                let color = colors[start + offset];
                payload[target] = color[0];
                payload[target + 1] = color[1];
                payload[target + 2] = color[2];
            }

            let response = self.creator_scene_command(&payload)?;
            if response.len() < 8
                || response[6] != start as u8
                || response[7] != count as u8
            {
                return Err(format!(
                    "Creator Scene chunk ACK mismatch at LED {}",
                    start
                ));
            }
        }

        let mut commit = [0u8; REPORT_BYTES];
        commit[0] = CMD_CREATOR_SCENE;
        commit[1] = 4;
        let response = self.creator_scene_command(&commit)?;

        if response.len() < 3 || response[2] == 0 {
            return Err("Creator Scene commit did not enable scene".to_string());
        }

        self.creator_scene_status()
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
