use clap::Parser;
use std::{
    io::{self, BufRead, Write},
    process::ExitCode,
};

const TAB_WIDTH: usize = 8;

#[derive(Parser)]
#[command(name = "colrm", version, about = "Remove columns from a file")]
pub struct Args {
    /// First column to remove (1-based)
    pub first: Option<usize>,
    /// Last column to remove (1-based, inclusive)
    pub last: Option<usize>,
}

pub fn run(args: Args) -> ExitCode {
    let first = match args.first {
        Some(0) => {
            eprintln!("colrm: first column must be at least 1");
            return ExitCode::FAILURE;
        }
        Some(f) => Some(f),
        None => None,
    };

    let last = args.last;

    if let (Some(f), Some(l)) = (first, last)
        && l < f
    {
        eprintln!("colrm: last column must be >= first column");
        return ExitCode::FAILURE;
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("colrm: {e}");
                return ExitCode::FAILURE;
            }
        };

        if let Err(e) = process_line(&line, first, last, &mut out) {
            eprintln!("colrm: {e}");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

fn process_line(
    line: &str,
    first: Option<usize>,
    last: Option<usize>,
    out: &mut impl Write,
) -> io::Result<()> {
    let first = match first {
        Some(f) => f,
        None => {
            writeln!(out, "{line}")?;
            return Ok(());
        }
    };

    // Track visual column position (1-based).
    let mut col: usize = 1;

    for ch in line.chars() {
        if ch == '\t' {
            let next_tab = next_tab_stop(col);
            // A tab spans from col to next_tab-1 (visually).
            // We need to output the visible portions of the tab that fall
            // outside the removed range.
            for c in col..next_tab {
                if c < first || last.is_some_and(|l| c > l) {
                    out.write_all(b" ")?;
                }
            }
            col = next_tab;
        } else if ch == '\x08' {
            // Backspace moves column back (like original colrm).
            if col < first || last.is_some_and(|l| col > l) {
                write!(out, "{ch}")?;
            }
            col = col.saturating_sub(1);
        } else {
            if col < first || last.is_some_and(|l| col > l) {
                write!(out, "{ch}")?;
            }
            col += 1;
        }
    }

    writeln!(out)?;
    Ok(())
}

fn next_tab_stop(col: usize) -> usize {
    // Columns are 1-based. Tab stops at 9, 17, 25, ...
    col + TAB_WIDTH - ((col - 1) % TAB_WIDTH)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_colrm(
        input: &str,
        first: Option<usize>,
        last: Option<usize>,
    ) -> String {
        let mut out = Vec::new();
        process_line(input, first, last, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn no_args_passes_through() {
        assert_eq!(run_colrm("hello world", None, None), "hello world\n");
    }

    #[test]
    fn remove_from_column() {
        assert_eq!(run_colrm("hello world", Some(3), None), "he\n");
    }

    #[test]
    fn remove_range() {
        assert_eq!(run_colrm("hello world", Some(3), Some(5)), "he world\n");
    }

    #[test]
    fn tab_expansion_in_removed_range() {
        // "ab\tcd" -> visual: "ab      cd" (ab at 1-2, tab fills 3-8, cd at 9-10)
        // Remove columns 3-8 -> "abcd"
        assert_eq!(run_colrm("ab\tcd", Some(3), Some(8)), "abcd\n");
    }

    #[test]
    fn tab_partial_removal() {
        // "ab\tcd" -> remove column 4 to 6
        // Columns 3-8 are the tab. Keep col 3, remove 4-6, keep 7-8, keep cd at 9-10.
        assert_eq!(run_colrm("ab\tcd", Some(4), Some(6)), "ab   cd\n");
    }

    #[test]
    fn empty_line() {
        assert_eq!(run_colrm("", Some(1), None), "\n");
    }

    #[test]
    fn first_beyond_line_length() {
        assert_eq!(run_colrm("hello", Some(20), None), "hello\n");
    }

    #[test]
    fn remove_first_column() {
        assert_eq!(run_colrm("hello", Some(1), Some(1)), "ello\n");
    }
}
