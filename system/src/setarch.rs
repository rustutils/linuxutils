//! Run a program with a different reported architecture or
//! personality flags, or both.
//!
//! `setarch <arch> [options] <program> [args]` calls `personality(2)`
//! with the computed value, then `execvp`s `<program>`. The kernel
//! preserves the personality across exec, so the new program sees
//! a `uname()` machine name and runtime quirks consistent with
//! `<arch>`.
//!
//! v0 covers the cross-host arch names (`linux32`, `linux64`) plus
//! the host-architecture-conditional ones used by upstream
//! `setarch`. The `argv[0]`-symlink form (`linux32 program`) is
//! deferred. For v0 use the explicit positional.

use clap::{ArgAction, Parser};
use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};
use std::{ffi::CString, process::ExitCode};

pub const MAN: ManContent = ManContent::empty().section(8);

// `<sys/personality.h>` is not in libc, so it is defined here as the
// kernel
// uapi values.
const PER_LINUX: libc::c_ulong = 0x0000;
const PER_LINUX32: libc::c_ulong = 0x0008;
const PER_MASK: u32 = 0x00ff;

/// Pseudo-value `personality(2)` accepts to *read* the current
/// personality without changing it.
const PERSONALITY_QUERY: libc::c_ulong = 0xffffffff;

/// Table mapping architecture names to base personalities. Upstream
/// `setarch.c` filters this list per host so unsupported names are
/// rejected at parse time. Our v0 keeps the full list and lets the
/// kernel reject the call if the personality doesn't make sense on
/// the running host.
const ARCH_TABLE: &[(&str, libc::c_ulong)] = &[
    // Cross-host
    ("linux32", PER_LINUX32),
    ("linux64", PER_LINUX),
    // x86 family
    ("i386", PER_LINUX32),
    ("i486", PER_LINUX32),
    ("i586", PER_LINUX32),
    ("i686", PER_LINUX32),
    ("athlon", PER_LINUX32),
    ("x86_64", PER_LINUX),
    ("amd64", PER_LINUX),
    // ARM
    ("arm", PER_LINUX32),
    ("armv7l", PER_LINUX32),
    ("armv8l", PER_LINUX32),
    ("aarch64", PER_LINUX),
    ("arm64", PER_LINUX),
    // PowerPC
    ("ppc", PER_LINUX32),
    ("ppc32", PER_LINUX32),
    ("ppc64", PER_LINUX),
    ("ppc64le", PER_LINUX),
    // s390
    ("s390", PER_LINUX32),
    ("s390x", PER_LINUX),
    // SPARC
    ("sparc", PER_LINUX32),
    ("sparc32", PER_LINUX32),
    ("sparc64", PER_LINUX),
    // MIPS
    ("mips", PER_LINUX32),
    ("mips64", PER_LINUX),
];

/// Personality flag bits, ordered ascending so verbose / `--show`
/// output matches upstream.
const FLAG_TABLE: &[(u32, &str)] = &[
    (0x0020000, "UNAME26"),
    (0x0040000, "ADDR_NO_RANDOMIZE"),
    (0x0080000, "FDPIC_FUNCPTRS"),
    (0x0100000, "MMAP_PAGE_ZERO"),
    (0x0200000, "ADDR_COMPAT_LAYOUT"),
    (0x0400000, "READ_IMPLIES_EXEC"),
    (0x0800000, "ADDR_LIMIT_32BIT"),
    (0x1000000, "SHORT_INODE"),
    (0x2000000, "WHOLE_SECONDS"),
    (0x4000000, "STICKY_TIMEOUTS"),
    (0x8000000, "ADDR_LIMIT_3GB"),
];

/// Named personalities, where the entire value matches.
const NAMED_TABLE: &[(u32, &str)] = &[
    (0x00000000, "PER_LINUX"),
    (0x00800000, "PER_LINUX_32BIT"),
    (0x00080000, "PER_LINUX_FDPIC"),
    (0x04100001, "PER_SVR4"),
    (0x05000002, "PER_SVR3"),
    (0x07000003, "PER_SCOSVR3"),
    (0x06000003, "PER_OSR5"),
    (0x05000004, "PER_WYSEV386"),
    (0x04000005, "PER_ISCR4"),
    (0x00000006, "PER_BSD"),
    (0x04000006, "PER_SUNOS"),
    (0x05000007, "PER_XENIX"),
    (0x00000008, "PER_LINUX32"),
    (0x08000008, "PER_LINUX32_3GB"),
    (0x00000009, "PER_RISCOS"),
    (0x0400000a, "PER_SOLARIS"),
    (0x0410000b, "PER_UW7"),
    (0x0000000c, "PER_OSF4"),
    (0x0000000d, "PER_HPUX"),
];

/// Base-personality (low byte) names, used when the full value
/// doesn't match `NAMED_TABLE`.
const BASE_TABLE: &[(u32, &str)] = &[
    (0x0, "PER_LINUX"),
    (0x1, "PER_SVR4"),
    (0x2, "PER_SVR3"),
    (0x3, "PER_SCOSVR3"),
    (0x4, "PER_WYSEV386"),
    (0x5, "PER_ISCR4"),
    (0x6, "PER_BSD"),
    (0x7, "PER_XENIX"),
    (0x8, "PER_LINUX32"),
    (0x9, "PER_RISCOS"),
    (0xa, "PER_SOLARIS"),
    (0xb, "PER_UW7"),
    (0xc, "PER_OSF4"),
    (0xd, "PER_HPUX"),
];

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "setarch",
    version,
    about = "Run a program with altered architecture or personality flags"
)]
pub struct Args {
    /// List the recognised architecture names and exit.
    #[arg(long, action = ArgAction::SetTrue)]
    pub list: bool,

    /// Show personality flags and exit. Without a value, shows the
    /// personality of the current process, or of the one given with
    /// --pid. With a value, decodes that value instead.
    #[arg(long, value_name = "PERSONALITY", num_args = 0..=1, default_missing_value = "")]
    pub show: Option<String>,

    /// Read the personality of process PID from /proc/PID/personality.
    /// Only works together with --show.
    #[arg(short = 'p', long = "pid", value_name = "PID")]
    pub pid: Option<i32>,

    /// Print what we're doing
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Disable address-space randomisation (ADDR_NO_RANDOMIZE)
    #[arg(short = 'R', long = "addr-no-randomize", action = ArgAction::SetTrue)]
    pub addr_no_randomize: bool,

    /// Turn on FDPIC_FUNCPTRS (function pointer-style returns)
    #[arg(short = 'F', long = "fdpic-funcptrs", action = ArgAction::SetTrue)]
    pub fdpic_funcptrs: bool,

    /// Turn on MMAP_PAGE_ZERO (map page 0 read-only)
    #[arg(short = 'Z', long = "mmap-page-zero", action = ArgAction::SetTrue)]
    pub mmap_page_zero: bool,

    /// Use the legacy bottom-up address layout (ADDR_COMPAT_LAYOUT)
    #[arg(short = 'L', long = "addr-compat-layout", action = ArgAction::SetTrue)]
    pub addr_compat_layout: bool,

    /// Make readable pages executable too (READ_IMPLIES_EXEC)
    #[arg(short = 'X', long = "read-implies-exec", action = ArgAction::SetTrue)]
    pub read_implies_exec: bool,

    /// Cap the address space to 32 bits (ADDR_LIMIT_32BIT)
    #[arg(short = 'B', long = "32bit", action = ArgAction::SetTrue)]
    pub bit32: bool,

    /// Return only 16-bit inode numbers from stat() (SHORT_INODE)
    #[arg(short = 'I', long = "short-inode", action = ArgAction::SetTrue)]
    pub short_inode: bool,

    /// Round timeouts up to whole seconds (WHOLE_SECONDS)
    #[arg(short = 'S', long = "whole-seconds", action = ArgAction::SetTrue)]
    pub whole_seconds: bool,

    /// Don't reduce system call timeouts when interrupted with EINTR
    /// (STICKY_TIMEOUTS)
    #[arg(short = 'T', long = "sticky-timeouts", action = ArgAction::SetTrue)]
    pub sticky_timeouts: bool,

    /// Cap the address space to 3GB (ADDR_LIMIT_3GB)
    #[arg(short = '3', long = "3gb", action = ArgAction::SetTrue)]
    pub limit_3gb: bool,

    /// Remove the 3GB cap, which gives the default 4GB layout
    #[arg(long = "4gb", action = ArgAction::SetTrue)]
    pub limit_4gb: bool,

    /// Make uname() report kernel 2.6.x (UNAME26)
    #[arg(long = "uname-2.6", action = ArgAction::SetTrue)]
    pub uname_26: bool,

    /// Architecture name, such as linux32, linux64 or i686
    pub arch: Option<String>,

    /// Program to run, with its arguments. Required unless --list or
    /// --show is given.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    if args.list {
        list_archs();
        return ExitCode::SUCCESS;
    }

    if args.pid.is_some() && args.show.is_none() {
        eprintln!("setarch: use -p/--pid option with --show option");
        return ExitCode::from(1);
    }

    if let Some(name) = args.show.as_deref() {
        return show_personality(name, args.pid);
    }

    let Some(arch) = args.arch.as_deref() else {
        eprintln!("setarch: missing architecture argument (try --list)");
        return ExitCode::from(2);
    };

    let mut persona = match resolve_arch(arch) {
        Some(p) => p,
        None => {
            eprintln!("setarch: {arch}: Unrecognized architecture");
            return ExitCode::from(1);
        }
    };
    persona = apply_flags(persona, &args);

    if args.command.is_empty() {
        eprintln!("setarch: a program to run is required");
        return ExitCode::from(2);
    }

    if args.verbose {
        verbose_personality(persona);
    }

    let rc = unsafe { libc::personality(persona) };
    if rc < 0 {
        eprintln!(
            "setarch: personality(0x{persona:08x}): {}",
            std::io::Error::last_os_error()
        );
        return ExitCode::FAILURE;
    }

    if args.verbose {
        println!("Execute command `{}'.", &args.command[0]);
    }

    exec_program(&args.command[0], &args.command[1..])
}

fn resolve_arch(name: &str) -> Option<libc::c_ulong> {
    ARCH_TABLE.iter().find(|(n, _)| *n == name).map(|(_, p)| *p)
}

fn apply_flags(mut p: libc::c_ulong, args: &Args) -> libc::c_ulong {
    if args.bit32 {
        p |= libc::ADDR_LIMIT_32BIT as libc::c_ulong;
    }
    if args.addr_no_randomize {
        p |= libc::ADDR_NO_RANDOMIZE as libc::c_ulong;
    }
    if args.fdpic_funcptrs {
        p |= 0x0080000; // FDPIC_FUNCPTRS
    }
    if args.mmap_page_zero {
        p |= libc::MMAP_PAGE_ZERO as libc::c_ulong;
    }
    if args.addr_compat_layout {
        p |= libc::ADDR_COMPAT_LAYOUT as libc::c_ulong;
    }
    if args.read_implies_exec {
        p |= libc::READ_IMPLIES_EXEC as libc::c_ulong;
    }
    if args.short_inode {
        p |= libc::SHORT_INODE as libc::c_ulong;
    }
    if args.whole_seconds {
        p |= libc::WHOLE_SECONDS as libc::c_ulong;
    }
    if args.sticky_timeouts {
        p |= libc::STICKY_TIMEOUTS as libc::c_ulong;
    }
    if args.limit_3gb {
        p |= libc::ADDR_LIMIT_3GB as libc::c_ulong;
    }
    if args.limit_4gb {
        p &= !(libc::ADDR_LIMIT_3GB as libc::c_ulong);
    }
    if args.uname_26 {
        p |= 0x0020000; // UNAME26
    }
    p
}

fn list_archs() {
    for (name, _) in ARCH_TABLE {
        println!("{name}");
    }
}

fn verbose_personality(pers: libc::c_ulong) {
    let p = pers as u32;
    for (bit, name) in FLAG_TABLE {
        if p & bit != 0 {
            println!("Switching on {name}.");
        }
    }
}

/// Parse the `--show=VALUE` argument or the contents of
/// `/proc/PID/personality`. Both are hex strings, optionally with a
/// `0x` prefix.
fn parse_personality(s: &str) -> Option<u32> {
    let trimmed = s.trim();
    let (s, radix) = if let Some(stripped) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        (stripped, 16)
    } else {
        (trimmed, 16)
    };
    u32::from_str_radix(s, radix).ok()
}

fn show_personality(arg: &str, pid: Option<i32>) -> ExitCode {
    let value: u32 = if !arg.is_empty() {
        match parse_personality(arg) {
            Some(v) => v,
            None => {
                eprintln!("setarch: invalid personality: {arg}");
                return ExitCode::from(1);
            }
        }
    } else if let Some(pid) = pid {
        let path = format!("/proc/{pid}/personality");
        match std::fs::read_to_string(&path) {
            Ok(s) => match parse_personality(&s) {
                Some(v) => v,
                None => {
                    eprintln!("setarch: cannot parse {path}: {s:?}");
                    return ExitCode::from(1);
                }
            },
            Err(e) => {
                eprintln!("setarch: cannot read {path}: {e}");
                return ExitCode::from(1);
            }
        }
    } else {
        let cur = unsafe { libc::personality(PERSONALITY_QUERY) };
        if cur < 0 {
            eprintln!(
                "setarch: personality(read): {}",
                std::io::Error::last_os_error()
            );
            return ExitCode::FAILURE;
        }
        cur as u32
    };
    println!("{}", format_personality(value));
    ExitCode::SUCCESS
}

fn format_personality(pers: u32) -> String {
    if let Some((_, name)) = NAMED_TABLE.iter().find(|(v, _)| *v == pers) {
        return (*name).to_string();
    }

    let base = pers & PER_MASK;
    let mut flags = pers & !PER_MASK;

    let base_str = BASE_TABLE
        .iter()
        .find(|(v, _)| *v == base)
        .map(|(_, n)| (*n).to_string())
        .unwrap_or_else(|| format!("0x{base:x}"));

    if flags == 0 {
        return base_str;
    }

    let mut out = format!("{base_str} (");
    let mut first = true;
    for (bit, name) in FLAG_TABLE {
        if flags & bit != 0 {
            if !first {
                out.push(' ');
            }
            out.push_str(name);
            first = false;
            flags &= !bit;
        }
    }
    if flags != 0 {
        if !first {
            out.push(' ');
        }
        out.push_str(&format!("0x{flags:08x}"));
    }
    out.push(')');
    out
}

fn exec_program(program: &str, arguments: &[String]) -> ExitCode {
    let c_program = match CString::new(program.as_bytes()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("setarch: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut c_args: Vec<CString> = vec![c_program.clone()];
    for arg in arguments {
        match CString::new(arg.as_bytes()) {
            Ok(s) => c_args.push(s),
            Err(e) => {
                eprintln!("setarch: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    let c_ptrs: Vec<*const libc::c_char> = c_args
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect();

    unsafe {
        libc::execvp(c_program.as_ptr(), c_ptrs.as_ptr());
    }
    let err = std::io::Error::last_os_error();
    eprintln!("setarch: failed to execute {program}: {}", strerror(&err));
    ExitCode::from(127)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_table_includes_cross_host_names() {
        assert_eq!(resolve_arch("linux32"), Some(PER_LINUX32));
        assert_eq!(resolve_arch("linux64"), Some(PER_LINUX));
    }

    #[test]
    fn unknown_arch_returns_none() {
        assert_eq!(resolve_arch("definitely-not-an-arch"), None);
    }

    #[test]
    fn flags_or_in_correctly() {
        let mut a = Args {
            list: false,
            show: None,
            pid: None,
            verbose: false,
            addr_no_randomize: true,
            fdpic_funcptrs: false,
            mmap_page_zero: false,
            addr_compat_layout: false,
            read_implies_exec: false,
            bit32: true,
            short_inode: false,
            whole_seconds: false,
            sticky_timeouts: false,
            limit_3gb: false,
            limit_4gb: false,
            uname_26: false,
            arch: None,
            command: vec![],
        };
        let p = apply_flags(PER_LINUX, &a);
        assert!(p & (libc::ADDR_NO_RANDOMIZE as libc::c_ulong) != 0);
        assert!(p & (libc::ADDR_LIMIT_32BIT as libc::c_ulong) != 0);

        a.limit_3gb = true;
        let p = apply_flags(PER_LINUX, &a);
        assert!(p & (libc::ADDR_LIMIT_3GB as libc::c_ulong) != 0);

        a.limit_4gb = true;
        let p = apply_flags(PER_LINUX, &a);
        // 4gb cancels 3gb
        assert!(p & (libc::ADDR_LIMIT_3GB as libc::c_ulong) == 0);
    }

    #[test]
    fn show_named_personalities() {
        assert_eq!(format_personality(0x00000000), "PER_LINUX");
        assert_eq!(format_personality(0x00800000), "PER_LINUX_32BIT");
        assert_eq!(format_personality(0x00000008), "PER_LINUX32");
        assert_eq!(format_personality(0x08000008), "PER_LINUX32_3GB");
    }

    #[test]
    fn show_base_with_flags() {
        assert_eq!(
            format_personality(0x00880000),
            "PER_LINUX (FDPIC_FUNCPTRS ADDR_LIMIT_32BIT)"
        );
    }

    #[test]
    fn show_unknown_base_and_unknown_flag() {
        assert_eq!(format_personality(0x000040ff), "0xff (0x00004000)");
    }

    #[test]
    fn parses_personality_with_and_without_prefix() {
        assert_eq!(parse_personality("0x00000000"), Some(0));
        assert_eq!(parse_personality("0x08000008"), Some(0x08000008));
        assert_eq!(parse_personality("00000000"), Some(0));
        assert_eq!(parse_personality("08000008"), Some(0x08000008));
    }
}
