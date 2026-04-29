use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use chrono::Local;
use clap::Parser;
use std::{
    fs::{self, File, Metadata},
    io::{self, BufRead, Write},
    os::unix::fs::MetadataExt,
    process::ExitCode,
    time::SystemTime,
};

const UTMP_PATH: &str = "/run/utmp";
const UTMP_RECORD_SIZE: usize = 384;
const UT_USER_PROCESS: i16 = 7;

#[derive(Parser)]
#[command(name = "write", about = "Send a message to another user")]
pub struct Args {
    /// User to write to
    user: String,

    /// Terminal to write to (e.g. pts/1)
    ttyname: Option<String>,
}

struct UtmpEntry {
    user: String,
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
        let user = extract_string(&chunk[44..76]);

        if line.starts_with(':') {
            continue;
        }

        entries.push(UtmpEntry { user, line });
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

fn idle_time(meta: &Metadata) -> u64 {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(meta.atime() as u64)
}

fn is_writable(path: &str) -> bool {
    File::options().write(true).open(path).is_ok()
}

pub fn run(args: Args) -> ExitCode {
    let entries = match read_utmp() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("write: failed to read utmp: {e}");
            return ExitCode::FAILURE;
        }
    };

    let user_entries: Vec<&UtmpEntry> =
        entries.iter().filter(|e| e.user == args.user).collect();

    if user_entries.is_empty() {
        eprintln!("write: {} is not logged in", args.user);
        return ExitCode::FAILURE;
    }

    let target = if let Some(ref tty) = args.ttyname {
        match user_entries.iter().find(|e| e.line == *tty) {
            Some(e) => e,
            None => {
                eprintln!("write: {} is not logged in on {tty}", args.user);
                return ExitCode::FAILURE;
            }
        }
    } else if user_entries.len() == 1 {
        user_entries[0]
    } else {
        // Pick terminal with shortest idle time
        let best = user_entries
            .iter()
            .min_by_key(|e| {
                let path = format!("/dev/{}", e.line);
                fs::metadata(&path)
                    .map(|m| idle_time(&m))
                    .unwrap_or(u64::MAX)
            })
            .unwrap();
        eprintln!(
            "write: {} is logged in more than once; writing to {}",
            args.user, best.line
        );
        best
    };

    let tty_path = format!("/dev/{}", target.line);

    if !is_writable(&tty_path) {
        eprintln!("write: {} has messages disabled", args.user);
        return ExitCode::FAILURE;
    }

    let mut tty = match File::options().write(true).open(&tty_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("write: failed to open {tty_path}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let from_user = current_user();
    let host = hostname();
    let from_tty = current_tty();
    let now = Local::now();
    let banner = format!(
        "Message from {from_user}@{host} on {from_tty} at {:02}:{:02} ...",
        now.format("%H"),
        now.format("%M")
    );
    let _ = writeln!(tty, "\r\n{banner}\r");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                let _ = writeln!(tty, "{line}\r");
            }
            Err(_) => break,
        }
    }

    let _ = writeln!(tty, "EOF\r");

    ExitCode::SUCCESS
}
