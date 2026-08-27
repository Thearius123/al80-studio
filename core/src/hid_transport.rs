//! Narrow OS transport boundary for the AL80 Raw HID session.
//!
//! Linux deliberately keeps the physically validated hidraw `File` backend.
//! Windows uses `hidapi`'s native hid.dll backend. Higher protocol layers
//! continue to see the same `Read + Write` byte stream and keep one I/O owner.

use std::io::{self, Read, Write};

pub const PROTOCOL_REPORT_BYTES: usize = 32;
pub const REPORT_WITH_ID_BYTES: usize = PROTOCOL_REPORT_BYTES + 1;
pub const REPORT_ID: u8 = 0;

#[cfg(unix)]
use std::fs::File;

#[cfg(windows)]
use hidapi::HidDevice;

#[derive(Debug)]
pub struct RawHidTransport {
    #[cfg(unix)]
    linux: File,

    #[cfg(windows)]
    windows: HidDevice,
}

impl RawHidTransport {
    #[cfg(unix)]
    pub fn from_linux_file(file: File) -> Self {
        Self { linux: file }
    }

    #[cfg(windows)]
    pub fn from_windows_device(device: HidDevice) -> Self {
        Self { windows: device }
    }
}

#[cfg(unix)]
impl Read for RawHidTransport {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.linux.read(buffer)
    }
}

#[cfg(unix)]
impl Write for RawHidTransport {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.linux.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.linux.flush()
    }
}

#[cfg(windows)]
fn normalize_windows_input(
    payload: &[u8],
    output: &mut [u8],
) -> io::Result<usize> {
    if payload.len() != PROTOCOL_REPORT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Windows AL80 HID returned {} payload bytes; expected {}",
                payload.len(),
                PROTOCOL_REPORT_BYTES
            ),
        ));
    }

    if output.len() < REPORT_WITH_ID_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Raw HID read buffer has {} bytes; need {}",
                output.len(),
                REPORT_WITH_ID_BYTES
            ),
        ));
    }

    output[0] = REPORT_ID;
    output[1..REPORT_WITH_ID_BYTES].copy_from_slice(payload);

    Ok(REPORT_WITH_ID_BYTES)
}

#[cfg(windows)]
impl Read for RawHidTransport {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut payload = [0u8; PROTOCOL_REPORT_BYTES];

        let count = self
            .windows
            .read_timeout(&mut payload, 5)
            .map_err(|error| {
                io::Error::other(format!("Windows AL80 HID read failed: {error}"))
            })?;

        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Windows AL80 HID read timeout",
            ));
        }

        normalize_windows_input(&payload[..count], output)
    }
}

#[cfg(windows)]
impl Write for RawHidTransport {
    fn write(&mut self, report: &[u8]) -> io::Result<usize> {
        if report.len() != REPORT_WITH_ID_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Windows AL80 HID write has {} bytes; expected {}",
                    report.len(),
                    REPORT_WITH_ID_BYTES
                ),
            ));
        }

        if report[0] != REPORT_ID {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Windows AL80 HID write report ID is {}; expected {}",
                    report[0],
                    REPORT_ID
                ),
            ));
        }

        let written = self.windows.write(report).map_err(|error| {
            io::Error::other(format!("Windows AL80 HID write failed: {error}"))
        })?;

        if written != report.len() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                format!(
                    "Windows AL80 HID short write: {written}/{} bytes",
                    report.len()
                ),
            ));
        }

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::*;

    #[test]
    fn windows_hid_input_normalizes_report_id_zero() {
        let payload = [0xA5u8; PROTOCOL_REPORT_BYTES];
        let mut report = [0u8; REPORT_WITH_ID_BYTES];

        let count = normalize_windows_input(&payload, &mut report).unwrap();

        assert_eq!(count, REPORT_WITH_ID_BYTES);
        assert_eq!(report[0], REPORT_ID);
        assert_eq!(&report[1..], &payload);
    }

    #[test]
    fn windows_hid_input_rejects_wrong_payload_size() {
        let payload = [0u8; PROTOCOL_REPORT_BYTES - 1];
        let mut report = [0u8; REPORT_WITH_ID_BYTES];

        assert!(normalize_windows_input(&payload, &mut report).is_err());
    }
}
