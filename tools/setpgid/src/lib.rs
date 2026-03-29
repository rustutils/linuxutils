use clap::Parser;
use rustix::process::{getpid, setpgid as do_setpgid};
use rustix::termios::tcsetpgrp;
use std::ffi::CString;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "setpgid", version, about = "Run a program in a new process group")]
pub struct Args {
    /// Make the new process group the foreground process group
    #[arg(short = 'f', long = "foreground")]
    foreground: bool,

    /// Program and its arguments
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    pub command: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    let pid = getpid();

    if let Err(e) = do_setpgid(Some(pid), Some(pid)) {
        eprintln!("setpgid: setpgid: {e}");
        return ExitCode::FAILURE;
    }

    if args.foreground {
        let stderr = std::io::stderr();
        if let Err(e) = tcsetpgrp(&stderr, pid) {
            eprintln!("setpgid: tcsetpgrp: {e}");
            return ExitCode::FAILURE;
        }
    }

    let program = &args.command[0];
    let arguments = &args.command[1..];
    exec_program(program, arguments)
}

fn exec_program(program: &str, arguments: &[String]) -> ExitCode {
    let c_program = match CString::new(program.as_bytes()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("setpgid: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut c_args: Vec<CString> = vec![c_program.clone()];
    for arg in arguments {
        match CString::new(arg.as_bytes()) {
            Ok(s) => c_args.push(s),
            Err(e) => {
                eprintln!("setpgid: {e}");
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
    eprintln!("setpgid: exec {program}: {}", std::io::Error::last_os_error());
    ExitCode::from(127)
}
