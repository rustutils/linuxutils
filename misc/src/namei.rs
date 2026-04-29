use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use rustix::fs::{FileType, RawMode, Stat};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "namei",
    about = "Follow a pathname until a terminal point is found",
    override_usage = "namei [options] <pathname>..."
)]
pub struct Args {
    /// Long listing (equivalent to -m -o -v)
    #[arg(short = 'l', long)]
    long: bool,

    /// Show the mode bits of each file type
    #[arg(short = 'm', long)]
    modes: bool,

    /// Don't follow symlinks
    #[arg(short = 'n', long)]
    nosymlinks: bool,

    /// Show owner and group name
    #[arg(short = 'o', long)]
    owners: bool,

    /// Vertically align modes and owners
    #[arg(short = 'v', long)]
    vertical: bool,

    /// Show mountpoints with 'D' instead of 'd'
    #[arg(short = 'x', long)]
    mountpoints: bool,

    /// Pathnames to follow
    #[arg(required = true)]
    pathnames: Vec<String>,
}

struct Options {
    modes: bool,
    owners: bool,
    vertical: bool,
    nosymlinks: bool,
    mountpoints: bool,
}

fn format_mode(mode: RawMode) -> String {
    let mut s = String::with_capacity(9);
    let perms = [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ];
    for (bit, ch) in perms {
        if mode & bit != 0 {
            s.push(ch);
        } else {
            s.push('-');
        }
    }
    s
}

fn uid_to_name(uid: u32) -> String {
    fs::read_to_string("/etc/passwd")
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() >= 3
                    && let Ok(u) = fields[2].parse::<u32>()
                    && u == uid
                {
                    return Some(fields[0].to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| uid.to_string())
}

fn gid_to_name(gid: u32) -> String {
    fs::read_to_string("/etc/group")
        .ok()
        .and_then(|content| {
            for line in content.lines() {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() >= 3
                    && let Ok(g) = fields[2].parse::<u32>()
                    && g == gid
                {
                    return Some(fields[0].to_string());
                }
            }
            None
        })
        .unwrap_or_else(|| gid.to_string())
}

fn file_type_char(stat: &Stat, is_mountpoint: bool, opts: &Options) -> char {
    let ft = FileType::from_raw_mode(stat.st_mode);
    if ft == FileType::Directory {
        if opts.mountpoints && is_mountpoint {
            'D'
        } else {
            'd'
        }
    } else if ft == FileType::Symlink {
        'l'
    } else if ft == FileType::RegularFile {
        '-'
    } else if ft == FileType::Socket {
        's'
    } else if ft == FileType::BlockDevice {
        'b'
    } else if ft == FileType::CharacterDevice {
        'c'
    } else if ft == FileType::Fifo {
        'p'
    } else {
        '?'
    }
}

fn stat_path(path: &Path) -> io::Result<Stat> {
    use rustix::fs::{AtFlags, CWD};
    rustix::fs::statat(CWD, path, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(io::Error::from)
}

fn is_mountpoint(path: &Path, child_dev: u64) -> bool {
    let parent = match path.parent() {
        Some(p) if p == Path::new("") => Path::new("."),
        Some(p) => p,
        None => return true,
    };
    match stat_path(parent) {
        Ok(parent_stat) => parent_stat.st_dev != child_dev,
        Err(_) => false,
    }
}

fn print_entry(
    type_char: char,
    name: &str,
    stat: Option<&Stat>,
    link_target: Option<&str>,
    indent: usize,
    opts: &Options,
) {
    let indent_str = " ".repeat(indent * 2);

    if opts.modes || opts.owners {
        if let Some(st) = stat {
            if opts.vertical {
                let mode_str = if opts.modes {
                    format_mode(st.st_mode)
                } else {
                    String::new()
                };
                let owner_str = if opts.owners {
                    format!(
                        "{} {}",
                        uid_to_name(st.st_uid),
                        gid_to_name(st.st_gid)
                    )
                } else {
                    String::new()
                };
                if opts.modes && opts.owners {
                    print!("{indent_str}{mode_str} {owner_str} ");
                } else if opts.modes {
                    print!("{indent_str}{mode_str} ");
                } else {
                    print!("{indent_str}{owner_str} ");
                }
            } else {
                let mut parts = Vec::new();
                if opts.modes {
                    parts.push(format_mode(st.st_mode));
                }
                if opts.owners {
                    parts.push(format!(
                        "{} {}",
                        uid_to_name(st.st_uid),
                        gid_to_name(st.st_gid)
                    ));
                }
                print!("{indent_str}{} ", parts.join(" "));
            }
        } else {
            print!("{indent_str}");
        }
    } else {
        print!("{indent_str}");
    }

    print!("{type_char} {name}");
    if let Some(target) = link_target {
        print!(" -> {target}");
    }
    println!();
}

fn walk_path(
    pathname: &str,
    opts: &Options,
    visited: &mut HashSet<PathBuf>,
    indent: usize,
) {
    let components: Vec<&str> = pathname.split('/').collect();

    let mut current = PathBuf::new();
    let mut first = true;

    for component in &components {
        if first && component.is_empty() {
            // Absolute path: starts with "/"
            current.push("/");
            let stat_result = stat_path(&current);
            match &stat_result {
                Ok(st) => {
                    let mp =
                        opts.mountpoints && is_mountpoint(&current, st.st_dev);
                    let tc = file_type_char(st, mp, opts);
                    print_entry(tc, "/", Some(st), None, indent, opts);
                }
                Err(e) => {
                    print_entry('?', "/", None, None, indent, opts);
                    eprintln!("namei: failed to stat /: {e}");
                }
            }
            first = false;
            continue;
        }

        if component.is_empty() {
            first = false;
            continue;
        }

        current.push(component);
        first = false;

        let stat_result = stat_path(&current);
        match stat_result {
            Ok(st) => {
                let ft = FileType::from_raw_mode(st.st_mode);
                let mp = opts.mountpoints
                    && ft == FileType::Directory
                    && is_mountpoint(&current, st.st_dev);
                let tc = file_type_char(&st, mp, opts);

                if ft == FileType::Symlink {
                    match fs::read_link(&current) {
                        Ok(target) => {
                            let target_str =
                                target.to_string_lossy().to_string();
                            print_entry(
                                tc,
                                component,
                                Some(&st),
                                Some(&target_str),
                                indent,
                                opts,
                            );

                            if !opts.nosymlinks {
                                let resolved = if target.is_absolute() {
                                    target.clone()
                                } else {
                                    current
                                        .parent()
                                        .unwrap_or(Path::new("."))
                                        .join(&target)
                                };

                                if visited.contains(&resolved) {
                                    let warn_indent =
                                        " ".repeat((indent + 1) * 2);
                                    println!(
                                        "{warn_indent}  [loop detected at {}]",
                                        resolved.display()
                                    );
                                } else {
                                    visited.insert(resolved.clone());
                                    let walk_target =
                                        resolved.to_string_lossy().to_string();
                                    walk_path(
                                        &walk_target,
                                        opts,
                                        visited,
                                        indent + 1,
                                    );
                                }

                                // After resolving the symlink, update current to point
                                // through the resolved target for subsequent components
                                current = fs::canonicalize(
                                    current
                                        .parent()
                                        .unwrap_or(Path::new("."))
                                        .join(&target),
                                )
                                .unwrap_or(current);
                            }
                        }
                        Err(e) => {
                            print_entry(
                                tc,
                                component,
                                Some(&st),
                                None,
                                indent,
                                opts,
                            );
                            eprintln!(
                                "namei: failed to read link {}: {e}",
                                current.display()
                            );
                        }
                    }
                } else {
                    print_entry(tc, component, Some(&st), None, indent, opts);
                }
            }
            Err(e) => {
                print_entry('?', component, None, None, indent, opts);
                eprintln!("namei: failed to stat {}: {e}", current.display());
                return;
            }
        }
    }
}

pub fn run(args: Args) -> ExitCode {
    let opts = Options {
        modes: args.modes || args.long,
        owners: args.owners || args.long,
        vertical: args.vertical || args.long,
        nosymlinks: args.nosymlinks,
        mountpoints: args.mountpoints,
    };

    for pathname in &args.pathnames {
        println!("f: {pathname}");
        let mut visited = HashSet::new();
        walk_path(pathname, &opts, &mut visited, 0);
    }

    ExitCode::SUCCESS
}
