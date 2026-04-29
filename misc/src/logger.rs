use clap::Parser;
use std::{
    io::{self, BufRead},
    os::unix::net::UnixDatagram,
    process::ExitCode,
};

const LOG_SOCKET: &str = "/dev/log";

const FACILITIES: &[(&str, u8)] = &[
    ("kern", 0),
    ("user", 1),
    ("mail", 2),
    ("daemon", 3),
    ("auth", 4),
    ("syslog", 5),
    ("lpr", 6),
    ("news", 7),
    ("uucp", 8),
    ("cron", 9),
    ("authpriv", 10),
    ("ftp", 11),
    ("local0", 16),
    ("local1", 17),
    ("local2", 18),
    ("local3", 19),
    ("local4", 20),
    ("local5", 21),
    ("local6", 22),
    ("local7", 23),
    ("security", 4),
];

const LEVELS: &[(&str, u8)] = &[
    ("emerg", 0),
    ("alert", 1),
    ("crit", 2),
    ("err", 3),
    ("warning", 4),
    ("notice", 5),
    ("info", 6),
    ("debug", 7),
    ("panic", 0),
    ("error", 3),
    ("warn", 4),
];

#[derive(Parser)]
#[command(name = "logger", about = "Enter messages into the system log")]
pub struct Args {
    /// Log the PID of the logger process
    #[arg(short = 'i')]
    log_pid: bool,

    /// Log the contents of the specified file
    #[arg(short = 'f', long = "file")]
    file: Option<String>,

    /// Skip empty lines when processing files
    #[arg(short = 'e', long = "skip-empty")]
    skip_empty: bool,

    /// Set priority as facility.level or numeric (default: user.notice)
    #[arg(short = 'p', long = "priority", default_value = "user.notice")]
    priority: String,

    /// Also output message to stderr
    #[arg(short = 's', long = "stderr")]
    stderr: bool,

    /// Tag every line with this tag (default: username)
    #[arg(short = 't', long = "tag")]
    tag: Option<String>,

    /// Write to the specified Unix socket instead of /dev/log
    #[arg(short = 'u', long = "socket")]
    socket: Option<String>,

    /// Message to log
    message: Vec<String>,
}

fn parse_facility(name: &str) -> Result<u8, String> {
    for &(n, code) in FACILITIES {
        if n.eq_ignore_ascii_case(name) {
            return Ok(code);
        }
    }
    Err(format!("unknown facility: {name}"))
}

fn parse_level(name: &str) -> Result<u8, String> {
    for &(n, code) in LEVELS {
        if n.eq_ignore_ascii_case(name) {
            return Ok(code);
        }
    }
    Err(format!("unknown level: {name}"))
}

fn parse_priority(s: &str) -> Result<u32, String> {
    if let Ok(n) = s.parse::<u32>() {
        return Ok(n);
    }
    let (fac_name, level_name) = s
        .split_once('.')
        .ok_or_else(|| format!("invalid priority: {s}"))?;
    let fac = parse_facility(fac_name)? as u32;
    let level = parse_level(level_name)? as u32;
    Ok(fac * 8 + level)
}

fn format_message(
    priority: u32,
    tag: &str,
    pid: Option<u32>,
    message: &str,
) -> String {
    if let Some(pid) = pid {
        format!("<{priority}>{tag}[{pid}]: {message}")
    } else {
        format!("<{priority}>{tag}: {message}")
    }
}

fn default_tag() -> String {
    std::env::var("USER")
        .unwrap_or_else(|_| rustix::process::getuid().as_raw().to_string())
}

pub fn run(args: Args) -> ExitCode {
    let priority = match parse_priority(&args.priority) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("logger: {e}");
            return ExitCode::FAILURE;
        }
    };

    let tag = args.tag.unwrap_or_else(default_tag);
    let pid = if args.log_pid {
        Some(std::process::id())
    } else {
        None
    };

    let socket_path = args.socket.as_deref().unwrap_or(LOG_SOCKET);
    let socket = match UnixDatagram::unbound() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("logger: failed to create socket: {e}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = socket.connect(socket_path) {
        eprintln!("logger: failed to connect to {socket_path}: {e}");
        return ExitCode::FAILURE;
    }

    let mut failed = false;
    let mut send = |message: &str| {
        let formatted = format_message(priority, &tag, pid, message);
        if args.stderr {
            eprintln!("{tag}: {message}");
        }
        if let Err(e) = socket.send(formatted.as_bytes()) {
            eprintln!("logger: send failed: {e}");
            failed = true;
        }
    };

    if let Some(ref file) = args.file {
        let reader: Box<dyn BufRead> = if file == "-" {
            Box::new(io::stdin().lock())
        } else {
            match std::fs::File::open(file) {
                Ok(f) => Box::new(io::BufReader::new(f)),
                Err(e) => {
                    eprintln!("logger: failed to open {file}: {e}");
                    return ExitCode::FAILURE;
                }
            }
        };
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if args.skip_empty && line.is_empty() {
                        continue;
                    }
                    send(&line);
                }
                Err(e) => {
                    eprintln!("logger: read error: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    } else if !args.message.is_empty() {
        send(&args.message.join(" "));
    } else {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if args.skip_empty && line.is_empty() {
                        continue;
                    }
                    send(&line);
                }
                Err(e) => {
                    eprintln!("logger: read error: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
