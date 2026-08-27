pub mod input_event_bridge;
pub mod raw_hid_session;

pub mod auto_lcd_feedback;
pub mod lcd_feedback;

use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const VID: &str = "28e9";
pub const PID: &str = "30af";

pub const CMD_SCAN_RATE: u8 = 0x47;
pub const CMD_RGB_CORE: u8 = 0x48;
pub const CMD_OVERLAY: u8 = 0x49;
pub const CMD_CREATOR_SCENE: u8 = 0x4A;
pub const CMD_INPUT_ROUTER: u8 = 0x4B;

pub const CREATOR_LED_COUNT: usize = 82;
pub const CREATOR_CHUNK_MAX: usize = 9;
pub const INPUT_ROUTER_VERSION: u8 = 1;
pub const INPUT_BINDING_MAX: usize = 12;
pub const INPUT_ACTION_MAX: u8 = 24;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputEvent {
    KnobCcw = 1,
    KnobCw = 2,
    KnobPress = 3,
}

impl TryFrom<u8> for InputEvent {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::KnobCcw),
            2 => Ok(Self::KnobCw),
            3 => Ok(Self::KnobPress),
            other => Err(format!("invalid Input Router event {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum InputTrigger {
    None = 0,
    Layer = 1,
    Matrix = 2,
    Modifiers = 3,
}

impl TryFrom<u8> for InputTrigger {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Layer),
            2 => Ok(Self::Matrix),
            3 => Ok(Self::Modifiers),
            other => Err(format!("invalid Input Router trigger {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputAction(u8);

impl InputAction {
    pub const NONE: Self = Self(0);
    pub const VOLUME_DOWN: Self = Self(1);
    pub const VOLUME_UP: Self = Self(2);
    pub const MUTE: Self = Self(3);
    pub const MEDIA_PREVIOUS: Self = Self(4);
    pub const MEDIA_NEXT: Self = Self(5);
    pub const MEDIA_PLAY_PAUSE: Self = Self(6);
    pub const BRIGHTNESS_DOWN: Self = Self(7);
    pub const BRIGHTNESS_UP: Self = Self(8);
    pub const LEFT: Self = Self(9);
    pub const RIGHT: Self = Self(10);
    pub const UP: Self = Self(11);
    pub const DOWN: Self = Self(12);
    pub const PAGE_UP: Self = Self(13);
    pub const PAGE_DOWN: Self = Self(14);
    pub const RGB_VALUE_DOWN: Self = Self(15);
    pub const RGB_VALUE_UP: Self = Self(16);
    pub const RGB_HUE_DOWN: Self = Self(17);
    pub const RGB_HUE_UP: Self = Self(18);
    pub const RGB_SPEED_DOWN: Self = Self(19);
    pub const RGB_SPEED_UP: Self = Self(20);
    pub const SNAKE_OFF: Self = Self(21);
    pub const SNAKE_ON: Self = Self(22);
    pub const SNAKE_TOGGLE: Self = Self(23);
    pub const CREATOR_SCENE_OFF: Self = Self(24);

    pub fn from_id(value: u8) -> Result<Self, String> {
        if value <= INPUT_ACTION_MAX {
            Ok(Self(value))
        } else {
            Err(format!("invalid Input Router action {value}"))
        }
    }

    pub fn id(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputBinding {
    pub event: InputEvent,
    pub trigger: InputTrigger,
    pub trigger_a: u8,
    pub trigger_b: u8,
    pub action: InputAction,
}

impl InputBinding {
    pub fn new(
        event: InputEvent,
        trigger: InputTrigger,
        trigger_a: u8,
        trigger_b: u8,
        action: InputAction,
    ) -> Result<Self, String> {
        match trigger {
            InputTrigger::None => {
                if trigger_a != 0 || trigger_b != 0 {
                    return Err("NONE trigger requires A=0 and B=0".to_string());
                }
            }
            InputTrigger::Layer => {
                if trigger_a >= 32 || trigger_b != 0 {
                    return Err("LAYER trigger requires layer 0..31 and B=0".to_string());
                }
            }
            InputTrigger::Matrix => {}
            InputTrigger::Modifiers => {
                if trigger_a == 0 || trigger_b != 0 {
                    return Err("MODIFIERS trigger requires a non-zero mask and B=0".to_string());
                }
            }
        }

        Ok(Self {
            event,
            trigger,
            trigger_a,
            trigger_b,
            action,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputRouterStatus {
    pub enabled: bool,
    pub version: u8,
    pub binding_slots: u8,
    pub max_action: u8,
    pub fallback_supported: bool,
}

pub const LIVE_RGB_LED_COUNT: usize = 82;
const LIVE_RGB_TELEMETRY_COMMAND: u8 = 0x4D;
const LIVE_RGB_TELEMETRY_VERSION: u8 = 1;
const LIVE_RGB_TELEMETRY_CHUNK: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveRgbTelemetry {
    pub version: u8,
    pub source: u8,
    pub frame_valid: bool,
    pub rgb_core_enabled: bool,
    pub overlay_enabled: bool,
    pub creator_scene_enabled: bool,
    pub colors: Vec<[u8; 3]>,
}

pub struct Al80 {
    session: raw_hid_session::RawHidSession,
    info: DeviceInfo,
}

impl DeviceInfo {
    pub fn discover() -> Result<Self, String> {
        let sys = Path::new("/sys/class/hidraw");

        let entries =
            fs::read_dir(sys).map_err(|e| format!("cannot read {}: {e}", sys.display()))?;

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

            let identity_matches = device_text.contains(&format!("{VID}:{PID}"))
                || device_text.contains(&format!("0003:{VID}:{PID}"));

            if !identity_matches {
                continue;
            }

            let descriptor = match fs::read(class_path.join("device").join("report_descriptor")) {
                Ok(value) => value,
                Err(_) => continue,
            };

            let has_usage_page = descriptor.windows(3).any(|w| w == [0x06, 0x60, 0xFF]);

            let has_usage = descriptor.windows(2).any(|w| w == [0x09, 0x61])
                || descriptor.windows(3).any(|w| w == [0x0A, 0x61, 0x00]);

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
            .map_err(|e| format!("cannot open {}: {e}", self.devnode.display()))
    }
}

impl Al80 {
    pub fn connect() -> Result<Self, String> {
        let info = DeviceInfo::discover()?;
        let file = info.open()?;
        let session = raw_hid_session::RawHidSession::new(file)?;

        Ok(Self { session, info })
    }

    pub fn device_info(&self) -> &DeviceInfo {
        &self.info
    }

    fn transact(&mut self, command: u8, argument: Option<u8>) -> Result<Vec<u8>, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = command;

        if let Some(value) = argument {
            payload[1] = value;
        }

        let (response, _) = self.session.transact(&payload, Duration::from_secs(1))?;

        if response[1] != STATUS_OK {
            return Err(format!(
                "0x{command:02X} returned status 0x{:02X}",
                response[1]
            ));
        }

        Ok(response.to_vec())
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
        let (response, elapsed_ms) = self.session.transact(payload, timeout)?;

        Ok((response.to_vec(), elapsed_ms))
    }

    pub fn pop_input_event(&self) -> Result<Option<raw_hid_session::HostInputEvent>, String> {
        self.session.pop_input_event()
    }

    pub fn queued_input_events(&self) -> Result<usize, String> {
        self.session.queued_input_events()
    }

    pub fn raw_hid_session_stats(&self) -> raw_hid_session::RawHidSessionStats {
        self.session.stats()
    }

    /// Return the keyboard LCD to its normal HOME screen.
    pub fn lcd_home(&mut self) -> Result<(), String> {
        const GO_HOME: [u8; 7] = [0xA5, 0x5A, 0x0B, 0x00, 0x00, 0x02, 0x00];

        let mut begin = [0u8; REPORT_BYTES];
        begin[0] = 0x40;
        begin[3] = GO_HOME.len() as u8;
        begin[7..7 + GO_HOME.len()].copy_from_slice(&GO_HOME);

        let (response, _) = self.transact_raw32(&begin, Duration::from_millis(500))?;

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

        let (response, _) = self.transact_raw32(&end, Duration::from_millis(500))?;

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
    pub fn lcd_volume_osd(&mut self, percent: u8, muted: bool) -> Result<f64, String> {
        if percent > 100 {
            return Err(format!("LCD volume percent out of range: {percent}"));
        }

        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = 0x43;
        payload[1] = percent;
        payload[2] = if muted { 1 } else { 0 };

        let (response, elapsed_ms) = self.transact_raw32(&payload, Duration::from_millis(500))?;

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

    fn lcd_bridge_packet(&mut self, command: u8, offset: u16, data: &[u8]) -> Result<f64, String> {
        const MAX_LCD_BRIDGE_DATA: usize = 25;

        if data.len() > MAX_LCD_BRIDGE_DATA {
            return Err(format!(
                "LCD bridge chunk too large: {} > {}",
                data.len(),
                MAX_LCD_BRIDGE_DATA
            ));
        }

        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = command;
        payload[1] = (offset & 0xFF) as u8;
        payload[2] = (offset >> 8) as u8;
        payload[3] = data.len() as u8;
        payload[7..7 + data.len()].copy_from_slice(data);

        let (response, elapsed_ms) = self.transact_raw32(&payload, Duration::from_millis(750))?;

        if response.len() <= 6 || response[6] != STATUS_OK {
            let status = response.get(6).copied();

            return Err(format!(
                "LCD bridge 0x{command:02X} ACK invalid: {}",
                status
                    .map(|value| format!("0x{value:02X}"))
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }

        Ok(elapsed_ms)
    }

    fn lcd_bridge_finish(&mut self) -> Result<f64, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = 0x42;

        let (response, elapsed_ms) = self.transact_raw32(&payload, Duration::from_millis(750))?;

        if response.len() <= 6 || response[6] != STATUS_OK {
            let status = response.get(6).copied();

            return Err(format!(
                "LCD bridge FINISH ACK invalid: {}",
                status
                    .map(|value| format!("0x{value:02X}"))
                    .unwrap_or_else(|| "missing".to_string())
            ));
        }

        Ok(elapsed_ms)
    }

    /// Stream one typed, volatile native RGB565 feedback frame.
    /// Stream one complete typed, volatile native RGB565 feedback frame.
    pub fn lcd_generic_feedback(
        &mut self,
        feedback: &crate::lcd_feedback::LcdFeedback,
    ) -> Result<crate::lcd_feedback::LcdFeedbackTransfer, String> {
        self.lcd_generic_feedback_until(feedback, || true)
    }

    /// Stream typed LCD feedback while `keep_going` remains true.
    ///
    /// The bridge finish sequence is preserved after a started transfer,
    /// including cancellation after GUI_EVENT/ADD_PIC/data chunks.
    pub fn lcd_generic_feedback_until<F>(
        &mut self,
        feedback: &crate::lcd_feedback::LcdFeedback,
        mut keep_going: F,
    ) -> Result<crate::lcd_feedback::LcdFeedbackTransfer, String>
    where
        F: FnMut() -> bool,
    {
        const GUI_EVENT: [u8; 8] = [0xA5, 0x5A, 0x10, 0x00, 0x01, 0xC5, 0xB1, 0x01];

        const ADD_PIC: [u8; 7] = [0xA5, 0x5A, 0x0C, 0x78, 0x00, 0xC3, 0x93];

        const CHUNK: usize = 25;

        let frame = crate::lcd_feedback::render_feedback_rgb565(*feedback);

        if frame.len() != crate::lcd_feedback::LCD_FRAME_BYTES {
            return Err(format!("LCD feedback frame size mismatch: {}", frame.len()));
        }

        let started = std::time::Instant::now();
        let mut begun = false;
        let mut chunks = 0usize;
        let mut cancelled = false;

        let result = (|| -> Result<(), String> {
            if !keep_going() {
                cancelled = true;
                return Ok(());
            }

            self.lcd_bridge_packet(0x40, 0, &GUI_EVENT)?;

            begun = true;

            std::thread::sleep(Duration::from_millis(150));

            if !keep_going() {
                cancelled = true;
                return Ok(());
            }

            self.lcd_bridge_packet(0x41, 0, &ADD_PIC)?;

            chunks += 1;

            for (index, bytes) in frame.chunks(CHUNK).enumerate() {
                if !keep_going() {
                    cancelled = true;
                    break;
                }

                let offset = index * CHUNK;

                self.lcd_bridge_packet(0x41, offset as u16, bytes)?;

                chunks += 1;
            }

            Ok(())
        })();

        if begun {
            let finish = self.lcd_bridge_finish();

            match (&result, finish) {
                (Ok(()), Err(error)) => {
                    return Err(error);
                }

                (Err(_), _) | (Ok(()), Ok(_)) => {}
            }
        }

        result?;

        Ok(crate::lcd_feedback::LcdFeedbackTransfer {
            bytes: frame.len(),
            chunks,
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
            cancelled,
        })
    }

    pub fn scan_rate_hz(&mut self) -> Result<u32, String> {
        let payload = self.transact(CMD_SCAN_RATE, None)?;

        if payload.len() < 6 {
            return Err(format!("0x47 response too short: {} bytes", payload.len()));
        }

        let rate = u32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]);

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

    pub fn set_rgb_core(&mut self, enabled: bool) -> Result<bool, String> {
        let payload = self.transact(CMD_RGB_CORE, Some(if enabled { 1 } else { 0 }))?;

        if payload.len() < 3 {
            return Err("0x48 response too short".to_string());
        }

        Ok(payload[2] != 0)
    }

    fn creator_scene_command(&mut self, payload: &[u8; REPORT_BYTES]) -> Result<Vec<u8>, String> {
        let (response, _) = self.transact_raw32(payload, Duration::from_millis(800))?;

        let normalized: &[u8] = if response.len() >= LINUX_WRITE_BYTES && response[0] == 0 {
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
            return Err(format!("0x4A returned status 0x{:02X}", normalized[1]));
        }

        Ok(normalized.to_vec())
    }

    /// Read the firmware-authored live RGB shadow frame.
    ///
    /// This query is valid only while the AL80-specific overlay,
    /// Creator Scene, or low-battery safety frame is authoritative.
    pub fn live_rgb_telemetry(&mut self) -> Result<LiveRgbTelemetry, String> {
        let mut colors = vec![[0u8; 3]; LIVE_RGB_LED_COUNT];
        let mut first_meta: Option<(u8, u8, bool, bool, bool, bool)> = None;
        let mut coherent = true;
        let mut start = 0usize;

        while start < LIVE_RGB_LED_COUNT {
            let mut payload = [0u8; REPORT_BYTES];
            payload[0] = LIVE_RGB_TELEMETRY_COMMAND;
            payload[1] = start as u8;

            let (response, _) = self.transact_raw32(&payload, Duration::from_millis(500))?;

            let normalized: &[u8] = if response.len() >= LINUX_WRITE_BYTES && response[0] == 0 {
                &response[1..usize::min(LINUX_WRITE_BYTES, response.len())]
            } else {
                &response[0..usize::min(REPORT_BYTES, response.len())]
            };

            if normalized.len() < 7 {
                return Err("0x4D telemetry response too short".to_string());
            }

            if normalized[0] != LIVE_RGB_TELEMETRY_COMMAND {
                return Err(format!(
                    "unexpected live RGB telemetry command 0x{:02X}",
                    normalized[0]
                ));
            }

            if normalized[1] != STATUS_OK {
                return Err(format!(
                    "0x4D telemetry returned status 0x{:02X}",
                    normalized[1]
                ));
            }

            if normalized[2] != LIVE_RGB_TELEMETRY_VERSION {
                return Err(format!(
                    "unsupported live RGB telemetry version {}",
                    normalized[2]
                ));
            }

            if normalized[3] as usize != start {
                return Err(format!(
                    "0x4D telemetry start mismatch: expected {}, got {}",
                    start, normalized[3]
                ));
            }

            let count = normalized[4] as usize;

            if count == 0
                || count > LIVE_RGB_TELEMETRY_CHUNK
                || start + count > LIVE_RGB_LED_COUNT
                || normalized.len() < 7 + count * 3
            {
                return Err(format!(
                    "invalid 0x4D telemetry chunk start={} count={}",
                    start, count
                ));
            }

            let flags = normalized[5];
            let source = normalized[6];
            let meta = (
                normalized[2],
                source,
                (flags & 0x01) != 0,
                (flags & 0x02) != 0,
                (flags & 0x04) != 0,
                (flags & 0x10) != 0,
            );

            if let Some(first) = first_meta {
                if first != meta {
                    coherent = false;
                }
            } else {
                first_meta = Some(meta);
            }

            for index in 0..count {
                let src = 7 + index * 3;
                colors[start + index] = [normalized[src], normalized[src + 1], normalized[src + 2]];
            }

            start += count;
        }

        let (version, source, rgb, overlay, creator, valid) =
            first_meta.ok_or_else(|| "live RGB telemetry returned no metadata".to_string())?;

        Ok(LiveRgbTelemetry {
            version,
            source,
            frame_valid: valid && coherent,
            rgb_core_enabled: rgb,
            overlay_enabled: overlay,
            creator_scene_enabled: creator,
            colors,
        })
    }

    pub fn creator_scene_status(&mut self) -> Result<CreatorSceneStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_CREATOR_SCENE;
        payload[1] = 0;
        let response = self.creator_scene_command(&payload)?;

        if response.len() < 9 {
            return Err("0x4A query response too short".to_string());
        }
        if response[3] as usize != CREATOR_LED_COUNT {
            return Err(format!("0x4A reports unexpected LED count {}", response[3]));
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

    pub fn creator_scene_disable(&mut self) -> Result<CreatorSceneStatus, String> {
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
            let count = usize::min(CREATOR_CHUNK_MAX, CREATOR_LED_COUNT - start);
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
            if response.len() < 8 || response[6] != start as u8 || response[7] != count as u8 {
                return Err(format!("Creator Scene chunk ACK mismatch at LED {}", start));
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

    fn input_router_command(&mut self, payload: &[u8; REPORT_BYTES]) -> Result<Vec<u8>, String> {
        let (response, _) = self.transact_raw32(payload, Duration::from_millis(800))?;

        let normalized: &[u8] = if response.len() >= LINUX_WRITE_BYTES && response[0] == 0 {
            &response[1..usize::min(LINUX_WRITE_BYTES, response.len())]
        } else {
            &response[0..usize::min(REPORT_BYTES, response.len())]
        };

        if normalized.len() < 2 {
            return Err("0x4B response too short".to_string());
        }

        if normalized[0] != CMD_INPUT_ROUTER {
            return Err(format!(
                "unexpected Input Router response command 0x{:02X}",
                normalized[0]
            ));
        }

        if normalized[1] != STATUS_OK {
            return Err(format!("0x4B returned status 0x{:02X}", normalized[1]));
        }

        Ok(normalized.to_vec())
    }

    pub fn input_router_status(&mut self) -> Result<InputRouterStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 0;

        let response = self.input_router_command(&payload)?;

        if response.len() < 8 {
            return Err("0x4B query response too short".to_string());
        }

        if response[3] != INPUT_ROUTER_VERSION {
            return Err(format!("unsupported Input Router version {}", response[3]));
        }

        if response[4] as usize != INPUT_BINDING_MAX {
            return Err(format!(
                "unexpected Input Router slot count {}",
                response[4]
            ));
        }

        if response[5] != INPUT_ACTION_MAX {
            return Err(format!(
                "unexpected Input Router action max {}",
                response[5]
            ));
        }

        Ok(InputRouterStatus {
            enabled: response[2] != 0,
            version: response[3],
            binding_slots: response[4],
            max_action: response[5],
            fallback_supported: response[6] != 0,
        })
    }

    pub fn input_router_disable(&mut self) -> Result<InputRouterStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 1;
        self.input_router_command(&payload)?;
        self.input_router_status()
    }

    pub fn input_router_enable(&mut self) -> Result<InputRouterStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 2;
        self.input_router_command(&payload)?;
        self.input_router_status()
    }

    pub fn input_router_clear(&mut self) -> Result<(), String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 3;
        self.input_router_command(&payload)?;
        Ok(())
    }

    pub fn input_router_set_binding(
        &mut self,
        slot: u8,
        binding: InputBinding,
    ) -> Result<(), String> {
        if slot as usize >= INPUT_BINDING_MAX {
            return Err(format!("Input Router slot {slot} is out of range"));
        }

        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 4;
        payload[2] = slot;
        payload[3] = binding.event as u8;
        payload[4] = binding.trigger as u8;
        payload[5] = binding.trigger_a;
        payload[6] = binding.trigger_b;
        payload[7] = binding.action.id();
        payload[8] = 0;

        let response = self.input_router_command(&payload)?;

        if response.len() < 15
            || response[8] != slot
            || response[9] != binding.event as u8
            || response[10] != binding.trigger as u8
            || response[11] != binding.trigger_a
            || response[12] != binding.trigger_b
            || response[13] != binding.action.id()
            || response[14] != 0
        {
            return Err(format!("Input Router binding ACK mismatch at slot {slot}"));
        }

        Ok(())
    }

    pub fn input_router_get_binding(&mut self, slot: u8) -> Result<Option<InputBinding>, String> {
        if slot as usize >= INPUT_BINDING_MAX {
            return Err(format!("Input Router slot {slot} is out of range"));
        }

        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 5;
        payload[2] = slot;

        let response = self.input_router_command(&payload)?;

        if response.len() < 15 || response[8] != slot {
            return Err(format!("Input Router GET ACK mismatch at slot {slot}"));
        }

        if response[9] == 0 {
            return Ok(None);
        }

        if response[14] != 0 {
            return Err(format!(
                "Input Router slot {slot} has unsupported flags 0x{:02X}",
                response[14]
            ));
        }

        let event = InputEvent::try_from(response[9])?;
        let trigger = InputTrigger::try_from(response[10])?;
        let action = InputAction::from_id(response[13])?;

        Ok(Some(InputBinding::new(
            event,
            trigger,
            response[11],
            response[12],
            action,
        )?))
    }

    pub fn input_router_restore_defaults(&mut self) -> Result<InputRouterStatus, String> {
        let mut payload = [0u8; REPORT_BYTES];
        payload[0] = CMD_INPUT_ROUTER;
        payload[1] = 6;
        self.input_router_command(&payload)?;
        self.input_router_status()
    }

    pub fn input_router_apply_defaults(&mut self) -> Result<InputRouterStatus, String> {
        self.input_router_disable()?;
        self.input_router_restore_defaults()?;
        self.input_router_enable()
    }

    pub fn input_router_apply(
        &mut self,
        bindings: &[InputBinding],
    ) -> Result<InputRouterStatus, String> {
        if bindings.is_empty() {
            return Err("Input Router profile must contain at least one binding".to_string());
        }

        if bindings.len() > INPUT_BINDING_MAX {
            return Err(format!(
                "Input Router accepts at most {} bindings, got {}",
                INPUT_BINDING_MAX,
                bindings.len()
            ));
        }

        self.input_router_disable()?;

        let apply_result = (|| -> Result<InputRouterStatus, String> {
            self.input_router_clear()?;

            for (slot, binding) in bindings.iter().copied().enumerate() {
                self.input_router_set_binding(slot as u8, binding)?;
            }

            self.input_router_enable()
        })();

        match apply_result {
            Ok(status) => Ok(status),
            Err(error) => {
                let _ = self.input_router_disable();
                let _ = self.input_router_restore_defaults();

                Err(format!(
                    "Input Router apply failed; router left disabled with safe defaults restored: {error}"
                ))
            }
        }
    }

    pub fn overlay_status(&mut self) -> Result<OverlayStatus, String> {
        let payload = self.transact(CMD_OVERLAY, Some(2))?;

        if payload.len() < 4 {
            return Err("0x49 response too short".to_string());
        }

        Ok(OverlayStatus {
            enabled: payload[2] != 0,
            rgb_core_enabled: payload[3] != 0,
        })
    }

    pub fn set_overlay(&mut self, enabled: bool) -> Result<OverlayStatus, String> {
        let payload = self.transact(CMD_OVERLAY, Some(if enabled { 1 } else { 0 }))?;

        if payload.len() < 4 {
            return Err("0x49 response too short".to_string());
        }

        Ok(OverlayStatus {
            enabled: payload[2] != 0,
            rgb_core_enabled: payload[3] != 0,
        })
    }
}
