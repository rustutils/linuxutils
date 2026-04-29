use clap::Parser;
use std::{
    io::{BufRead, Write},
    path::Path,
    process::ExitCode,
};

#[derive(Parser)]
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

fn replace_string(
    input: &str,
    expression: &str,
    replacement: &str,
    all: bool,
    last: bool,
) -> String {
    if expression.is_empty() {
        if all {
            let mut result = String::with_capacity(
                input.len() + replacement.len() * (input.chars().count() + 1),
            );
            result.push_str(replacement);
            for ch in input.chars() {
                result.push(ch);
                result.push_str(replacement);
            }
            result
        } else if last {
            let mut result =
                String::with_capacity(input.len() + replacement.len());
            result.push_str(input);
            result.push_str(replacement);
            result
        } else {
            let mut result =
                String::with_capacity(input.len() + replacement.len());
            result.push_str(replacement);
            result.push_str(input);
            result
        }
    } else if all {
        input.replace(expression, replacement)
    } else if last {
        match input.rfind(expression) {
            Some(pos) => {
                let mut result = String::with_capacity(
                    input.len() - expression.len() + replacement.len(),
                );
                result.push_str(&input[..pos]);
                result.push_str(replacement);
                result.push_str(&input[pos + expression.len()..]);
                result
            }
            None => input.to_string(),
        }
    } else {
        match input.find(expression) {
            Some(pos) => {
                let mut result = String::with_capacity(
                    input.len() - expression.len() + replacement.len(),
                );
                result.push_str(&input[..pos]);
                result.push_str(replacement);
                result.push_str(&input[pos + expression.len()..]);
                result
            }
            None => input.to_string(),
        }
    }
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

pub fn run(args: Args) -> ExitCode {
    let use_full_path =
        args.expression.contains('/') || args.replacement.contains('/');

    let mut succeeded = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;

    for file in &args.files {
        let path = Path::new(file);

        if args.symlink {
            let target = match std::fs::read_link(path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("rename: {file}: not a symlink: {e}");
                    failed += 1;
                    continue;
                }
            };

            let target_str = target.to_string_lossy();
            let new_target = replace_string(
                &target_str,
                &args.expression,
                &args.replacement,
                args.all,
                args.last,
            );

            if new_target == target_str.as_ref() {
                skipped += 1;
                continue;
            }

            let new_target_path = Path::new(&new_target);

            if args.no_overwrite && new_target_path.exists() {
                if args.verbose {
                    eprintln!(
                        "rename: {file}: not overwriting symlink target `{new_target}'"
                    );
                }
                skipped += 1;
                continue;
            }

            if args.verbose || args.no_act {
                println!("`{file}': `{target_str}' -> `{new_target}'");
            }

            if !args.no_act {
                if let Err(e) = std::fs::remove_file(path) {
                    eprintln!("rename: {file}: removing symlink failed: {e}");
                    failed += 1;
                    continue;
                }
                #[cfg(unix)]
                {
                    if let Err(e) =
                        std::os::unix::fs::symlink(new_target_path, path)
                    {
                        eprintln!(
                            "rename: {file}: creating symlink failed: {e}"
                        );
                        failed += 1;
                        continue;
                    }
                }
                succeeded += 1;
            }
        } else {
            let (dir, name_to_transform) = if use_full_path {
                (String::new(), file.to_string())
            } else {
                let parent = path
                    .parent()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let basename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.to_string());
                (parent, basename)
            };

            let new_name_part = replace_string(
                &name_to_transform,
                &args.expression,
                &args.replacement,
                args.all,
                args.last,
            );

            if new_name_part == name_to_transform {
                skipped += 1;
                continue;
            }

            let new_path = if !use_full_path && !dir.is_empty() {
                format!("{dir}/{new_name_part}")
            } else {
                new_name_part.clone()
            };

            let target_path = Path::new(&new_path);

            if target_path.exists() || target_path.symlink_metadata().is_ok() {
                if args.no_overwrite {
                    if args.verbose {
                        eprintln!("rename: {file}: not overwritten");
                    }
                    skipped += 1;
                    continue;
                }
                if args.interactive && !prompt_overwrite(&new_path) {
                    skipped += 1;
                    continue;
                }
            }

            if args.verbose || args.no_act {
                println!("`{file}' -> `{new_path}'");
            }

            if !args.no_act {
                if let Err(e) = std::fs::rename(file, &new_path) {
                    eprintln!("rename: {file}: rename failed: {e}");
                    failed += 1;
                    continue;
                }
                succeeded += 1;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_first() {
        assert_eq!(
            replace_string("foo.bar.baz", "bar", "qux", false, false),
            "foo.qux.baz"
        );
    }

    #[test]
    fn test_replace_all() {
        assert_eq!(
            replace_string("foo.bar.bar", "bar", "qux", true, false),
            "foo.qux.qux"
        );
    }

    #[test]
    fn test_replace_last() {
        assert_eq!(
            replace_string("foo.bar.bar", "bar", "qux", false, true),
            "foo.bar.qux"
        );
    }

    #[test]
    fn test_replace_no_match() {
        assert_eq!(
            replace_string("foobar", "xyz", "qux", false, false),
            "foobar"
        );
    }

    #[test]
    fn test_replace_empty_expr() {
        assert_eq!(replace_string("abc", "", "X", false, false), "Xabc");
    }

    #[test]
    fn test_replace_empty_expr_last() {
        assert_eq!(replace_string("abc", "", "X", false, true), "abcX");
    }

    #[test]
    fn test_replace_empty_expr_all() {
        assert_eq!(replace_string("ab", "", "X", true, false), "XaXbX");
    }
}
