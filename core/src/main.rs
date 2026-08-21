use std::env;
use std::time::Instant;

use al80_core::Al80;

const VERSION: &str = "0.2.0-alpha";

fn yes_no(value: bool) -> &'static str {
    if value {
        "YES"
    } else {
        "NO"
    }
}

fn print_help() {
    println!("AL80 Studio Core {VERSION}");
    println!();
    println!("Usage:");
    println!("  al80-core status");
    println!("  al80-core scan");
    println!("  al80-core rgb status");
    println!("  al80-core rgb on");
    println!("  al80-core rgb off");
    println!("  al80-core overlay status");
    println!("  al80-core overlay on");
    println!("  al80-core overlay off");
    println!("  al80-core help");
    println!("  al80-core version");
    println!();
    println!("Safety:");
    println!("  status/scan/query operations are read-only.");
    println!("  rgb/overlay on/off are volatile runtime operations.");
    println!(
        "  no EEPROM, LCD media write, bootloader or firmware flashing is implemented."
    );
}

fn connect() -> Result<Al80, String> {
    let device = Al80::connect()?;

    println!("AL80_CONNECTED=YES");
    println!(
        "RAW_HID_DEVICE={}",
        device.device_info().devnode.display()
    );

    Ok(device)
}

fn command_status() -> Result<(), String> {
    let mut device = connect()?;
    let started = Instant::now();

    let scan = device.scan_rate_hz()?;
    let rgb = device.rgb_core_enabled()?;
    let overlay = device.overlay_status()?;

    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    println!("MATRIX_SCAN_HZ={scan}");
    println!(
        "MATRIX_SCAN_INTERVAL_US={:.1}",
        1_000_000.0 / scan as f64
    );
    println!("RGB_CORE_ENABLED={}", yes_no(rgb));
    println!(
        "CUSTOM_OVERLAY_ENABLED={}",
        yes_no(overlay.enabled)
    );
    println!(
        "OVERLAY_REPORTS_RGB_CORE={}",
        yes_no(overlay.rgb_core_enabled)
    );
    println!("STATUS_QUERY_TOTAL_MS={elapsed_ms:.3}");

    if scan >= 1000 {
        println!("MATRIX_ABOVE_USB_1KHZ_GATE=PASS");
    } else {
        println!("MATRIX_ABOVE_USB_1KHZ_GATE=FAIL");
    }

    println!("DEVICE_OPERATION=QUERY_ONLY");
    println!("PERSISTENT_WRITE=NO");
    println!("AL80_CORE_STATUS_GATE=PASS");

    Ok(())
}

fn command_scan() -> Result<(), String> {
    let mut device = connect()?;
    let started = Instant::now();

    let scan = device.scan_rate_hz()?;
    let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;

    println!("COMMAND=0x47");
    println!("COMMAND_CLASS=READ_ONLY");
    println!("MATRIX_SCAN_HZ={scan}");
    println!(
        "MATRIX_SCAN_INTERVAL_US={:.1}",
        1_000_000.0 / scan as f64
    );
    println!("RAW_HID_ROUND_TRIP_MS={elapsed_ms:.3}");
    println!("PERSISTENT_WRITE=NO");
    println!("AL80_CORE_SCAN_GATE=PASS");

    Ok(())
}

fn command_rgb(action: &str) -> Result<(), String> {
    let mut device = connect()?;

    match action {
        "status" => {
            let enabled = device.rgb_core_enabled()?;

            println!("COMMAND=0x48");
            println!("ARGUMENT=QUERY");
            println!("COMMAND_CLASS=READ_ONLY");
            println!("RGB_CORE_ENABLED={}", yes_no(enabled));
            println!("PERSISTENT_WRITE=NO");
        }

        "on" | "off" => {
            let wanted = action == "on";
            let actual = device.set_rgb_core(wanted)?;

            println!("COMMAND=0x48");
            println!(
                "ARGUMENT={}",
                if wanted { "ON" } else { "OFF" }
            );
            println!("COMMAND_CLASS=VOLATILE_RUNTIME");
            println!("RGB_CORE_ENABLED={}", yes_no(actual));
            println!("EEPROM_WRITE=NO");
            println!("PERSISTENT_WRITE=NO");

            if actual != wanted {
                return Err(
                    "RGB state verification failed".to_string()
                );
            }
        }

        _ => {
            return Err(
                "usage: al80-core rgb status|on|off".to_string()
            );
        }
    }

    println!("AL80_CORE_RGB_GATE=PASS");

    Ok(())
}

fn command_overlay(action: &str) -> Result<(), String> {
    let mut device = connect()?;

    match action {
        "status" => {
            let state = device.overlay_status()?;

            println!("COMMAND=0x49");
            println!("ARGUMENT=QUERY");
            println!("COMMAND_CLASS=READ_ONLY");
            println!(
                "CUSTOM_OVERLAY_ENABLED={}",
                yes_no(state.enabled)
            );
            println!(
                "RGB_CORE_ENABLED={}",
                yes_no(state.rgb_core_enabled)
            );
            println!("PERSISTENT_WRITE=NO");
        }

        "on" | "off" => {
            let wanted = action == "on";
            let state = device.set_overlay(wanted)?;

            println!("COMMAND=0x49");
            println!(
                "ARGUMENT={}",
                if wanted { "ON" } else { "OFF" }
            );
            println!("COMMAND_CLASS=VOLATILE_RUNTIME");
            println!(
                "CUSTOM_OVERLAY_ENABLED={}",
                yes_no(state.enabled)
            );
            println!(
                "RGB_CORE_ENABLED={}",
                yes_no(state.rgb_core_enabled)
            );
            println!("EEPROM_WRITE=NO");
            println!("PERSISTENT_WRITE=NO");

            if state.enabled != wanted {
                return Err(
                    "overlay state verification failed".to_string()
                );
            }
        }

        _ => {
            return Err(
                "usage: al80-core overlay status|on|off".to_string()
            );
        }
    }

    println!("AL80_CORE_OVERLAY_GATE=PASS");

    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        return command_status();
    }

    match args[0].as_str() {
        "status" => {
            if args.len() != 1 {
                return Err(
                    "usage: al80-core status".to_string()
                );
            }

            command_status()
        }

        "scan" => {
            if args.len() != 1 {
                return Err(
                    "usage: al80-core scan".to_string()
                );
            }

            command_scan()
        }

        "rgb" => {
            if args.len() != 2 {
                return Err(
                    "usage: al80-core rgb status|on|off".to_string()
                );
            }

            command_rgb(&args[1])
        }

        "overlay" => {
            if args.len() != 2 {
                return Err(
                    "usage: al80-core overlay status|on|off".to_string()
                );
            }

            command_overlay(&args[1])
        }

        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }

        "version" | "--version" | "-V" => {
            println!("al80-core {VERSION}");
            Ok(())
        }

        other => Err(format!("unknown command: {other}")),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AL80_CORE_ERROR={error}");
        eprintln!("AL80_CORE_COMMAND_GATE=FAIL");
        std::process::exit(1);
    }
}
