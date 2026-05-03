use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, BufRead, Write},
    process::ExitCode,
};

const TAB_WIDTH: usize = 8;

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
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

/// Faithful port of upstream colrm's three-phase algorithm:
/// 1. Copy characters as-is while the running column is still
///    below `first`.
/// 2. (If a tab straddled the boundary) emit padding spaces for
///    its visible portion before `first`, then read+drop chars
///    until the running column reaches `last`.
/// 3. (If we overshot `last`) pad once more with spaces, then
///    emit any remaining characters of the line verbatim.
///
/// Column tracking matches upstream: tab advances to the next
/// 8-column stop; backspace decrements (clamped to 0); other
/// characters advance by 1. (Wide-character widths from
/// `wcwidth(3)` aren't modeled — same simplification we already
/// had.)
fn process_line(
    line: &str,
    first: Option<usize>,
    last: Option<usize>,
    out: &mut impl Write,
) -> io::Result<()> {
    // Upstream uses 0 as a sentinel meaning "no first / no last".
    // Our public API uses Option<usize>; map None → 0.
    let first = first.unwrap_or(0);
    let last = last.unwrap_or(0);

    let mut chars = line.chars();
    let mut ct: usize = 0;
    let mut last_w: usize = 0;
    let mut breaking_char: Option<char> = None;

    // ---- Phase 1: copy until ct reaches first.
    for c in chars.by_ref() {
        let w = char_width(c, ct);
        ct = step_ct(c, ct, w);
        last_w = w;
        if first == 0 || ct < first {
            write!(out, "{c}")?;
        } else {
            breaking_char = Some(c);
            break;
        }
    }

    // No `first` set, or line ended before reaching `first`. The
    // copy in phase 1 already emitted everything; just terminate.
    if first == 0 || breaking_char.is_none() {
        writeln!(out)?;
        return Ok(());
    }

    // ---- Phase 2 (pad): if the breaking char (a tab) straddled
    // the boundary, emit spaces for its visible portion before
    // `first`.
    let span_start = ct.saturating_sub(last_w).saturating_add(1);
    for _ in span_start..first {
        out.write_all(b" ")?;
    }

    // ---- Phase 2 (eat): drop characters while ct stays below
    // `last`. `last == 0` means "until end of line".
    for c in chars.by_ref() {
        if !(last == 0 || ct < last) {
            // ct already reached `last`; the unhandled `c` belongs
            // to phase 3 — but we've consumed it from the iterator.
            // Re-prepend by leaning on the rebuilt loop below: we
            // pass `c` through directly.
            return finish_with_overshoot(out, last, ct, Some(c), chars);
        }
        let w = char_width(c, ct);
        ct = step_ct(c, ct, w);
    }

    // Iterator ran out during the eat phase — line had no
    // surviving suffix.
    finish_with_overshoot(out, last, ct, None, chars)
}

fn finish_with_overshoot(
    out: &mut impl Write,
    last: usize,
    ct: usize,
    held: Option<char>,
    rest: std::str::Chars<'_>,
) -> io::Result<()> {
    let mut padded = false;
    let emit =
        |c: char, out: &mut dyn Write, padded: &mut bool| -> io::Result<()> {
            if !*padded && last != 0 && last < ct {
                for _ in last..ct {
                    out.write_all(b" ")?;
                }
                *padded = true;
            }
            write!(out, "{c}")
        };
    if let Some(c) = held {
        emit(c, out, &mut padded)?;
    }
    for c in rest {
        emit(c, out, &mut padded)?;
    }
    // If nothing followed, still emit the padding (matches
    // upstream's behaviour for lines that end exactly at `last`).
    if !padded && last != 0 && last < ct {
        for _ in last..ct {
            out.write_all(b" ")?;
        }
    }
    writeln!(out)?;
    Ok(())
}

fn char_width(c: char, ct: usize) -> usize {
    match c {
        '\t' => TAB_WIDTH - (ct % TAB_WIDTH),
        '\x08' => 0,
        _ => 1,
    }
}

fn step_ct(c: char, ct: usize, w: usize) -> usize {
    if c == '\x08' {
        ct.saturating_sub(1)
    } else {
        ct + w
    }
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
