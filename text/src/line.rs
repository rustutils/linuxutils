use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, BufRead, Write},
    process::ExitCode,
};

/// Read one line from standard input and write it to standard output.
#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "line", version, about)]
pub struct Args {}

pub fn run(_args: Args) -> ExitCode {
    let stdin = io::stdin();
    let reader = stdin.lock();
    let mut stdout = io::stdout().lock();

    match line(reader, &mut stdout) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("line: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Read one line from `reader` and write it to `writer`, followed by a newline.
///
/// Returns `true` if a line was read, `false` on EOF.
pub fn line(
    mut reader: impl BufRead,
    writer: &mut impl Write,
) -> io::Result<bool> {
    let mut buf = String::new();
    let bytes_read = reader.read_line(&mut buf)?;

    // Strip the trailing newline if present — we always write our own.
    let line = buf.trim_end_matches('\n').trim_end_matches('\r');
    writeln!(writer, "{line}")?;

    Ok(bytes_read > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_first_line_only() {
        let input = b"hello\nworld\n";
        let mut output = Vec::new();
        let ok = line(&input[..], &mut output).unwrap();
        assert!(ok);
        assert_eq!(output, b"hello\n");
    }

    #[test]
    fn eof_returns_false() {
        let input = b"";
        let mut output = Vec::new();
        let ok = line(&input[..], &mut output).unwrap();
        assert!(!ok);
        assert_eq!(output, b"\n");
    }

    #[test]
    fn line_without_newline() {
        let input = b"hello";
        let mut output = Vec::new();
        let ok = line(&input[..], &mut output).unwrap();
        assert!(ok);
        assert_eq!(output, b"hello\n");
    }
}
