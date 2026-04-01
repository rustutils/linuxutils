use clap::Parser;
use std::{
    fs::File,
    os::unix::io::{AsRawFd, RawFd},
    process::ExitCode,
};

/// Set or examine pipe and FIFO buffer sizes.
///
/// Uses fcntl(2) with F_GETPIPE_SZ / F_SETPIPE_SZ to get or set the
/// internal buffer sizes of pipes and FIFOs.
#[derive(Parser)]
#[command(name = "pipesz", about = "Set or examine pipe and FIFO buffer sizes")]
pub struct Args {
    /// Report pipe buffer sizes
    #[arg(short = 'g', long)]
    get: bool,

    /// Set pipe buffer size in bytes
    #[arg(short = 's', long)]
    set: Option<i32>,

    /// Operate on a FIFO/pipe at this path (repeatable)
    #[arg(short = 'f', long = "file", action = clap::ArgAction::Append)]
    file: Vec<String>,

    /// Operate on this file descriptor number (repeatable)
    #[arg(short = 'n', long = "fd", action = clap::ArgAction::Append)]
    fd: Vec<RawFd>,

    /// Shorthand for --fd 0
    #[arg(short = 'i', long)]
    stdin: bool,

    /// Shorthand for --fd 1
    #[arg(short = 'o', long)]
    stdout: bool,

    /// Shorthand for --fd 2
    #[arg(short = 'e', long)]
    stderr: bool,

    /// Exit immediately on any error
    #[arg(short = 'c', long)]
    check: bool,

    /// Suppress non-fatal warnings
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Emit headers (get) or print actual sizes (set)
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Command to execute after setting pipe sizes
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

fn get_pipe_sz(fd: RawFd) -> Result<i32, std::io::Error> {
    let ret = unsafe { libc::fcntl(fd, libc::F_GETPIPE_SZ) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

fn set_pipe_sz(fd: RawFd, size: i32) -> Result<i32, std::io::Error> {
    let ret = unsafe { libc::fcntl(fd, libc::F_SETPIPE_SZ, size) };
    if ret < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(ret)
    }
}

fn get_unread(fd: RawFd) -> i32 {
    let mut count: libc::c_int = 0;
    if unsafe { libc::ioctl(fd, libc::FIONREAD, &mut count) } < 0 {
        0
    } else {
        count
    }
}

fn fd_name(fd: RawFd) -> String {
    match fd {
        0 => "stdin".to_string(),
        1 => "stdout".to_string(),
        2 => "stderr".to_string(),
        n => format!("fd {n}"),
    }
}

fn collect_fds(args: &Args) -> Vec<(String, RawFd, Option<File>)> {
    let mut fds: Vec<(String, RawFd, Option<File>)> = Vec::new();

    if args.stdin {
        fds.push(("stdin".to_string(), 0, None));
    }
    if args.stdout {
        fds.push(("stdout".to_string(), 1, None));
    }
    if args.stderr {
        fds.push(("stderr".to_string(), 2, None));
    }
    for &n in &args.fd {
        fds.push((fd_name(n), n, None));
    }
    for path in &args.file {
        match File::options().read(true).write(true).open(path) {
            Ok(f) => {
                let raw = f.as_raw_fd();
                fds.push((path.clone(), raw, Some(f)));
            }
            Err(e) => {
                if !args.quiet {
                    eprintln!("pipesz: {path}: {e}");
                }
            }
        }
    }
    fds
}

pub fn run(args: Args) -> ExitCode {
    let mut fds = collect_fds(&args);

    if args.get {
        if fds.is_empty() {
            fds.push(("stdin".to_string(), 0, None));
        }
        if args.verbose {
            println!("name\tsize\tunread");
        }
        let mut failed = false;
        for (name, fd, _file) in &fds {
            match get_pipe_sz(*fd) {
                Ok(sz) => {
                    let unread = get_unread(*fd);
                    println!("{name}\t{sz}\t{unread}");
                }
                Err(e) => {
                    eprintln!("pipesz: {name}: {e}");
                    failed = true;
                    if args.check {
                        return ExitCode::FAILURE;
                    }
                }
            }
        }
        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    if let Some(size) = args.set {
        if fds.is_empty() {
            fds.push(("stdout".to_string(), 1, None));
        }
        let mut failed = false;
        for (name, fd, _file) in &fds {
            match set_pipe_sz(*fd, size) {
                Ok(actual) => {
                    if args.verbose {
                        println!("{name}\t{actual}");
                    }
                }
                Err(e) => {
                    eprintln!("pipesz: {name}: {e}");
                    failed = true;
                    if args.check {
                        return ExitCode::FAILURE;
                    }
                }
            }
        }

        if !args.command.is_empty() {
            let err = std::os::unix::process::CommandExt::exec(
                std::process::Command::new(&args.command[0])
                    .args(&args.command[1..]),
            );
            eprintln!("pipesz: failed to exec '{}': {err}", args.command[0]);
            return ExitCode::FAILURE;
        }

        return if failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    eprintln!("pipesz: one of --get or --set must be specified");
    ExitCode::FAILURE
}
