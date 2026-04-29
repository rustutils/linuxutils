use clap::Parser;
use rustix::process::{
    Pid, WaitOptions, getpgrp, getpid, setsid as do_setsid, waitpid,
};
use std::{ffi::CString, os::unix::io::AsRawFd, process::ExitCode};

#[derive(Parser)]
#[command(name = "setsid", version, about = "Run a program in a new session")]
pub struct Args {
    /// Set the controlling terminal to the current one
    #[arg(short = 'c', long = "ctty")]
    ctty: bool,

    /// Always create a new process
    #[arg(short = 'f', long = "fork")]
    fork: bool,

    /// Wait for the program to end and return its exit status
    #[arg(short = 'w', long = "wait")]
    wait: bool,

    /// Program and its arguments
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        required = true
    )]
    pub command: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    let need_fork = args.fork || is_process_group_leader();

    if need_fork {
        // fork() is not provided by rustix because it is inherently unsafe.
        let pid = unsafe { libc::fork() };
        match pid {
            -1 => {
                eprintln!("setsid: fork: {}", std::io::Error::last_os_error());
                return ExitCode::FAILURE;
            }
            0 => {
                // Child: fall through to setsid + exec.
            }
            child_pid => {
                // Parent.
                if !args.wait {
                    return ExitCode::SUCCESS;
                }
                return wait_for_child(child_pid);
            }
        }
    }

    if let Err(e) = do_setsid() {
        eprintln!("setsid: setsid: {e}");
        return ExitCode::FAILURE;
    }

    if args.ctty {
        let stdin = std::io::stdin();
        let fd = stdin.as_raw_fd();
        // TIOCSCTTY: set controlling terminal.
        let ret = unsafe { libc::ioctl(fd, libc::TIOCSCTTY, 0) };
        if ret < 0 {
            eprintln!(
                "setsid: ioctl TIOCSCTTY: {}",
                std::io::Error::last_os_error()
            );
            return ExitCode::FAILURE;
        }
    }

    let program = &args.command[0];
    let arguments = &args.command[1..];
    exec_program(program, arguments)
}

fn is_process_group_leader() -> bool {
    let pid = getpid();
    let pgrp = getpgrp();
    pid.as_raw_nonzero() == pgrp.as_raw_nonzero()
}

fn wait_for_child(pid: i32) -> ExitCode {
    let pid = Pid::from_raw(pid);
    let Some(pid) = pid else {
        return ExitCode::FAILURE;
    };
    loop {
        match waitpid(Some(pid), WaitOptions::empty()) {
            Ok(Some((_pid, status))) => {
                if let Some(code) = status.exit_status() {
                    return ExitCode::from(code as u8);
                }
                if status.terminating_signal().is_some() {
                    return ExitCode::FAILURE;
                }
                // Stopped/continued — keep waiting.
            }
            Ok(None) => continue,
            Err(e) => {
                eprintln!("setsid: waitpid: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
}

fn exec_program(program: &str, arguments: &[String]) -> ExitCode {
    let c_program = match CString::new(program.as_bytes()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("setsid: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut c_args: Vec<CString> = vec![c_program.clone()];
    for arg in arguments {
        match CString::new(arg.as_bytes()) {
            Ok(s) => c_args.push(s),
            Err(e) => {
                eprintln!("setsid: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let c_ptrs: Vec<*const libc::c_char> = c_args
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe {
        libc::execvp(c_program.as_ptr(), c_ptrs.as_ptr());
    }

    // execvp only returns on error.
    eprintln!(
        "setsid: exec {program}: {}",
        std::io::Error::last_os_error()
    );
    ExitCode::from(127)
}
