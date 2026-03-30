use clap::{CommandFactory, FromArgMatches};
use std::{
    collections::BTreeSet,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Debug, Default)]
pub struct Args {
    pub search_binaries: bool,
    pub search_manuals: bool,
    pub search_sources: bool,
    pub unusual_only: bool,
    pub list_paths: bool,
    pub glob_mode: bool,
    pub custom_bin_dirs: Vec<PathBuf>,
    pub custom_man_dirs: Vec<PathBuf>,
    pub custom_src_dirs: Vec<PathBuf>,
    pub names: Vec<String>,
}

#[derive(clap::Parser)]
#[command(
    name = "whereis",
    about = "Locate the binary, source, and manual page files for a command"
)]
struct ClapArgs {
    #[arg(short = 'b', help = "Search for binaries only")]
    binaries: bool,
    #[arg(short = 'm', help = "Search for manuals only")]
    manuals: bool,
    #[arg(short = 's', help = "Search for sources only")]
    sources: bool,
    #[arg(short = 'u', help = "Only show unusual entries")]
    unusual: bool,
    #[arg(short = 'l', help = "List effective lookup paths")]
    list: bool,
    #[arg(short = 'g', help = "Interpret names as glob patterns")]
    glob: bool,
}

impl Args {
    pub fn parse_from(raw: &[String]) -> Self {
        let mut args = Args::default();

        let mut i = 0;
        while i < raw.len() {
            match raw[i].as_str() {
                "-b" => args.search_binaries = true,
                "-m" => args.search_manuals = true,
                "-s" => args.search_sources = true,
                "-u" => args.unusual_only = true,
                "-l" => args.list_paths = true,
                "-g" => args.glob_mode = true,
                "-B" => {
                    i += 1;
                    while i < raw.len() && raw[i] != "-f" {
                        args.custom_bin_dirs.push(PathBuf::from(&raw[i]));
                        i += 1;
                    }
                    if i >= raw.len() || raw[i] != "-f" {
                        eprintln!(
                            "whereis: -B requires -f to terminate the directory list"
                        );
                        std::process::exit(1);
                    }
                }
                "-M" => {
                    i += 1;
                    while i < raw.len() && raw[i] != "-f" {
                        args.custom_man_dirs.push(PathBuf::from(&raw[i]));
                        i += 1;
                    }
                    if i >= raw.len() || raw[i] != "-f" {
                        eprintln!(
                            "whereis: -M requires -f to terminate the directory list"
                        );
                        std::process::exit(1);
                    }
                }
                "-S" => {
                    i += 1;
                    while i < raw.len() && raw[i] != "-f" {
                        args.custom_src_dirs.push(PathBuf::from(&raw[i]));
                        i += 1;
                    }
                    if i >= raw.len() || raw[i] != "-f" {
                        eprintln!(
                            "whereis: -S requires -f to terminate the directory list"
                        );
                        std::process::exit(1);
                    }
                }
                "-h" | "--help" => {
                    let _ = ClapArgs::command().print_help();
                    println!();
                    std::process::exit(0);
                }
                "-V" | "--version" => {
                    let cmd = ClapArgs::command();
                    println!(
                        "{} {}",
                        cmd.get_name(),
                        cmd.get_version().unwrap_or("")
                    );
                    std::process::exit(0);
                }
                other if other.starts_with('-') && other.len() > 1 => {
                    let chars: Vec<char> = other[1..].chars().collect();
                    let mut j = 0;
                    while j < chars.len() {
                        match chars[j] {
                            'b' => args.search_binaries = true,
                            'm' => args.search_manuals = true,
                            's' => args.search_sources = true,
                            'u' => args.unusual_only = true,
                            'l' => args.list_paths = true,
                            'g' => args.glob_mode = true,
                            c => {
                                eprintln!("whereis: invalid option -- '{c}'");
                                std::process::exit(1);
                            }
                        }
                        j += 1;
                    }
                }
                name => {
                    args.names.push(name.to_string());
                }
            }
            i += 1;
        }

        args
    }

    pub fn command() -> clap::Command {
        ClapArgs::command()
    }

    pub fn from_arg_matches(m: &clap::ArgMatches) -> Result<Self, clap::Error> {
        let clap_args = ClapArgs::from_arg_matches(m)?;
        Ok(Args {
            search_binaries: clap_args.binaries,
            search_manuals: clap_args.manuals,
            search_sources: clap_args.sources,
            unusual_only: clap_args.unusual,
            list_paths: clap_args.list,
            glob_mode: clap_args.glob,
            custom_bin_dirs: Vec::new(),
            custom_man_dirs: Vec::new(),
            custom_src_dirs: Vec::new(),
            names: Vec::new(),
        })
    }
}

fn default_bin_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = [
        "/usr/bin",
        "/usr/sbin",
        "/bin",
        "/sbin",
        "/usr/local/bin",
        "/usr/local/sbin",
        "/usr/games",
        "/usr/local/games",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();

    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(':') {
            if !p.is_empty() {
                let pb = PathBuf::from(p);
                if !dirs.contains(&pb) {
                    dirs.push(pb);
                }
            }
        }
    }

    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

fn default_man_dirs() -> Vec<PathBuf> {
    let base_patterns = [
        "/usr/share/man/man",
        "/usr/local/share/man/man",
        "/usr/local/man/man",
    ];
    let sections = ["1", "2", "3", "4", "5", "6", "7", "8", "1p", "3p"];

    let mut dirs = Vec::new();
    for base in &base_patterns {
        for section in &sections {
            let dir = PathBuf::from(format!("{base}{section}"));
            if dir.is_dir() {
                dirs.push(dir);
            }
        }
    }

    if let Ok(manpath) = std::env::var("MANPATH") {
        for p in manpath.split(':') {
            if !p.is_empty() {
                let base = PathBuf::from(p);
                if base.is_dir() {
                    for section in &sections {
                        let dir = base.join(format!("man{section}"));
                        if dir.is_dir() && !dirs.contains(&dir) {
                            dirs.push(dir);
                        }
                    }
                }
            }
        }
    }

    dirs
}

fn default_src_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for base in &["/usr/src", "/usr/local/src"] {
        let base_path = Path::new(base);
        if base_path.is_dir()
            && let Ok(entries) = fs::read_dir(base_path)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                }
            }
        }
    }
    dirs
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0 && m.is_file())
        .unwrap_or(false)
}

fn strip_name(name: &str) -> &str {
    let name = name.rsplit('/').next().unwrap_or(name);
    name.strip_prefix("s.").unwrap_or(name)
}

fn matches_name(filename: &str, name: &str, glob_mode: bool) -> bool {
    if glob_mode {
        glob_match(name, filename)
    } else {
        filename == name
    }
}

fn matches_name_with_ext(filename: &str, name: &str, glob_mode: bool) -> bool {
    if glob_mode {
        let base = filename.split('.').next().unwrap_or(filename);
        glob_match(name, base)
    } else {
        filename == name || filename.starts_with(&format!("{name}."))
    }
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_inner(&pat, &txt, 0, 0)
}

fn glob_match_inner(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }
    if pat[pi] == '*' {
        for t in ti..=txt.len() {
            if glob_match_inner(pat, txt, pi + 1, t) {
                return true;
            }
        }
        return false;
    }
    if pat[pi] == '?' {
        if ti < txt.len() {
            return glob_match_inner(pat, txt, pi + 1, ti + 1);
        }
        return false;
    }
    if ti < txt.len() && pat[pi] == txt[ti] {
        return glob_match_inner(pat, txt, pi + 1, ti + 1);
    }
    false
}

fn find_binaries(
    name: &str,
    dirs: &[PathBuf],
    glob_mode: bool,
) -> Vec<PathBuf> {
    let mut results = BTreeSet::new();
    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(fname) = path.file_name().and_then(|f| f.to_str())
                    && matches_name(fname, name, glob_mode)
                    && is_executable(&path)
                {
                    results.insert(path);
                }
            }
        }
    }
    results.into_iter().collect()
}

fn find_manuals(name: &str, dirs: &[PathBuf], glob_mode: bool) -> Vec<PathBuf> {
    find_files_with_ext(name, dirs, glob_mode)
}

fn find_sources(name: &str, dirs: &[PathBuf], glob_mode: bool) -> Vec<PathBuf> {
    find_files_with_ext(name, dirs, glob_mode)
}

fn find_files_with_ext(
    name: &str,
    dirs: &[PathBuf],
    glob_mode: bool,
) -> Vec<PathBuf> {
    let mut results = BTreeSet::new();
    for dir in dirs {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(fname) = path.file_name().and_then(|f| f.to_str())
                    && matches_name_with_ext(fname, name, glob_mode)
                    && path.is_file()
                {
                    results.insert(path);
                }
            }
        }
    }
    results.into_iter().collect()
}

pub fn run(args: Args) -> ExitCode {
    let search_all =
        !args.search_binaries && !args.search_manuals && !args.search_sources;
    let do_bins = search_all || args.search_binaries;
    let do_mans = search_all || args.search_manuals;
    let do_srcs = search_all || args.search_sources;

    let bin_dirs = if !args.custom_bin_dirs.is_empty() {
        args.custom_bin_dirs.clone()
    } else {
        default_bin_dirs()
    };

    let man_dirs = if !args.custom_man_dirs.is_empty() {
        args.custom_man_dirs.clone()
    } else {
        default_man_dirs()
    };

    let src_dirs = if !args.custom_src_dirs.is_empty() {
        args.custom_src_dirs.clone()
    } else {
        default_src_dirs()
    };

    if args.list_paths {
        if do_bins {
            println!("bin: {}", format_dir_list(&bin_dirs));
        }
        if do_mans {
            println!("man: {}", format_dir_list(&man_dirs));
        }
        if do_srcs {
            println!("src: {}", format_dir_list(&src_dirs));
        }
        return ExitCode::SUCCESS;
    }

    if args.names.is_empty() {
        eprintln!("whereis: no name specified");
        return ExitCode::FAILURE;
    }

    for name in &args.names {
        let name = strip_name(name);
        let mut paths: Vec<PathBuf> = Vec::new();

        let bin_results = if do_bins {
            find_binaries(name, &bin_dirs, args.glob_mode)
        } else {
            Vec::new()
        };
        let man_results = if do_mans {
            find_manuals(name, &man_dirs, args.glob_mode)
        } else {
            Vec::new()
        };
        let src_results = if do_srcs {
            find_sources(name, &src_dirs, args.glob_mode)
        } else {
            Vec::new()
        };

        if args.unusual_only {
            let mut expected_types = 0;
            let mut found_single = 0;
            if do_bins {
                expected_types += 1;
                if bin_results.len() == 1 {
                    found_single += 1;
                }
            }
            if do_mans {
                expected_types += 1;
                if man_results.len() == 1 {
                    found_single += 1;
                }
            }
            if do_srcs {
                expected_types += 1;
                if src_results.len() == 1 {
                    found_single += 1;
                }
            }
            if found_single == expected_types {
                continue;
            }
        }

        paths.extend(bin_results);
        paths.extend(man_results);
        paths.extend(src_results);

        let path_strs: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if path_strs.is_empty() {
            println!("{name}:");
        } else {
            println!("{name}: {}", path_strs.join(" "));
        }
    }

    ExitCode::SUCCESS
}

fn format_dir_list(dirs: &[PathBuf]) -> String {
    dirs.iter()
        .map(|d| d.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_name_simple() {
        assert_eq!(strip_name("ls"), "ls");
    }

    #[test]
    fn test_strip_name_with_path() {
        assert_eq!(strip_name("/usr/bin/ls"), "ls");
    }

    #[test]
    fn test_strip_name_sccs_prefix() {
        assert_eq!(strip_name("s.main.c"), "main.c");
    }

    #[test]
    fn test_glob_match_exact() {
        assert!(glob_match("ls", "ls"));
        assert!(!glob_match("ls", "lsblk"));
    }

    #[test]
    fn test_glob_match_star() {
        assert!(glob_match("ls*", "ls"));
        assert!(glob_match("ls*", "lsblk"));
        assert!(!glob_match("ls*", "cat"));
    }

    #[test]
    fn test_glob_match_question() {
        assert!(glob_match("l?", "ls"));
        assert!(!glob_match("l?", "lsblk"));
    }

    #[test]
    fn test_matches_name_with_ext_exact() {
        assert!(matches_name_with_ext("ls.1", "ls", false));
        assert!(matches_name_with_ext("ls.1.gz", "ls", false));
        assert!(!matches_name_with_ext("lsblk.1", "ls", false));
    }

    #[test]
    fn test_default_bin_dirs_not_empty() {
        let dirs = default_bin_dirs();
        assert!(!dirs.is_empty());
    }
}
