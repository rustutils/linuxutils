use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, Write},
    process::ExitCode,
};

/// Read one line from standard input and write it to standard output.
///
/// Reads byte-by-byte directly from FD 0 (no `BufReader`) so that
/// chained invocations like `(line && line) <pipe` consume exactly
/// one line each — a buffered reader would over-read and steal the
/// rest from the next process.
///
/// Behaviour matches POSIX:
///   - exit 0 if a `\n` is read and echoed
///   - exit 1 if EOF is hit before a `\n`; a trailing `\n` is added
///     to stdout in that case
///
/// Extra positional arguments are ignored to match upstream
/// `line(1)` (the test harness sometimes passes the literal string
/// `line` as a buffer-size hint).
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "line", version, about)]
pub struct Args {
    /// Ignored — accepted for compatibility with upstream `line(1)`.
    #[arg(trailing_var_arg = true, hide = true)]
    pub _ignored: Vec<String>,
}

pub fn run(_args: Args) -> ExitCode {
    let mut stdout = io::stdout().lock();
    match copy_one_line(&mut stdout) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("line: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Read bytes from FD 0 one at a time, echoing each to `writer`,
/// until a `\n` is seen or EOF is reached. Returns `Ok(true)` if a
/// newline was consumed, `Ok(false)` on EOF (in which case a single
/// `\n` is appended to keep output well-formed).
fn copy_one_line(writer: &mut impl Write) -> io::Result<bool> {
    let mut byte = 0u8;
    loop {
        let n = unsafe {
            libc::read(0, &mut byte as *mut u8 as *mut libc::c_void, 1)
        };
        if n < 0 {
            let e = io::Error::last_os_error();
            if e.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(e);
        }
        if n == 0 {
            // EOF before newline — POSIX line(1) writes one anyway.
            writer.write_all(b"\n")?;
            writer.flush()?;
            return Ok(false);
        }
        writer.write_all(std::slice::from_ref(&byte))?;
        if byte == b'\n' {
            writer.flush()?;
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    // The behavioural tests live in `tests/util-linux/ts/line/line` —
    // they exercise the FD-0 read loop end-to-end. Unit-testing the
    // helper here would require redirecting FD 0, which isn't worth
    // the ceremony for a 20-line function.
}
