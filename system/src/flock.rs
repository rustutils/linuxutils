use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::{
    fd::{AsFd, BorrowedFd, OwnedFd},
    fs::{FlockOperation, OFlags, flock},
};
use std::{
    os::unix::process::CommandExt,
    process,
    process::ExitCode,
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "flock",
    about = "Manage locks from shell scripts",
    override_usage = "flock [options] <file>|<directory> <command> [<argument>...]\n       \
                      flock [options] <file>|<directory> -c <command>\n       \
                      flock [options] <file descriptor number>"
)]
pub struct Args {
    /// Get a shared lock
    #[arg(short, long)]
    shared: bool,

    /// Get an exclusive lock (default)
    #[arg(short = 'x', short_alias = 'e', long)]
    exclusive: bool,

    /// Remove a lock
    #[arg(short, long)]
    unlock: bool,

    /// Fail rather than wait if the lock cannot be immediately acquired
    #[arg(short, long, alias = "nb")]
    nonblock: bool,

    /// Wait at most this many seconds. Fractions are allowed
    #[arg(short = 'w', long, value_name = "secs", alias = "wait")]
    timeout: Option<f64>,

    /// Exit code when the lock cannot be acquired (default: 1)
    #[arg(short = 'E', long, value_name = "number", default_value = "1")]
    conflict_exit_code: u8,

    /// Close the file descriptor before executing the command
    #[arg(short = 'o', long)]
    close: bool,

    /// Execute command without forking
    #[arg(short = 'F', long)]
    no_fork: bool,

    /// Run a single command string through the shell
    #[arg(short = 'c', long, value_name = "command")]
    command: Option<String>,

    /// Increase verbosity
    #[arg(long)]
    verbose: bool,

    /// File, directory, or open file descriptor number
    #[arg(required = true)]
    file: String,

    /// Command and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn acquire_lock(
    fd: BorrowedFd,
    op: FlockOperation,
    nonblock: bool,
    timeout: Option<f64>,
    conflict_exit_code: u8,
    verbose: bool,
) -> Result<(), ExitCode> {
    let effective_nonblock = nonblock || matches!(timeout, Some(t) if t == 0.0);

    if effective_nonblock {
        let nb_op = match op {
            FlockOperation::LockShared => FlockOperation::NonBlockingLockShared,
            FlockOperation::LockExclusive => {
                FlockOperation::NonBlockingLockExclusive
            }
            other => other,
        };
        flock(fd, nb_op).map_err(|_| ExitCode::from(conflict_exit_code))?;
        return Ok(());
    }

    if let Some(secs) = timeout {
        let deadline = Instant::now() + Duration::from_secs_f64(secs);
        let nb_op = match op {
            FlockOperation::LockShared => FlockOperation::NonBlockingLockShared,
            FlockOperation::LockExclusive => {
                FlockOperation::NonBlockingLockExclusive
            }
            other => other,
        };
        loop {
            match flock(fd, nb_op) {
                Ok(()) => return Ok(()),
                Err(_) => {
                    let now = Instant::now();
                    if now >= deadline {
                        if verbose {
                            eprintln!(
                                "flock: timeout while waiting to get lock"
                            );
                        }
                        return Err(ExitCode::from(conflict_exit_code));
                    }
                    let remaining = deadline - now;
                    std::thread::sleep(
                        remaining.min(Duration::from_millis(100)),
                    );
                }
            }
        }
    }

    // Blocking acquire.
    flock(fd, op).map_err(|e| {
        eprintln!("flock: {e}");
        ExitCode::FAILURE
    })
}

pub fn run(args: Args) -> ExitCode {
    // Determine lock operation.
    let lock_op = if args.unlock {
        FlockOperation::Unlock
    } else if args.shared {
        FlockOperation::LockShared
    } else {
        FlockOperation::LockExclusive
    };

    // Determine if `file` is a bare file descriptor number or a path.
    let is_fd_num = args.file.parse::<i32>().is_ok()
        && args.args.is_empty()
        && args.command.is_none();

    let owned_fd: Option<OwnedFd>;
    let borrowed: BorrowedFd;

    if is_fd_num {
        let n: i32 = args.file.parse().unwrap();
        // SAFETY: the caller is responsible for passing a valid open fd.
        owned_fd = None;
        borrowed = unsafe { BorrowedFd::borrow_raw(n) };
    } else {
        let flags = OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW;
        let perms = rustix::fs::Mode::RUSR
            | rustix::fs::Mode::WUSR
            | rustix::fs::Mode::RGRP
            | rustix::fs::Mode::WGRP
            | rustix::fs::Mode::ROTH
            | rustix::fs::Mode::WOTH;

        // Try RDWR first; fall back to RDONLY for read-only files/dirs.
        let fd = rustix::fs::open(&args.file, flags, perms)
            .or_else(|_| {
                rustix::fs::open(
                    &args.file,
                    OFlags::RDONLY | OFlags::CREATE | OFlags::NOFOLLOW,
                    perms,
                )
            })
            .or_else(|_| {
                // For directories, RDONLY without CREAT.
                rustix::fs::open(
                    &args.file,
                    OFlags::RDONLY,
                    rustix::fs::Mode::empty(),
                )
            });

        match fd {
            Ok(f) => {
                owned_fd = Some(f);
                borrowed = owned_fd.as_ref().unwrap().as_fd();
            }
            Err(e) => {
                eprintln!("flock: {}: {e}", args.file);
                return ExitCode::FAILURE;
            }
        }
    };

    if args.verbose {
        eprintln!("flock: getting lock...");
    }

    if let Err(code) = acquire_lock(
        borrowed,
        lock_op,
        args.nonblock,
        args.timeout,
        args.conflict_exit_code,
        args.verbose,
    ) {
        return code;
    }

    if args.verbose {
        eprintln!("flock: got lock");
    }

    // Build the command to run.
    let cmd_parts: Vec<String> = if let Some(ref cmd) = args.command {
        vec!["sh".to_string(), "-c".to_string(), cmd.clone()]
    } else if !args.args.is_empty() {
        args.args.clone()
    } else {
        // No command — just hold the lock until this process exits.
        return ExitCode::SUCCESS;
    };

    let (prog, prog_args) = cmd_parts.split_first().unwrap();

    if args.close {
        drop(owned_fd);
    }

    if args.no_fork {
        let err = process::Command::new(prog).args(prog_args).exec();
        eprintln!("flock: {prog}: {err}");
        return ExitCode::FAILURE;
    }

    match process::Command::new(prog).args(prog_args).status() {
        Ok(status) => {
            let code = status.code().unwrap_or(1) as u8;
            ExitCode::from(code)
        }
        Err(e) => {
            eprintln!("flock: {prog}: {e}");
            ExitCode::FAILURE
        }
    }
}
