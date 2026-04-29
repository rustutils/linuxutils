use chrono::Local;
use clap::Parser;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    process::ExitCode,
};

const UTMP_PATH: &str = "/run/utmp";
const UTMP_RECORD_SIZE: usize = 384;
const UT_USER_PROCESS: i16 = 7;
const LINE_WIDTH: usize = 79;

#[derive(Parser)]
#[command(name = "wall", about = "Write a message to all users")]
pub struct Args {
    /// Suppress the banner (root only)
    #[arg(short = 'n', long)]
    nobanner: bool,

    /// Write timeout in seconds
    #[arg(short = 't', long, default_value = "300")]
    timeout: u32,

    /// Limit message to members of a group
    #[arg(short = 'g', long)]
    group: Option<String>,

    /// Message or file to send
    message: Vec<String>,
}

struct UtmpEntry {
    line: String,
}

fn extract_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

fn read_utmp() -> io::Result<Vec<UtmpEntry>> {
    let data = fs::read(UTMP_PATH)?;
    let mut entries = Vec::new();

    for chunk in data.chunks_exact(UTMP_RECORD_SIZE) {
        let ut_type = i16::from_ne_bytes([chunk[0], chunk[1]]);
        if ut_type != UT_USER_PROCESS {
            continue;
        }

        let line = extract_string(&chunk[8..40]);

        if line.starts_with(':') {
            continue;
        }

        entries.push(UtmpEntry { line });
    }

    Ok(entries)
}

fn hostname() -> String {
    let mut buf = [0u8; 256];
    if unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } == 0 {
        extract_string(&buf)
    } else {
        "localhost".to_string()
    }
}

fn current_user() -> String {
    std::env::var("USER")
        .unwrap_or_else(|_| unsafe { libc::getuid() }.to_string())
}

fn current_tty() -> String {
    fs::read_link("/proc/self/fd/2")
        .ok()
        .and_then(|p| {
            p.to_str()
                .map(|s| s.strip_prefix("/dev/").unwrap_or(s).to_string())
        })
        .unwrap_or_default()
}

fn read_message(args: &Args) -> io::Result<String> {
    if !args.message.is_empty() {
        let path = &args.message[0];
        if std::path::Path::new(path).is_file() {
            return fs::read_to_string(path);
        }
        return Ok(args.message.join(" "));
    }

    let mut buf = String::new();
    io::stdin().lock().read_to_string(&mut buf)?;
    Ok(buf)
}

fn format_output(banner: Option<&str>, message: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"\r\n");

    if let Some(banner) = banner {
        out.extend_from_slice(banner.as_bytes());
        out.extend_from_slice(b"\r\n");
    }

    for line in message.lines() {
        let mut remaining = line;
        loop {
            if remaining.len() <= LINE_WIDTH {
                out.extend_from_slice(remaining.as_bytes());
                out.extend_from_slice(b"\r\n");
                break;
            }
            out.extend_from_slice(&remaining.as_bytes()[..LINE_WIDTH]);
            out.extend_from_slice(b"\r\n");
            remaining = &remaining[LINE_WIDTH..];
        }
    }

    out.extend_from_slice(b"\r\n");
    out
}

pub fn run(args: Args) -> ExitCode {
    let message = match read_message(&args) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("wall: failed to read message: {e}");
            return ExitCode::FAILURE;
        }
    };

    let banner = if args.nobanner {
        if unsafe { libc::getuid() } != 0 {
            eprintln!("wall: --nobanner is available only for root");
            return ExitCode::FAILURE;
        }
        None
    } else {
        let user = current_user();
        let host = hostname();
        let tty = current_tty();
        let now = Local::now().format("%a %b %e %H:%M:%S %Y");
        Some(format!(
            "Broadcast message from {user}@{host} ({tty}) ({now}):"
        ))
    };

    let output = format_output(banner.as_deref(), &message);

    let entries = match read_utmp() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("wall: failed to read utmp: {e}");
            return ExitCode::FAILURE;
        }
    };

    for entry in &entries {
        let path = format!("/dev/{}", entry.line);
        if let Ok(mut f) = File::options().write(true).open(&path) {
            let _ = f.write_all(&output);
        }
    }

    ExitCode::SUCCESS
}
