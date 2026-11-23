use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    fs::fstat,
    process::{Pid, PidfdFlags, pidfd_open},
};
use std::{process::ExitCode, time::Instant};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "waitpid", about = "Wait for arbitrary processes to exit")]
pub struct Args {
    /// Be more verbose
    #[arg(short, long)]
    verbose: bool,

    /// Maximum number of seconds to wait (fractional values allowed)
    #[arg(short, long)]
    timeout: Option<f64>,

    /// Don't error on already-exited PIDs
    #[arg(short, long)]
    exited: bool,

    /// Number of process exits to wait for
    #[arg(short, long)]
    count: Option<usize>,

    /// PIDs to wait for. Each can be a plain PID, or the race-free
    /// PID:INODE form printed by getino -p.
    #[arg(required = true)]
    pids: Vec<String>,
}

/// `(raw_pid, optional pidfs inode)` parsed from a single CLI arg.
struct PidSpec {
    pid: i32,
    inode: Option<u64>,
    raw: String,
}

fn parse_pid(arg: &str) -> Result<PidSpec, String> {
    let (pid_str, inode) = match arg.split_once(':') {
        Some((p, i)) => {
            let inode = i
                .parse::<u64>()
                .map_err(|_| format!("invalid PID:INODE inode: {arg}"))?;
            (p, Some(inode))
        }
        None => (arg, None),
    };
    let pid = pid_str
        .parse::<i32>()
        .map_err(|_| format!("invalid PID: {arg}"))?;
    if pid <= 0 {
        return Err(format!("invalid PID: {arg}"));
    }
    Ok(PidSpec {
        pid,
        inode,
        raw: arg.to_string(),
    })
}

pub fn run(args: Args) -> ExitCode {
    let specs: Vec<PidSpec> = match args
        .pids
        .iter()
        .map(|s| parse_pid(s))
        .collect::<Result<_, _>>()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("waitpid: {e}");
            return ExitCode::FAILURE;
        }
    };

    let target_count = args.count.unwrap_or(specs.len());
    let deadline = args.timeout.map(|t| {
        let dur = std::time::Duration::from_secs_f64(t.max(0.0));
        Instant::now() + dur
    });

    let mut pidfds = Vec::new();
    for spec in &specs {
        let pid = match Pid::from_raw(spec.pid) {
            Some(p) => p,
            None => {
                eprintln!("waitpid: invalid PID: {}", spec.raw);
                return ExitCode::FAILURE;
            }
        };

        match pidfd_open(pid, PidfdFlags::NONBLOCK) {
            Ok(fd) => {
                // Race-free check: if the caller passed PID:INODE,
                // verify the pidfs inode hasn't been reused.
                if let Some(expected) = spec.inode {
                    match fstat(&fd) {
                        Ok(st) if st.st_ino != expected => {
                            if args.exited {
                                eprintln!(
                                    "waitpid: PID {} has exited, skipping",
                                    spec.pid
                                );
                                continue;
                            }
                            eprintln!("waitpid: PID {} has exited", spec.pid);
                            return ExitCode::FAILURE;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "waitpid: fstat pidfd for {}: {e}",
                                spec.pid
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                pidfds.push((spec.pid, fd));
            }
            Err(e) if e == rustix::io::Errno::SRCH && args.exited => {
                eprintln!("waitpid: PID {} has exited, skipping", spec.pid);
            }
            Err(e) => {
                eprintln!("waitpid: failed to open PID {}: {e}", spec.pid);
                if e == rustix::io::Errno::NOSYS {
                    return ExitCode::from(2);
                }
                return ExitCode::FAILURE;
            }
        }
    }

    let mut exited = specs.len() - pidfds.len();

    while exited < target_count && !pidfds.is_empty() {
        let timeout = deadline.map(|d| {
            let remaining = d.saturating_duration_since(Instant::now());
            Timespec {
                tv_sec: remaining.as_secs() as i64,
                tv_nsec: remaining.subsec_nanos() as i64,
            }
        });

        let mut fds: Vec<PollFd<'_>> = pidfds
            .iter()
            .map(|(_, fd)| PollFd::new(fd, PollFlags::IN))
            .collect();

        match poll(&mut fds, timeout.as_ref()) {
            Ok(0) => {
                if args.verbose {
                    println!("Timeout expired");
                }
                return ExitCode::from(3);
            }
            Ok(_) => {
                let mut i = 0;
                while i < pidfds.len() {
                    let mut pfd = [PollFd::new(&pidfds[i].1, PollFlags::IN)];
                    let ready = poll(
                        &mut pfd,
                        Some(&Timespec {
                            tv_sec: 0,
                            tv_nsec: 0,
                        }),
                    )
                    .unwrap_or(0);
                    if ready > 0 {
                        pidfds.swap_remove(i);
                        exited += 1;
                        if exited >= target_count {
                            break;
                        }
                    } else {
                        i += 1;
                    }
                }
            }
            Err(e) => {
                if e != rustix::io::Errno::INTR {
                    eprintln!("waitpid: poll failed: {e}");
                    return ExitCode::FAILURE;
                }
            }
        }

        if let Some(d) = deadline
            && Instant::now() >= d
        {
            if args.verbose {
                println!("Timeout expired");
            }
            return ExitCode::from(3);
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_pid() {
        let s = parse_pid("12345").unwrap();
        assert_eq!(s.pid, 12345);
        assert_eq!(s.inode, None);
    }

    #[test]
    fn parses_pid_with_inode() {
        let s = parse_pid("12345:67890").unwrap();
        assert_eq!(s.pid, 12345);
        assert_eq!(s.inode, Some(67890));
    }

    #[test]
    fn rejects_non_positive_pid() {
        assert!(parse_pid("0").is_err());
        assert!(parse_pid("-1").is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_pid("abc").is_err());
        assert!(parse_pid("12:abc").is_err());
    }
}
