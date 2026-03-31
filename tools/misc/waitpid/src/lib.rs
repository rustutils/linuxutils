use clap::Parser;
use rustix::{
    event::{PollFd, PollFlags, Timespec, poll},
    process::{Pid, PidfdFlags, pidfd_open},
};
use std::{process::ExitCode, time::Instant};

#[derive(Parser)]
#[command(name = "waitpid", about = "Wait for arbitrary processes to exit")]
pub struct Args {
    /// Be more verbose
    #[arg(short, long)]
    verbose: bool,

    /// Maximum number of seconds to wait
    #[arg(short, long)]
    timeout: Option<u64>,

    /// Don't error on already-exited PIDs
    #[arg(short, long)]
    exited: bool,

    /// Number of process exits to wait for
    #[arg(short, long)]
    count: Option<usize>,

    /// PIDs to wait for
    #[arg(required = true)]
    pids: Vec<i32>,
}

pub fn run(args: Args) -> ExitCode {
    let target_count = args.count.unwrap_or(args.pids.len());
    let deadline = args
        .timeout
        .map(|t| Instant::now() + std::time::Duration::from_secs(t));

    let mut pidfds = Vec::new();
    for &raw_pid in &args.pids {
        let pid = match Pid::from_raw(raw_pid) {
            Some(p) => p,
            None => {
                eprintln!("waitpid: invalid PID: {raw_pid}");
                return ExitCode::FAILURE;
            }
        };

        match pidfd_open(pid, PidfdFlags::NONBLOCK) {
            Ok(fd) => {
                if args.verbose {
                    eprintln!("waitpid: waiting for PID {raw_pid}");
                }
                pidfds.push((raw_pid, fd));
            }
            Err(e) => {
                if e == rustix::io::Errno::SRCH && args.exited {
                    if args.verbose {
                        eprintln!("waitpid: PID {raw_pid} already exited");
                    }
                } else {
                    eprintln!("waitpid: failed to open PID {raw_pid}: {e}");
                    if e == rustix::io::Errno::NOSYS {
                        return ExitCode::from(2);
                    }
                    return ExitCode::FAILURE;
                }
            }
        }
    }

    let mut exited = args.pids.len() - pidfds.len(); // already-exited count

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
                // Timeout.
                if args.verbose {
                    eprintln!("waitpid: timeout");
                }
                return ExitCode::from(3);
            }
            Ok(_) => {
                // Check which pidfds are ready (process exited).
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
                        let pid = pidfds[i].0;
                        if args.verbose {
                            eprintln!("waitpid: PID {pid} exited");
                        }
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
                eprintln!("waitpid: timeout");
            }
            return ExitCode::from(3);
        }
    }

    ExitCode::SUCCESS
}
