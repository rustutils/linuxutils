use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    process::ExitCode,
};

/// Reverse lines characterwise.
#[derive(Parser)]
#[command(name = "rev", version, about)]
pub struct Args {
    /// Use '\0' as the line separator instead of newline.
    #[arg(short = '0', long)]
    zero: bool,

    /// Files to read. If none are given, reads from standard input.
    files: Vec<PathBuf>,
}

pub fn run(args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    if args.files.is_empty() {
        let stdin = io::stdin();
        let reader = stdin.lock();
        if let Err(e) = rev(reader, &mut writer, args.zero) {
            eprintln!("rev: {e}");
            return ExitCode::FAILURE;
        }
    } else {
        for path in &args.files {
            let file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("rev: {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            };
            let reader = BufReader::new(file);
            if let Err(e) = rev(reader, &mut writer, args.zero) {
                eprintln!("rev: {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}

/// Reverse each line from `reader` and write to `writer`.
///
/// If `zero` is true, lines are delimited by '\0' instead of '\n'.
pub fn rev(
    mut reader: impl BufRead,
    writer: &mut impl Write,
    zero: bool,
) -> io::Result<()> {
    let delimiter = if zero { b'\0' } else { b'\n' };
    let mut buf = Vec::new();

    loop {
        buf.clear();
        let bytes_read = reader.read_until(delimiter, &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        let has_delimiter = buf.last() == Some(&delimiter);
        let line = if has_delimiter {
            &buf[..buf.len() - 1]
        } else {
            &buf
        };

        let s = String::from_utf8_lossy(line);
        let reversed: String = s.chars().rev().collect();
        writer.write_all(reversed.as_bytes())?;

        if has_delimiter {
            writer.write_all(&[delimiter])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_simple_lines() {
        let input = b"hello\nworld\n";
        let mut output = Vec::new();
        rev(&input[..], &mut output, false).unwrap();
        assert_eq!(output, b"olleh\ndlrow\n");
    }

    #[test]
    fn reverse_no_trailing_newline() {
        let input = b"hello\nworld";
        let mut output = Vec::new();
        rev(&input[..], &mut output, false).unwrap();
        assert_eq!(output, b"olleh\ndlrow");
    }

    #[test]
    fn reverse_empty_input() {
        let input = b"";
        let mut output = Vec::new();
        rev(&input[..], &mut output, false).unwrap();
        assert_eq!(output, b"");
    }

    #[test]
    fn reverse_empty_lines() {
        let input = b"\n\n\n";
        let mut output = Vec::new();
        rev(&input[..], &mut output, false).unwrap();
        assert_eq!(output, b"\n\n\n");
    }

    #[test]
    fn reverse_zero_terminated() {
        let input = b"hello\0world\0";
        let mut output = Vec::new();
        rev(&input[..], &mut output, true).unwrap();
        assert_eq!(output, b"olleh\0dlrow\0");
    }

    #[test]
    fn reverse_unicode() {
        let input = "café\n".as_bytes();
        let mut output = Vec::new();
        rev(&input[..], &mut output, false).unwrap();
        assert_eq!(output, "éfac\n".as_bytes());
    }
}
