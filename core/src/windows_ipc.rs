//! Windows-only local IPC transport for AL80 Studio.
//!
//! The Linux implementation remains the physically validated Unix-domain
//! socket path. This module provides the Windows equivalent using a local,
//! byte-mode Win32 Named Pipe.

#![cfg(windows)]

use std::env;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::iter;
use std::ptr::{null, null_mut};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HANDLE, ERROR_BROKEN_PIPE, ERROR_FILE_NOT_FOUND,
    ERROR_PIPE_BUSY, ERROR_PIPE_CONNECTED, GENERIC_READ, GENERIC_WRITE,
    INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, ReadFile, WriteFile, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
};
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, WaitNamedPipeW,
    PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const RETRY_SLEEP: Duration = Duration::from_millis(10);
const PIPE_BUFFER_BYTES: u32 = 4096;

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(iter::once(0)).collect()
}

fn safe_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            out.push(ch);
        } else {
            out.push('_');
        }
    }

    if out.is_empty() {
        "user".to_string()
    } else {
        out
    }
}

pub fn default_pipe_name() -> String {
    let user = env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    format!(r"\\.\pipe\al80d-{}", safe_component(&user))
}

#[derive(Debug)]
pub struct NamedPipeStream {
    handle: HANDLE,
    server_side: bool,
}

unsafe impl Send for NamedPipeStream {}

impl NamedPipeStream {
    fn from_handle(handle: HANDLE, server_side: bool) -> Self {
        Self {
            handle,
            server_side,
        }
    }

    pub fn connect_default() -> Result<Self, String> {
        Self::connect(&default_pipe_name())
    }

    pub fn connect(name: &str) -> Result<Self, String> {
        let name_wide = wide(name);
        let deadline = Instant::now() + CONNECT_TIMEOUT;

        loop {
            let handle = unsafe {
                CreateFileW(
                    name_wide.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    null(),
                    OPEN_EXISTING,
                    0,
                    null_mut(),
                )
            };

            if handle != INVALID_HANDLE_VALUE {
                return Ok(Self::from_handle(handle, false));
            }

            let error = unsafe { GetLastError() };

            if Instant::now() >= deadline {
                return Err(format!(
                    "timed out connecting to Windows AL80 Named Pipe {name}: {}",
                    io::Error::from_raw_os_error(error as i32)
                ));
            }

            if error == ERROR_PIPE_BUSY {
                unsafe {
                    WaitNamedPipeW(
                        name_wide.as_ptr(),
                        RETRY_SLEEP.as_millis() as u32,
                    );
                }
                continue;
            }

            if error == ERROR_FILE_NOT_FOUND {
                thread::sleep(RETRY_SLEEP);
                continue;
            }

            return Err(format!(
                "cannot connect to Windows AL80 Named Pipe {name}: {}",
                io::Error::from_raw_os_error(error as i32)
            ));
        }
    }
}

impl Read for NamedPipeStream {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let requested = buffer.len().min(u32::MAX as usize) as u32;
        let mut read = 0u32;

        let ok = unsafe {
            ReadFile(
                self.handle,
                buffer.as_mut_ptr(),
                requested,
                &mut read,
                null_mut(),
            )
        };

        if ok != 0 {
            return Ok(read as usize);
        }

        let error = unsafe { GetLastError() };

        if error == ERROR_BROKEN_PIPE {
            return Ok(0);
        }

        Err(io::Error::from_raw_os_error(error as i32))
    }
}

impl Write for NamedPipeStream {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let requested = buffer.len().min(u32::MAX as usize) as u32;
        let mut written = 0u32;

        let ok = unsafe {
            WriteFile(
                self.handle,
                buffer.as_ptr(),
                requested,
                &mut written,
                null_mut(),
            )
        };

        if ok != 0 {
            Ok(written as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.server_side {
            return Ok(());
        }

        let ok = unsafe { FlushFileBuffers(self.handle) };

        if ok != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

impl Drop for NamedPipeStream {
    fn drop(&mut self) {
        unsafe {
            if self.server_side {
                DisconnectNamedPipe(self.handle);
            }
            CloseHandle(self.handle);
        }
    }
}

#[derive(Clone, Debug)]
pub struct NamedPipeListener {
    name: String,
    name_wide: Vec<u16>,
}

impl NamedPipeListener {
    pub fn bind_default() -> Result<Self, String> {
        Self::bind(&default_pipe_name())
    }

    pub fn bind(name: &str) -> Result<Self, String> {
        if !name.starts_with(r"\\.\pipe\") {
            return Err(format!(
                "invalid Windows Named Pipe path {name}; expected \\\\.\\pipe\\..."
            ));
        }

        Ok(Self {
            name: name.to_string(),
            name_wide: wide(name),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn accept(&self) -> Result<NamedPipeStream, String> {
        let handle = unsafe {
            CreateNamedPipeW(
                self.name_wide.as_ptr(),
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE
                    | PIPE_READMODE_BYTE
                    | PIPE_WAIT
                    | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                PIPE_BUFFER_BYTES,
                PIPE_BUFFER_BYTES,
                CONNECT_TIMEOUT.as_millis() as u32,
                null(),
            )
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(format!(
                "cannot create Windows AL80 Named Pipe {}: {}",
                self.name,
                io::Error::last_os_error()
            ));
        }

        let connected = unsafe { ConnectNamedPipe(handle, null_mut()) };

        if connected == 0 {
            let error = unsafe { GetLastError() };

            if error != ERROR_PIPE_CONNECTED {
                unsafe {
                    CloseHandle(handle);
                }

                return Err(format!(
                    "cannot accept Windows AL80 Named Pipe {}: {}",
                    self.name,
                    io::Error::from_raw_os_error(error as i32)
                ));
            }
        }

        Ok(NamedPipeStream::from_handle(handle, true))
    }
}

pub fn request(command: &str) -> Result<String, String> {
    request_to(&default_pipe_name(), command)
}

pub fn request_to(name: &str, command: &str) -> Result<String, String> {
    let mut stream = NamedPipeStream::connect(name)?;

    writeln!(stream, "{command}")
        .map_err(|error| format!("cannot write Windows IPC request: {error}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();

    reader
        .read_line(&mut response)
        .map_err(|error| format!("cannot read Windows IPC response: {error}"))?;

    let response = response.trim().to_string();

    if response.is_empty() {
        return Err("al80d returned an empty Windows IPC response".to_string());
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn windows_named_pipe_round_trip() {
        let name = format!(
            r"\\.\pipe\al80d-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let listener = NamedPipeListener::bind(&name).unwrap();
        let (ready_tx, ready_rx) = mpsc::channel();

        let server = thread::spawn(move || {
            ready_tx.send(()).unwrap();

            let stream = listener.accept().unwrap();
            let mut reader = BufReader::new(stream);
            let mut request = String::new();

            reader.read_line(&mut request).unwrap();
            assert_eq!(request.trim(), "PING");

            let stream = reader.get_mut();
            writeln!(stream, "OK PONG").unwrap();
            stream.flush().unwrap();
        });

        ready_rx.recv().unwrap();

        let response = request_to(&name, "PING").unwrap();
        assert_eq!(response, "OK PONG");

        server.join().unwrap();
    }

    #[test]
    fn windows_named_pipe_delayed_reader_preserves_reply() {
        let name = format!(
            r"\\.\pipe\al80d-test-delayed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let listener = NamedPipeListener::bind(&name).unwrap();
        let (ready_tx, ready_rx) = mpsc::channel();

        let server = thread::spawn(move || {
            ready_tx.send(()).unwrap();

            let stream = listener.accept().unwrap();
            let mut reader = BufReader::new(stream);
            let mut request = String::new();

            reader.read_line(&mut request).unwrap();
            assert_eq!(request.trim(), "PING");

            let stream = reader.get_mut();
            writeln!(stream, "OK PONG").unwrap();
            stream.flush().unwrap();
        });

        ready_rx.recv().unwrap();

        let mut client = NamedPipeStream::connect(&name).unwrap();
        writeln!(client, "PING").unwrap();

        thread::sleep(Duration::from_millis(150));

        let mut reader = BufReader::new(client);
        let mut response = String::new();
        reader.read_line(&mut response).unwrap();

        assert_eq!(response.trim(), "OK PONG");
        server.join().unwrap();
    }
}
