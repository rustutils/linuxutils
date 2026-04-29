use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs,
    io::{self, Write},
    process::ExitCode,
};

const DEFAULT_DICT: &str = "/usr/share/dict/words";
const ALT_DICT: &str = "/usr/share/dict/web2";

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "look",
    version,
    about = "Display lines beginning with a given string"
)]
pub struct Args {
    /// Use the alternative dictionary file
    #[arg(short = 'a', long = "alternative")]
    alternative: bool,

    /// Use normal dictionary character set and order (only blanks and
    /// alphanumeric characters are compared)
    #[arg(short = 'd', long = "alphanum")]
    alphanum: bool,

    /// Ignore the case of alphabetic characters
    #[arg(short = 'f', long = "ignore-case")]
    ignore_case: bool,

    /// String termination character
    #[arg(short = 't', long = "terminate")]
    terminate: Option<char>,

    /// String to search for
    pub string: String,

    /// File to search (default: /usr/share/dict/words)
    pub file: Option<String>,
}

pub fn run(args: Args) -> ExitCode {
    let using_dict = args.file.is_none();
    // When using the default dictionary, -d and -f are on by default.
    let alphanum = args.alphanum || using_dict;
    let ignore_case = args.ignore_case || using_dict;

    let path = if let Some(ref f) = args.file {
        f.clone()
    } else if args.alternative {
        ALT_DICT.to_string()
    } else if let Ok(w) = std::env::var("WORDLIST") {
        w
    } else {
        DEFAULT_DICT.to_string()
    };

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("look: {path}: {e}");
            return ExitCode::from(2);
        }
    };

    let search =
        prepare_key(&args.string, args.terminate, alphanum, ignore_case);

    let lines: Vec<&str> = content.lines().collect();

    // Binary search to find the first matching line.
    let start = lines.partition_point(|line| {
        let key = line_key(line, search.len(), alphanum, ignore_case);
        key < search
    });

    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let mut found = false;

    for line in &lines[start..] {
        let key = line_key(line, search.len(), alphanum, ignore_case);
        if key != search {
            break;
        }
        found = true;
        if let Err(e) = writeln!(out, "{line}") {
            eprintln!("look: {e}");
            return ExitCode::from(2);
        }
    }

    if found {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Prepare the search key: apply terminator, then normalize.
fn prepare_key(
    s: &str,
    terminate: Option<char>,
    alphanum: bool,
    ignore_case: bool,
) -> String {
    let s = match terminate {
        Some(t) => match s.find(t) {
            Some(pos) => &s[..pos + t.len_utf8()],
            None => s,
        },
        None => s,
    };
    normalize(s, alphanum, ignore_case)
}

/// Extract a comparable key from the beginning of a line.
/// The key length is bounded by the search key length (in normalized chars).
fn line_key(
    line: &str,
    key_len: usize,
    alphanum: bool,
    ignore_case: bool,
) -> String {
    if alphanum {
        let mut result = String::with_capacity(key_len);
        for ch in line.chars() {
            if result.len() >= key_len {
                break;
            }
            if ch.is_alphanumeric() || ch == ' ' || ch == '\t' {
                if ignore_case {
                    for c in ch.to_lowercase() {
                        result.push(c);
                    }
                } else {
                    result.push(ch);
                }
            }
        }
        result
    } else {
        let prefix: String = line.chars().take(key_len).collect();
        if ignore_case {
            prefix.to_lowercase()
        } else {
            prefix
        }
    }
}

fn normalize(s: &str, alphanum: bool, ignore_case: bool) -> String {
    let filtered: String = if alphanum {
        s.chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '\t')
            .collect()
    } else {
        s.to_string()
    };
    if ignore_case {
        filtered.to_lowercase()
    } else {
        filtered
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_file(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    fn run_look(
        string: &str,
        content: &str,
        alphanum: bool,
        ignore_case: bool,
        terminate: Option<char>,
    ) -> (ExitCode, String) {
        let file = make_file(content);
        let args = Args {
            alternative: false,
            alphanum,
            ignore_case,
            terminate,
            string: string.to_string(),
            file: Some(file.path().to_string_lossy().to_string()),
        };

        // Capture stdout by running the core logic directly.
        let search =
            prepare_key(&args.string, args.terminate, alphanum, ignore_case);
        let content = fs::read_to_string(file.path()).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        let start = lines.partition_point(|line| {
            let key = line_key(line, search.len(), alphanum, ignore_case);
            key < search
        });

        let mut output = Vec::new();
        let mut found = false;
        for line in &lines[start..] {
            let key = line_key(line, search.len(), alphanum, ignore_case);
            if key != search {
                break;
            }
            found = true;
            writeln!(output, "{line}").unwrap();
        }

        let code = if found {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        };
        (code, String::from_utf8(output).unwrap())
    }

    #[test]
    fn basic_prefix_match() {
        let (_, out) =
            run_look("b", "apple\nbanana\nberry\ncherry", false, false, None);
        assert_eq!(out, "banana\nberry\n");
    }

    #[test]
    fn no_match_returns_1() {
        let (code, out) = run_look("zzz", "apple\nbanana", false, false, None);
        assert_eq!(out, "");
        assert_eq!(code, ExitCode::from(1));
    }

    #[test]
    fn case_insensitive() {
        let (_, out) = run_look("APPLE", "apple\nbanana", false, true, None);
        assert_eq!(out, "apple\n");
    }

    #[test]
    fn alphanum_mode() {
        let (_, out) =
            run_look("ban", "ban-ana\nbanana\nberry", true, false, None);
        assert_eq!(out, "ban-ana\nbanana\n");
    }

    #[test]
    fn terminate_char() {
        let (_, out) = run_look(
            "apple pie",
            "apple\napple pie\nbanana",
            false,
            false,
            Some(' '),
        );
        // -t ' ' means only compare up to and including the first space: "apple "
        // "apple" is shorter than the key so it doesn't match.
        assert_eq!(out, "apple pie\n");
    }
}
