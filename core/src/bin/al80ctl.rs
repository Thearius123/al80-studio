use al80_core::lcd_feedback::LcdFeedback;
use std::env;
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

#[cfg(unix)]
fn socket_path() -> PathBuf {
    if let Ok(runtime) = env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("al80d.sock");
    }

    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    PathBuf::from(format!("/tmp/al80d-{user}.sock"))
}

#[cfg(unix)]
fn request(command: &str) -> Result<String, String> {
    let path = socket_path();

    let mut stream = UnixStream::connect(&path)
        .map_err(|error| format!("cannot connect to al80d at {}: {error}", path.display()))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| format!("cannot set read timeout: {error}"))?;

    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|error| format!("cannot set write timeout: {error}"))?;

    writeln!(stream, "{command}").map_err(|error| format!("cannot write request: {error}"))?;

    stream
        .flush()
        .map_err(|error| format!("cannot flush request: {error}"))?;

    let mut response = String::new();
    BufReader::new(stream)
        .read_line(&mut response)
        .map_err(|error| format!("cannot read response: {error}"))?;

    let response = response.trim().to_string();

    if response.is_empty() {
        return Err("al80d returned an empty response".to_string());
    }

    if let Some(error) = response.strip_prefix("ERR ") {
        return Err(error.to_string());
    }

    Ok(response)
}

#[cfg(windows)]
fn request(command: &str) -> Result<String, String> {
    Err(
        "AL80 Windows IPC transport is pending Windows Foundation Named Pipe stage".to_string(),
    )
}

fn help() {
    println!("AL80 Studio Control CLI");
    println!();
    println!("Usage:");
    println!("  al80ctl ping");
    println!("  al80ctl status");
    println!("  al80ctl capabilities");
    println!("  al80ctl telemetry rgb");
    println!("  al80ctl lcd status");
    println!("  al80ctl audio");
    println!("  al80ctl rgb on|off");
    println!("  al80ctl overlay status|on|off");
    println!("  al80ctl scene status");
    println!("  al80ctl scene off");
    println!("  al80ctl scene solid <RRGGBB>");
    println!("  al80ctl input status");
    println!("  al80ctl input dump");
    println!("  al80ctl input off");
    println!("  al80ctl input defaults");
    println!("  al80ctl input apply <event,trigger,a,b,action;...>");
    println!("  al80ctl lcd home");
    println!("  al80ctl lcd volume <0-100>");
    println!("  al80ctl lcd mute <0-100>");
    println!("  al80ctl lcd feedback <kind> <value>");
}

fn percent(raw: &str) -> Result<u8, String> {
    let value = raw
        .parse::<u8>()
        .map_err(|_| format!("invalid percent: {raw}"))?;

    if value > 100 {
        return Err("percent must be between 0 and 100".to_string());
    }

    Ok(value)
}

fn build_command(args: &[String]) -> Result<String, String> {
    match args {
        [cmd] if cmd == "ping" => Ok("PING".to_string()),
        [cmd] if cmd == "status" => Ok("STATUS".to_string()),
        [cmd] if cmd == "capabilities" => Ok("CAPABILITIES".to_string()),
        [cmd] if cmd == "audio" => Ok("AUDIO CURRENT".to_string()),

        [group, state] if group == "rgb" && (state == "on" || state == "off") => {
            Ok(format!("RGB {}", state.to_ascii_uppercase()))
        }

        [group, state]
            if group == "overlay" && matches!(state.as_str(), "status" | "on" | "off") =>
        {
            Ok(format!("OVERLAY {}", state.to_ascii_uppercase()))
        }

        [group, state] if group == "input" && state == "status" => Ok("INPUT STATUS".to_string()),

        [group, state] if group == "input" && state == "dump" => Ok("INPUT DUMP".to_string()),

        [group, state] if group == "input" && state == "off" => Ok("INPUT OFF".to_string()),

        [group, state] if group == "input" && state == "defaults" => {
            Ok("INPUT DEFAULTS".to_string())
        }

        [group, mode, raw] if group == "input" && mode == "apply" => {
            if raw.trim().is_empty() {
                return Err("input apply requires a binding spec".to_string());
            }
            Ok(format!("INPUT APPLY {raw}"))
        }

        [group, state] if group == "telemetry" && state == "rgb" => Ok("TELEMETRY RGB".to_string()),
        [group, state] if group == "lcd" && state == "status" => Ok("LCD STATUS".to_string()),
        [group, state] if group == "scene" && state == "status" => Ok("SCENE STATUS".to_string()),

        [group, state] if group == "scene" && state == "off" => Ok("SCENE OFF".to_string()),

        [group, mode, raw] if group == "scene" && mode == "solid" => {
            let color = raw.trim_start_matches('#').to_ascii_lowercase();
            if color.len() != 6 || !color.bytes().all(|value| value.is_ascii_hexdigit()) {
                return Err("scene solid expects RRGGBB".to_string());
            }
            Ok(format!("SCENE APPLY {}", color.repeat(82)))
        }

        [group, mode, kind, value] if group == "lcd" && mode == "feedback" => {
            let feedback = LcdFeedback::parse(kind, value)?;

            Ok(format!(
                "LCD FEEDBACK {} {}",
                feedback.kind_token(),
                feedback.value_token(),
            ))
        }

        [group, state] if group == "lcd" && state == "home" => Ok("LCD HOME".to_string()),

        [group, mode, raw] if group == "lcd" && (mode == "volume" || mode == "mute") => {
            let value = percent(raw)?;

            Ok(format!("LCD {} {}", mode.to_ascii_uppercase(), value))
        }

        _ => Err("invalid command; run al80ctl help".to_string()),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty()
        || args == ["help".to_string()]
        || args == ["--help".to_string()]
        || args == ["-h".to_string()]
    {
        help();
        return;
    }

    let command = match build_command(&args) {
        Ok(command) => command,
        Err(error) => {
            eprintln!("ERROR={error}");
            std::process::exit(2);
        }
    };

    match request(&command) {
        Ok(response) => {
            println!("{response}");
        }

        Err(error) => {
            eprintln!("ERROR={error}");
            std::process::exit(1);
        }
    }
}
