use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use chrono::{Local, TimeZone};
use clap::Parser;
use colored::Colorize;
use rustix::{
    fs::{self, Mode, OFlags, SeekFrom},
    io::Errno,
    time::{ClockId, clock_gettime},
};
use std::{
    io::{self, IsTerminal, Write},
    process::ExitCode,
};

const KMSG_PATH: &str = "/dev/kmsg";

const LEVEL_NAMES: [&str; 8] = [
    "emerg", "alert", "crit", "err", "warn", "notice", "info", "debug",
];

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "dmesg", about = "Print or control the kernel ring buffer")]
pub struct Args {
    /// Restrict output to the given comma-separated list of facilities
    #[arg(short = 'f', long, value_delimiter = ',')]
    facility: Vec<String>,

    /// Enable human-readable output (implies --color and --reltime)
    #[arg(short = 'H', long)]
    human: bool,

    /// Print only kernel messages
    #[arg(short = 'k', long)]
    kernel: bool,

    /// Restrict output to the given comma-separated list of levels
    #[arg(short = 'l', long, value_delimiter = ',')]
    level: Vec<String>,

    /// Colorize the output (auto, always, never)
    #[arg(short = 'L', long, num_args = 0..=1, default_missing_value = "auto")]
    color: Option<String>,

    /// Print raw message buffer (with priority prefix)
    #[arg(short = 'r', long)]
    raw: bool,

    /// Display human-readable timestamps
    #[arg(short = 'T', long)]
    ctime: bool,

    /// Do not print timestamps
    #[arg(short = 't', long)]
    notime: bool,

    /// Print only userspace messages
    #[arg(short = 'u', long)]
    userspace: bool,

    /// Wait for new messages
    #[arg(short = 'w', long)]
    follow: bool,

    /// Wait for and print only new messages
    #[arg(short = 'W', long = "follow-new")]
    follow_new: bool,

    /// Decode facility and priority to human-readable prefixes
    #[arg(short = 'x', long)]
    decode: bool,

    /// Show time delta between messages
    #[arg(short = 'd', long = "show-delta")]
    show_delta: bool,

    /// Display local time and delta in human-readable format
    #[arg(short = 'e', long)]
    reltime: bool,

    /// Time format: raw, ctime, reltime, delta, notime, iso
    #[arg(long = "time-format")]
    time_format: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum TimeFormat {
    Raw,
    Ctime,
    Iso,
    Notime,
    Delta,
    Reltime,
}

struct KmsgRecord {
    priority: u32,
    _sequence: u64,
    timestamp_us: u64,
    message: String,
}

impl KmsgRecord {
    fn level(&self) -> u8 {
        (self.priority & 0x7) as u8
    }
    fn facility(&self) -> u8 {
        (self.priority >> 3) as u8
    }
}

fn facility_name(code: u8) -> &'static str {
    match code {
        0 => "kern",
        1 => "user",
        2 => "mail",
        3 => "daemon",
        4 => "auth",
        5 => "syslog",
        6 => "lpr",
        7 => "news",
        8 => "uucp",
        9 => "cron",
        10 => "authpriv",
        11 => "ftp",
        16 => "local0",
        17 => "local1",
        18 => "local2",
        19 => "local3",
        20 => "local4",
        21 => "local5",
        22 => "local6",
        23 => "local7",
        _ => "unknown",
    }
}

fn parse_level_name(name: &str) -> Option<u8> {
    LEVEL_NAMES
        .iter()
        .position(|&n| n == name.to_ascii_lowercase())
        .map(|i| i as u8)
}

fn parse_facility_name(name: &str) -> Option<u8> {
    match name.to_ascii_lowercase().as_str() {
        "kern" => Some(0),
        "user" => Some(1),
        "mail" => Some(2),
        "daemon" => Some(3),
        "auth" => Some(4),
        "syslog" => Some(5),
        "lpr" => Some(6),
        "news" => Some(7),
        "uucp" => Some(8),
        "cron" => Some(9),
        "authpriv" => Some(10),
        "ftp" => Some(11),
        "local0" => Some(16),
        "local1" => Some(17),
        "local2" => Some(18),
        "local3" => Some(19),
        "local4" => Some(20),
        "local5" => Some(21),
        "local6" => Some(22),
        "local7" => Some(23),
        _ => None,
    }
}

fn parse_kmsg_record(data: &[u8]) -> Option<KmsgRecord> {
    let text = std::str::from_utf8(data).ok()?;
    let text = text.trim_end();
    let (header, message) = text.split_once(';')?;
    let mut parts = header.splitn(4, ',');

    let priority: u32 = parts.next()?.parse().ok()?;
    let sequence: u64 = parts.next()?.parse().ok()?;
    let timestamp_us: u64 = parts.next()?.parse().ok()?;
    let _flags = parts.next()?;

    // Take first line only (skip continuation and metadata lines)
    let message = message.lines().next().unwrap_or("").to_string();

    Some(KmsgRecord {
        priority,
        _sequence: sequence,
        timestamp_us,
        message,
    })
}

fn boot_time_offset_us() -> i64 {
    let rt = clock_gettime(ClockId::Realtime);
    let bt = clock_gettime(ClockId::Boottime);
    let rt_us = rt.tv_sec as i64 * 1_000_000 + rt.tv_nsec as i64 / 1000;
    let bt_us = bt.tv_sec as i64 * 1_000_000 + bt.tv_nsec as i64 / 1000;
    rt_us - bt_us
}

fn format_timestamp(
    timestamp_us: u64,
    format: TimeFormat,
    offset_us: i64,
    prev_timestamp_us: Option<u64>,
    use_color: bool,
) -> String {
    let colorize = |s: String| -> String {
        if use_color { s.green().to_string() } else { s }
    };

    match format {
        TimeFormat::Notime => String::new(),
        TimeFormat::Raw => {
            let secs = timestamp_us / 1_000_000;
            let usecs = timestamp_us % 1_000_000;
            colorize(format!("[{secs:>5}.{usecs:06}] "))
        }
        TimeFormat::Ctime => {
            let wall_us = offset_us + timestamp_us as i64;
            let secs = wall_us / 1_000_000;
            let nsecs = ((wall_us % 1_000_000) * 1000) as u32;
            let dt = Local.timestamp_opt(secs, nsecs).unwrap();
            colorize(format!("[{}] ", dt.format("%a %b %e %H:%M:%S %Y")))
        }
        TimeFormat::Iso => {
            let wall_us = offset_us + timestamp_us as i64;
            let secs = wall_us / 1_000_000;
            let nsecs = ((wall_us % 1_000_000) * 1000) as u32;
            let dt = Local.timestamp_opt(secs, nsecs).unwrap();
            colorize(format!("{} ", dt.format("%Y-%m-%dT%H:%M:%S,%6f%:z")))
        }
        TimeFormat::Delta => {
            let secs = timestamp_us / 1_000_000;
            let usecs = timestamp_us % 1_000_000;
            let delta = prev_timestamp_us
                .map(|prev| timestamp_us.saturating_sub(prev))
                .unwrap_or(0);
            let d_secs = delta / 1_000_000;
            let d_usecs = delta % 1_000_000;
            colorize(format!(
                "[{secs:>5}.{usecs:06} <{d_secs:>5}.{d_usecs:06}>] "
            ))
        }
        TimeFormat::Reltime => {
            match prev_timestamp_us
                .map(|prev| timestamp_us.saturating_sub(prev))
            {
                Some(d) if d < 60_000_000 => {
                    let d_secs = d / 1_000_000;
                    let d_usecs = d % 1_000_000;
                    colorize(format!("[  +{d_secs:>3}.{d_usecs:06}] "))
                }
                _ => {
                    let wall_us = offset_us + timestamp_us as i64;
                    let secs = wall_us / 1_000_000;
                    let nsecs = ((wall_us % 1_000_000) * 1000) as u32;
                    let dt = Local.timestamp_opt(secs, nsecs).unwrap();
                    let ts = format!("[{}] ", dt.format("%b%e %H:%M"));
                    if use_color { ts.cyan().to_string() } else { ts }
                }
            }
        }
    }
}

fn color_message(level: u8, msg: &str, use_color: bool) -> String {
    if !use_color {
        return msg.to_string();
    }
    if msg.contains("segfault") {
        return msg.red().bold().to_string();
    }
    match level {
        0..=2 => msg.red().bold().to_string(),
        3 => msg.red().to_string(),
        4 => msg.yellow().to_string(),
        _ => color_subsystem(msg),
    }
}

fn color_subsystem(msg: &str) -> String {
    if let Some(idx) = msg.find(": ") {
        let (subsys, rest) = msg.split_at(idx + 1);
        format!("{}{}", subsys.yellow(), rest)
    } else {
        msg.to_string()
    }
}

fn build_level_filter(args: &Args) -> Vec<u8> {
    let mut levels = Vec::new();
    for l in &args.level {
        if let Some(base) = l.strip_suffix('+') {
            if let Some(idx) = parse_level_name(base) {
                for i in 0..=idx {
                    if !levels.contains(&i) {
                        levels.push(i);
                    }
                }
            }
        } else if let Some(idx) = parse_level_name(l)
            && !levels.contains(&idx)
        {
            levels.push(idx);
        }
    }
    levels
}

fn build_facility_filter(args: &Args) -> Vec<u8> {
    let mut facilities = Vec::new();
    if args.kernel {
        facilities.push(0);
    }
    if args.userspace {
        for code in 1..=23u8 {
            if !facilities.contains(&code) {
                facilities.push(code);
            }
        }
    }
    for f in &args.facility {
        if let Some(code) = parse_facility_name(f)
            && !facilities.contains(&code)
        {
            facilities.push(code);
        }
    }
    facilities
}

fn matches_filters(
    record: &KmsgRecord,
    level_filter: &[u8],
    facility_filter: &[u8],
) -> bool {
    if !level_filter.is_empty() && !level_filter.contains(&record.level()) {
        return false;
    }
    if !facility_filter.is_empty()
        && !facility_filter.contains(&record.facility())
    {
        return false;
    }
    true
}

struct OutputOpts {
    time_format: TimeFormat,
    offset_us: i64,
    decode: bool,
    raw: bool,
    use_color: bool,
}

fn print_record(
    out: &mut impl Write,
    record: &KmsgRecord,
    opts: &OutputOpts,
    prev_timestamp_us: Option<u64>,
) {
    if opts.raw {
        let _ = writeln!(out, "<{}>{}", record.priority, record.message);
        return;
    }

    let mut line = String::new();

    if opts.decode {
        let fac = facility_name(record.facility());
        let lvl = LEVEL_NAMES[record.level() as usize];
        line.push_str(&format!("{fac:<6}:{lvl:<6}: "));
    }

    line.push_str(&format_timestamp(
        record.timestamp_us,
        opts.time_format,
        opts.offset_us,
        prev_timestamp_us,
        opts.use_color,
    ));

    line.push_str(&color_message(
        record.level(),
        &record.message,
        opts.use_color,
    ));

    let _ = writeln!(out, "{line}");
}

pub fn run(args: Args) -> ExitCode {
    let use_color = match args.color.as_deref() {
        Some("never") => false,
        Some("always") => true,
        _ if args.human => io::stdout().is_terminal(),
        Some(_) => io::stdout().is_terminal(),
        None => io::stdout().is_terminal(),
    };
    colored::control::set_override(use_color);

    let time_format = if args.notime {
        TimeFormat::Notime
    } else if let Some(ref fmt) = args.time_format {
        match fmt.as_str() {
            "raw" => TimeFormat::Raw,
            "ctime" => TimeFormat::Ctime,
            "iso" => TimeFormat::Iso,
            "notime" => TimeFormat::Notime,
            "delta" => TimeFormat::Delta,
            "reltime" => TimeFormat::Reltime,
            other => {
                eprintln!("dmesg: unknown time format: {other}");
                return ExitCode::FAILURE;
            }
        }
    } else if args.human || args.reltime {
        TimeFormat::Reltime
    } else if args.ctime {
        TimeFormat::Ctime
    } else if args.show_delta {
        TimeFormat::Delta
    } else {
        TimeFormat::Raw
    };

    let level_filter = build_level_filter(&args);
    let facility_filter = build_facility_filter(&args);

    let fd = match fs::open(
        KMSG_PATH,
        OFlags::RDONLY | OFlags::NONBLOCK,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(e) => {
            eprintln!(
                "dmesg: failed to open {KMSG_PATH}: {}",
                io::Error::from(e)
            );
            return ExitCode::FAILURE;
        }
    };

    if args.follow_new
        && let Err(e) = fs::seek(&fd, SeekFrom::End(0))
    {
        eprintln!("dmesg: seek error: {}", io::Error::from(e));
        return ExitCode::FAILURE;
    }

    let opts = OutputOpts {
        time_format,
        offset_us: boot_time_offset_us(),
        decode: args.decode,
        raw: args.raw,
        use_color,
    };
    let mut prev_timestamp_us: Option<u64> = None;
    let mut buf = [0u8; 8192];
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let following = args.follow || args.follow_new;

    loop {
        match rustix::io::read(&fd, &mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Some(record) = parse_kmsg_record(&buf[..n]) {
                    if matches_filters(&record, &level_filter, &facility_filter)
                    {
                        print_record(
                            &mut out,
                            &record,
                            &opts,
                            prev_timestamp_us,
                        );
                    }
                    prev_timestamp_us = Some(record.timestamp_us);
                }
            }
            Err(Errno::AGAIN) => break,
            Err(Errno::INTR) => continue,
            Err(e) => {
                eprintln!("dmesg: read error: {}", io::Error::from(e));
                return ExitCode::FAILURE;
            }
        }
    }

    if following {
        let flags = fs::fcntl_getfl(&fd).unwrap_or(OFlags::empty());
        if let Err(e) = fs::fcntl_setfl(&fd, flags.difference(OFlags::NONBLOCK))
        {
            eprintln!(
                "dmesg: failed to set blocking mode: {}",
                io::Error::from(e)
            );
            return ExitCode::FAILURE;
        }

        loop {
            match rustix::io::read(&fd, &mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if let Some(record) = parse_kmsg_record(&buf[..n]) {
                        if matches_filters(
                            &record,
                            &level_filter,
                            &facility_filter,
                        ) {
                            print_record(
                                &mut out,
                                &record,
                                &opts,
                                prev_timestamp_us,
                            );
                            let _ = out.flush();
                        }
                        prev_timestamp_us = Some(record.timestamp_us);
                    }
                }
                Err(Errno::INTR) => continue,
                Err(e) => {
                    eprintln!("dmesg: read error: {}", io::Error::from(e));
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    ExitCode::SUCCESS
}
