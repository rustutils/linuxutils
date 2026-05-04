use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{BufRead, Write},
    path::Path,
    process::ExitCode,
};

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "rename",
    about = "Rename files by replacing occurrences of a string in their filenames",
    override_usage = "rename [options] <expression> <replacement> <file>..."
)]
pub struct Args {
    /// Act on the target of symlinks instead of the symlink itself
    #[arg(short = 's', long = "symlink")]
    symlink: bool,

    /// Show which files were renamed
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Don't actually rename, just show what would happen
    #[arg(short = 'n', long = "no-act")]
    no_act: bool,

    /// Replace all occurrences of expression, not just the first
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Replace the last occurrence instead of the first
    #[arg(short = 'l', long = "last")]
    last: bool,

    /// Don't overwrite existing files
    #[arg(short = 'o', long = "no-overwrite")]
    no_overwrite: bool,

    /// Ask before overwriting existing files
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// The expression to search for
    expression: String,

    /// The replacement string
    replacement: String,

    /// Files to rename
    #[arg(required = true)]
    files: Vec<String>,
}

/// Find the byte index where the search for `from` should start
/// inside `orig`. If neither `from` nor `to` contains a `/`, the
/// search is restricted to the path's basename, matching upstream
/// rename(1)'s `find_initial_replace`. Returns `Some(idx)` to a
/// position inside `orig`, or `None` if there's no match at all.
fn find_initial_replace(from: &str, to: &str, orig: &str) -> Option<usize> {
    let search_start = if !from.contains('/') && !to.contains('/') {
        // Like upstream's `ul_basename`: skip past the last '/'
        // (or any trailing slashes that came before it).
        let trimmed = orig.trim_end_matches('/');
        match trimmed.rfind('/') {
            Some(i) => i + 1,
            None => 0,
        }
    } else {
        0
    };
    if from.is_empty() {
        return Some(search_start);
    }
    orig[search_start..].find(from).map(|i| i + search_start)
}

/// Apply a single (or all / last) replacement to `orig`. Returns
/// `None` if no replacement was made (caller treats that as
/// "skip"). Mirrors upstream's `string_replace` but written in
/// terms of slice indexes.
fn replace_string(
    orig: &str,
    from: &str,
    to: &str,
    all: bool,
    last: bool,
) -> Option<String> {
    let initial = find_initial_replace(from, to, orig)?;

    if from.is_empty() {
        // Empty `from` means "insert `to`" — same edge cases as
        // upstream: single-shot inserts at the basename boundary,
        // `--last` inserts at end, `--all` between every char of
        // the search region.
        let region_start = initial;
        return Some(if all {
            let mut out = String::new();
            out.push_str(&orig[..region_start]);
            out.push_str(to);
            for ch in orig[region_start..].chars() {
                out.push(ch);
                out.push_str(to);
            }
            out
        } else if last {
            format!("{orig}{to}")
        } else {
            let mut out = String::new();
            out.push_str(&orig[..region_start]);
            out.push_str(to);
            out.push_str(&orig[region_start..]);
            out
        });
    }

    // Collect the indexes to replace at, all in `orig`'s coordinates.
    let mut positions = vec![initial];
    if all {
        let mut p = initial + from.len();
        while let Some(rel) = orig[p..].find(from) {
            positions.push(p + rel);
            p += rel + from.len();
        }
    } else if last {
        let mut p = initial + 1;
        let mut last_pos = initial;
        while let Some(rel) = orig[p..].find(from) {
            last_pos = p + rel;
            p = last_pos + 1;
        }
        positions = vec![last_pos];
    }

    let mut out = String::with_capacity(
        orig.len() - positions.len() * from.len() + positions.len() * to.len(),
    );
    let mut cursor = 0;
    for pos in positions {
        out.push_str(&orig[cursor..pos]);
        out.push_str(to);
        cursor = pos + from.len();
    }
    out.push_str(&orig[cursor..]);
    Some(out)
}

fn prompt_overwrite(new_name: &str) -> bool {
    eprint!("{new_name}: overwrite? ");
    std::io::stderr().flush().ok();
    let mut line = String::new();
    if std::io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    let trimmed = line.trim().to_lowercase();
    trimmed == "y" || trimmed == "yes"
}

/// `strerror(3)`-style description (no trailing `(os error N)`).
fn strerror(e: &std::io::Error) -> String {
    if let Some(errno) = e.raw_os_error() {
        let p = unsafe { libc::strerror(errno) };
        if !p.is_null() {
            let cs = unsafe { std::ffi::CStr::from_ptr(p) };
            return cs.to_string_lossy().into_owned();
        }
    }
    e.to_string()
}

/// Result of attempting to rename one file. Mirrors upstream's
/// per-file `ret`: `Renamed` (1) on success, `Skipped` (0) when
/// the path either doesn't match the expression or hits a
/// `--no-overwrite` / interactive skip, and `Failed` (2) on a
/// real error.
enum Outcome {
    Renamed,
    Skipped,
    Failed,
}

pub fn run(args: Args) -> ExitCode {
    let mut succeeded = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;

    for file in &args.files {
        // Upstream's `ul_basename` strips trailing slashes from
        // the input path before processing; replicate that.
        let file = file.trim_end_matches('/').to_string();
        if file.is_empty() {
            // All slashes — rename root, nonsensical.
            failed += 1;
            continue;
        }

        let outcome = if args.symlink {
            do_symlink(&file, &args)
        } else {
            do_file(&file, &args)
        };

        match outcome {
            Outcome::Renamed => succeeded += 1,
            Outcome::Skipped => skipped += 1,
            Outcome::Failed => failed += 1,
        }
    }

    if args.no_act {
        return ExitCode::SUCCESS;
    }

    let total = succeeded + failed + skipped;
    if failed == total {
        ExitCode::from(1)
    } else if failed > 0 {
        ExitCode::from(2)
    } else if succeeded == 0 {
        ExitCode::from(4)
    } else {
        ExitCode::SUCCESS
    }
}

fn do_file(file: &str, args: &Args) -> Outcome {
    let path = Path::new(file);
    if let Err(e) = path.symlink_metadata() {
        eprintln!("rename: stat of {file} failed: {}", strerror(&e));
        return Outcome::Failed;
    }

    let new_name = match replace_string(
        file,
        &args.expression,
        &args.replacement,
        args.all,
        args.last,
    ) {
        Some(n) if n != file => n,
        _ => return Outcome::Skipped,
    };

    let new_path = Path::new(&new_name);
    let target_exists = new_path.symlink_metadata().is_ok();

    if target_exists
        && (args.no_overwrite
            || (args.interactive
                && (args.no_act || !prompt_overwrite(&new_name))))
    {
        if args.verbose {
            println!("Skipping existing file: `{new_name}'");
        }
        return Outcome::Skipped;
    }

    let outcome = if args.no_act {
        Outcome::Renamed
    } else {
        match std::fs::rename(file, &new_name) {
            Ok(()) => Outcome::Renamed,
            Err(e) => {
                eprintln!(
                    "rename: {file}: rename to {new_name} failed: {}",
                    strerror(&e)
                );
                Outcome::Failed
            }
        }
    };

    // Upstream prints the verbose `s -> new` line *after* the
    // attempt, only when it succeeded (or in --no-act).
    if args.verbose
        && matches!(outcome, Outcome::Renamed)
        && (args.no_act || !matches!(outcome, Outcome::Failed))
    {
        println!("`{file}' -> `{new_name}'");
    }
    outcome
}

fn do_symlink(file: &str, args: &Args) -> Outcome {
    let path = Path::new(file);
    let target = match std::fs::read_link(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("rename: {file}: not a symlink: {}", strerror(&e));
            return Outcome::Failed;
        }
    };

    let target_str = target.to_string_lossy().to_string();
    let new_target = match replace_string(
        &target_str,
        &args.expression,
        &args.replacement,
        args.all,
        args.last,
    ) {
        Some(n) if n != target_str => n,
        _ => return Outcome::Skipped,
    };

    let new_target_path = Path::new(&new_target);

    let target_exists = new_target_path.symlink_metadata().is_ok();
    if target_exists
        && (args.no_overwrite
            || (args.interactive && !prompt_overwrite(&new_target)))
    {
        if args.verbose {
            println!("Skipping existing link: `{file}' -> `{new_target}'");
        }
        return Outcome::Skipped;
    }

    let outcome = if args.no_act {
        Outcome::Renamed
    } else {
        match (
            std::fs::remove_file(path),
            std::os::unix::fs::symlink(new_target_path, path),
        ) {
            (Ok(()), Ok(())) => Outcome::Renamed,
            (Err(e), _) => {
                eprintln!(
                    "rename: {file}: removing symlink failed: {}",
                    strerror(&e)
                );
                Outcome::Failed
            }
            (_, Err(e)) => {
                eprintln!(
                    "rename: {file}: symlinking to {new_target} failed: {}",
                    strerror(&e)
                );
                Outcome::Failed
            }
        }
    };

    if args.verbose && matches!(outcome, Outcome::Renamed) {
        // Upstream's symlink format omits the backticks around the
        // filename and uses a colon instead.
        println!("{file}: `{target_str}' -> `{new_target}'");
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replace(orig: &str, from: &str, to: &str) -> Option<String> {
        replace_string(orig, from, to, false, false)
    }

    #[test]
    fn first_match_in_basename() {
        assert_eq!(
            replace("foo.bar.baz", "bar", "qux"),
            Some("foo.qux.baz".to_string())
        );
    }

    #[test]
    fn replace_all() {
        assert_eq!(
            replace_string("foo.bar.bar", "bar", "qux", true, false),
            Some("foo.qux.qux".to_string())
        );
    }

    #[test]
    fn replace_last() {
        assert_eq!(
            replace_string("foo.bar.bar", "bar", "qux", false, true),
            Some("foo.bar.qux".to_string())
        );
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(replace("foobar", "xyz", "qux"), None);
    }

    #[test]
    fn empty_expression_inserts_at_basename_start() {
        assert_eq!(replace("abc", "", "X"), Some("Xabc".to_string()));
    }

    #[test]
    fn empty_expression_last_appends() {
        assert_eq!(
            replace_string("abc", "", "X", false, true),
            Some("abcX".to_string())
        );
    }

    #[test]
    fn search_restricted_to_basename_when_no_slashes() {
        // Path-shaped input: `from`/`to` lack slashes, so we should
        // only replace inside the trailing component. `rename/aa`
        // → "rename/xa", not "renxme/aa".
        assert_eq!(
            replace("rename/aa", "a", "x"),
            Some("rename/xa".to_string())
        );
    }

    #[test]
    fn slashy_expression_searches_whole_path() {
        // Once `from` contains a `/`, the basename rule no longer
        // applies — match anywhere.
        assert_eq!(
            replace("rename/aa", "rename/aa", "rename/xa"),
            Some("rename/xa".to_string())
        );
    }

    #[test]
    fn trailing_slash_input() {
        // The trailing slash leaves the basename "rename_dir" so
        // we still match.
        assert_eq!(
            replace("rename_dir/", "dir", "new_dir"),
            Some("rename_new_dir/".to_string())
        );
    }
}
