use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, Write},
    process::ExitCode,
};

/// Read one line from standard input and write it to standard output.
///
/// Exits with status 0 if a whole line was read. If the input ends before
/// a newline, line still writes a newline and exits with status 1.
//
// Input is read one byte at a time straight from file descriptor 0, with
// no BufReader, so that `(line && line) <pipe` reads exactly one line
// each. A buffered reader would read too far and take input meant for
// the next process.
//
// Extra arguments are ignored, as upstream line(1) does. Its test suite
// sometimes passes the string "line" as a buffer size hint.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "line", version)]
pub struct Args {
    /// Ignored. Accepted for compatibility with upstream line(1).
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
