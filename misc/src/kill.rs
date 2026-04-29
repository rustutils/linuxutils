use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::process::{self, Pid, Signal};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "kill",
    about = "Terminate a process",
    override_usage = "kill [-s <signal>|-<signal>] <pid>...\n       \
                      kill -l [<number>]\n       \
                      kill -L"
)]
pub struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

const SIGNALS: &[(i32, &str)] = &[
    (1, "HUP"),
    (2, "INT"),
    (3, "QUIT"),
    (4, "ILL"),
    (5, "TRAP"),
    (6, "ABRT"),
    (7, "BUS"),
    (8, "FPE"),
    (9, "KILL"),
    (10, "USR1"),
    (11, "SEGV"),
    (12, "USR2"),
    (13, "PIPE"),
    (14, "ALRM"),
    (15, "TERM"),
    (16, "STKFLT"),
    (17, "CHLD"),
    (18, "CONT"),
    (19, "STOP"),
    (20, "TSTP"),
    (21, "TTIN"),
    (22, "TTOU"),
    (23, "URG"),
    (24, "XCPU"),
    (25, "XFSZ"),
    (26, "VTALRM"),
    (27, "PROF"),
    (28, "WINCH"),
    (29, "IO"),
    (30, "PWR"),
    (31, "SYS"),
];

const SIGNAL_ALIASES: &[(&str, i32)] = &[("IOT", 6), ("CLD", 17), ("POLL", 29)];

const RTMIN: i32 = 34;
const RTMAX: i32 = 64;

fn signal_name(num: i32) -> Option<String> {
    for &(n, name) in SIGNALS {
        if n == num {
            return Some(name.to_string());
        }
    }
    if (RTMIN..=RTMAX).contains(&num) {
        return Some(rt_signal_name(num));
    }
    None
}

fn rt_signal_name(num: i32) -> String {
    if num == RTMIN {
        return "RTMIN".to_string();
    }
    if num == RTMAX {
        return "RTMAX".to_string();
    }
    let mid = (RTMIN + RTMAX) / 2;
    if num <= mid {
        format!("RTMIN+{}", num - RTMIN)
    } else {
        format!("RTMAX-{}", RTMAX - num)
    }
}

fn parse_signal_str(s: &str) -> Result<i32, String> {
    if let Ok(n) = s.parse::<i32>() {
        if n == 0 || (1..=31).contains(&n) || (RTMIN..=RTMAX).contains(&n) {
            return Ok(n);
        }
        return Err(format!("unknown signal: {s}"));
    }

    let name = s.strip_prefix("SIG").unwrap_or(s);
    let upper = name.to_ascii_uppercase();

    for &(num, sname) in SIGNALS {
        if sname == upper {
            return Ok(num);
        }
    }
    for &(alias, num) in SIGNAL_ALIASES {
        if alias == upper {
            return Ok(num);
        }
    }

    if upper == "RTMIN" {
        return Ok(RTMIN);
    }
    if upper == "RTMAX" {
        return Ok(RTMAX);
    }
    if let Some(offset) = upper.strip_prefix("RTMIN+")
        && let Ok(n) = offset.parse::<i32>()
    {
        let sig = RTMIN + n;
        if sig <= RTMAX {
            return Ok(sig);
        }
    }
    if let Some(offset) = upper.strip_prefix("RTMAX-")
        && let Ok(n) = offset.parse::<i32>()
    {
        let sig = RTMAX - n;
        if sig >= RTMIN {
            return Ok(sig);
        }
    }

    Err(format!("unknown signal: {s}"))
}

fn signal_to_rustix(num: i32) -> Option<Signal> {
    Some(match num {
        1 => Signal::HUP,
        2 => Signal::INT,
        3 => Signal::QUIT,
        4 => Signal::ILL,
        5 => Signal::TRAP,
        6 => Signal::ABORT,
        7 => Signal::BUS,
        8 => Signal::FPE,
        9 => Signal::KILL,
        10 => Signal::USR1,
        11 => Signal::SEGV,
        12 => Signal::USR2,
        13 => Signal::PIPE,
        14 => Signal::ALARM,
        15 => Signal::TERM,
        16 => Signal::STKFLT,
        17 => Signal::CHILD,
        18 => Signal::CONT,
        19 => Signal::STOP,
        20 => Signal::TSTP,
        21 => Signal::TTIN,
        22 => Signal::TTOU,
        23 => Signal::URG,
        24 => Signal::XCPU,
        25 => Signal::XFSZ,
        26 => Signal::VTALARM,
        27 => Signal::PROF,
        28 => Signal::WINCH,
        29 => Signal::IO,
        30 => Signal::POWER,
        31 => Signal::SYS,
        // SAFETY: we've bounds-checked n to be a valid RT signal number
        34..=64 => unsafe { Signal::from_raw_unchecked(num) },
        _ => return None,
    })
}

enum Action {
    Kill { signal: i32, pids: Vec<i32> },
    List(Option<String>),
    Table,
}

fn parse_action(args: &[String]) -> Result<Action, String> {
    let mut signal: Option<i32> = None;
    let mut pids: Vec<i32> = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "-l" || arg == "--list" {
            let list_arg = args.get(i + 1).cloned();
            return Ok(Action::List(list_arg));
        }

        if arg == "-L" || arg == "--table" {
            return Ok(Action::Table);
        }

        if arg == "-s" || arg == "--signal" {
            i += 1;
            let sig_str =
                args.get(i).ok_or("option '-s' requires an argument")?;
            signal = Some(parse_signal_str(sig_str)?);
            i += 1;
            continue;
        }

        if arg == "--" {
            i += 1;
            break;
        }

        if signal.is_none()
            && let Some(sig_str) = arg.strip_prefix('-')
            && !sig_str.is_empty()
            && let Ok(sig) = parse_signal_str(sig_str)
        {
            signal = Some(sig);
            i += 1;
            continue;
        }

        match arg.parse::<i32>() {
            Ok(pid) => pids.push(pid),
            Err(_) => return Err(format!("failed to parse pid: '{arg}'")),
        }
        i += 1;
    }

    while i < args.len() {
        match args[i].parse::<i32>() {
            Ok(pid) => pids.push(pid),
            Err(_) => {
                return Err(format!("failed to parse pid: '{}'", args[i]));
            }
        }
        i += 1;
    }

    if pids.is_empty() {
        return Err("no process ID specified".to_string());
    }

    Ok(Action::Kill {
        signal: signal.unwrap_or(15),
        pids,
    })
}

fn send_signal(pid: i32, sig: i32) -> Result<(), rustix::io::Errno> {
    if sig == 0 {
        if pid == 0 {
            return process::test_kill_current_process_group();
        }
        let p = Pid::from_raw(pid).ok_or(rustix::io::Errno::INVAL)?;
        return process::test_kill_process(p);
    }

    let signal = signal_to_rustix(sig).ok_or(rustix::io::Errno::INVAL)?;

    if pid == 0 {
        process::kill_current_process_group(signal)
    } else {
        let p = Pid::from_raw(pid).ok_or(rustix::io::Errno::INVAL)?;
        if pid < 0 {
            process::kill_process_group(p, signal)
        } else {
            process::kill_process(p, signal)
        }
    }
}

fn list_signals(arg: Option<String>) -> ExitCode {
    match arg {
        None => {
            let names: Vec<String> = SIGNALS
                .iter()
                .map(|(_, name)| name.to_string())
                .chain((RTMIN..=RTMAX).map(rt_signal_name))
                .collect();
            println!("{}", names.join(" "));
            ExitCode::SUCCESS
        }
        Some(s) if s.starts_with("0x") || s.starts_with("0X") => {
            match u64::from_str_radix(&s[2..], 16) {
                Ok(mask) => {
                    for bit in 1..=RTMAX {
                        if mask & (1u64 << (bit - 1)) != 0
                            && let Some(name) = signal_name(bit)
                        {
                            println!("{name}");
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(_) => {
                    eprintln!("kill: invalid signal mask: {s}");
                    ExitCode::FAILURE
                }
            }
        }
        Some(s) => match s.parse::<i32>() {
            Ok(mut num) => {
                if num > 128 {
                    num -= 128;
                }
                match signal_name(num) {
                    Some(name) => {
                        println!("{name}");
                        ExitCode::SUCCESS
                    }
                    None => {
                        eprintln!("kill: unknown signal: {num}");
                        ExitCode::FAILURE
                    }
                }
            }
            Err(_) => match parse_signal_str(&s) {
                Ok(num) => {
                    println!("{num}");
                    ExitCode::SUCCESS
                }
                Err(_) => {
                    eprintln!("kill: unknown signal: {s}");
                    ExitCode::FAILURE
                }
            },
        },
    }
}

fn table_signals() -> ExitCode {
    let entries: Vec<(i32, String)> = SIGNALS
        .iter()
        .map(|&(num, name)| (num, name.to_string()))
        .chain((RTMIN..=RTMAX).map(|n| (n, rt_signal_name(n))))
        .collect();

    let cols = 7;
    for (i, (num, name)) in entries.iter().enumerate() {
        if i > 0 && i % cols == 0 {
            println!();
        }
        print!("{num:2} {name:<10}");
    }
    println!();
    ExitCode::SUCCESS
}

pub fn run(args: Args) -> ExitCode {
    let action = match parse_action(&args.args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("kill: {e}");
            return ExitCode::FAILURE;
        }
    };

    match action {
        Action::List(arg) => list_signals(arg),
        Action::Table => table_signals(),
        Action::Kill { signal, pids } => {
            let mut failures = 0;
            let total = pids.len();
            for pid in pids {
                if let Err(e) = send_signal(pid, signal) {
                    eprintln!(
                        "kill: sending signal to {pid} failed: {}",
                        std::io::Error::from(e)
                    );
                    failures += 1;
                }
            }
            if failures == 0 {
                ExitCode::SUCCESS
            } else if failures < total {
                ExitCode::from(64)
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
