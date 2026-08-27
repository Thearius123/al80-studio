//! Read-only Windows HID enumeration diagnostic.
//!
//! This binary deliberately does not call `Al80::connect`, `DeviceInfo::open`,
//! `RawHidSession::new`, or any protocol write path.

#[cfg(windows)]
fn main() {
    match al80_core::DeviceInfo::enumerate_windows_read_only() {
        Ok(paths) => {
            println!("AL80_WINDOWS_HID_ENUMERATION=PASS");
            println!("AL80_WINDOWS_HID_MATCH_COUNT={}", paths.len());

            for (index, path) in paths.iter().enumerate() {
                println!(
                    "AL80_WINDOWS_HID_MATCH_{}={}",
                    index,
                    path.display()
                );
            }

            println!("AL80_WINDOWS_HID_OPEN=NO");
            println!("AL80_WINDOWS_HID_WRITE=NO");
        }

        Err(error) => {
            eprintln!("AL80_WINDOWS_HID_ENUMERATION=FAIL");
            eprintln!("ERROR={error}");
            std::process::exit(1);
        }
    }
}

#[cfg(not(windows))]
fn main() {
    println!("AL80_WINDOWS_HID_ENUMERATION=UNSUPPORTED_PLATFORM");
    println!("AL80_WINDOWS_HID_OPEN=NO");
    println!("AL80_WINDOWS_HID_WRITE=NO");
}
