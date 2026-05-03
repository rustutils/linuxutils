//! Run a program with a different reported architecture and / or
//! personality flags.
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
//! deferred — for v0 use the explicit positional.

use clap::{ArgAction, Parser};
use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};
use std::{ffi::CString, process::ExitCode};

pub const MAN: ManContent = ManContent::empty();

// `<sys/personality.h>` — not in libc, defined here as the kernel
// uapi values.
const PER_LINUX: libc::c_ulong = 0x0000;
const PER_LINUX32: libc::c_ulong = 0x0008;

/// Pseudo-value `personality(2)` accepts to *read* the current
/// personality without changing it.
const PERSONALITY_QUERY: libc::c_ulong = 0xffffffff;

/// Architecture-name → base personality table. Upstream
/// `setarch.c` filters this list per host so unsupported names are
/// rejected at parse time; our v0 keeps the full list and lets the
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

    /// Show personality flag bits and exit. Without an argument,
    /// shows the running process's current personality; with a
    /// value, shows what the named personality would be.
    #[arg(long, value_name = "PERSONALITY", num_args = 0..=1, default_missing_value = "")]
    pub show: Option<String>,

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

    /// READ_IMPLIES_EXEC — pages with PROT_READ also get PROT_EXEC
    #[arg(short = 'X', long = "read-implies-exec", action = ArgAction::SetTrue)]
    pub read_implies_exec: bool,

    /// 32-bit base personality (PER_LINUX32) without picking an
    /// arch alias
    #[arg(short = 'B', long = "32bit", action = ArgAction::SetTrue)]
    pub bit32: bool,

    /// SHORT_INODE — return only 16-bit inode numbers from stat()
    #[arg(short = 'I', long = "short-inode", action = ArgAction::SetTrue)]
    pub short_inode: bool,

    /// WHOLE_SECONDS — round timeouts up to whole seconds
    #[arg(short = 'S', long = "whole-seconds", action = ArgAction::SetTrue)]
    pub whole_seconds: bool,

    /// STICKY_TIMEOUTS — system call timeouts don't decrement on
    /// EINTR
    #[arg(short = 'T', long = "sticky-timeouts", action = ArgAction::SetTrue)]
    pub sticky_timeouts: bool,

    /// Cap the address space to 3GB (ADDR_LIMIT_3GB)
    #[arg(short = '3', long = "3gb", action = ArgAction::SetTrue)]
    pub limit_3gb: bool,

    /// Reset the 3GB cap; alias for the default 4GB layout
    #[arg(long = "4gb", action = ArgAction::SetTrue)]
    pub limit_4gb: bool,

    /// Make `uname()` report kernel 2.6.x (UNAME26)
    #[arg(long = "uname-2.6", action = ArgAction::SetTrue)]
    pub uname_26: bool,

    /// Architecture name (e.g. `linux32`, `linux64`, `i686`)
    pub arch: Option<String>,

    /// Program and its arguments. Required unless `--list` /
    /// `--show` is given.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

pub fn run(args: Args) -> ExitCode {
    if args.list {
        list_archs();
        return ExitCode::SUCCESS;
    }

    if let Some(name) = args.show.as_deref() {
        return show_personality(name);
    }

    let Some(arch) = args.arch.as_deref() else {
        eprintln!("setarch: missing architecture argument (try --list)");
        return ExitCode::from(2);
    };

    let mut persona = match resolve_arch(arch) {
        Some(p) => p,
        None => {
            eprintln!("setarch: unknown architecture: {arch}");
            return ExitCode::from(2);
        }
    };
    persona = apply_flags(persona, &args);

    if args.command.is_empty() {
        eprintln!("setarch: a program to run is required");
        return ExitCode::from(2);
    }

    if args.verbose {
        eprintln!("setarch: personality = 0x{persona:08x}");
    }

    let rc = unsafe { libc::personality(persona) };
    if rc < 0 {
        eprintln!(
            "setarch: personality(0x{persona:08x}): {}",
            std::io::Error::last_os_error()
        );
        return ExitCode::FAILURE;
    }

    exec_program(&args.command[0], &args.command[1..])
}

fn resolve_arch(name: &str) -> Option<libc::c_ulong> {
    ARCH_TABLE.iter().find(|(n, _)| *n == name).map(|(_, p)| *p)
}

fn apply_flags(mut p: libc::c_ulong, args: &Args) -> libc::c_ulong {
    if args.bit32 {
        p |= PER_LINUX32;
    }
    if args.addr_no_randomize {
        p |= libc::ADDR_NO_RANDOMIZE as libc::c_ulong;
    }
    if args.fdpic_funcptrs {
        p |= libc::FDPIC_FUNCPTRS as libc::c_ulong;
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
        p |= libc::UNAME26 as libc::c_ulong;
    }
    p
}

fn list_archs() {
    for (name, _) in ARCH_TABLE {
        println!("{name}");
    }
}

fn show_personality(name: &str) -> ExitCode {
    let value = if name.is_empty() {
        let cur = unsafe { libc::personality(PERSONALITY_QUERY) };
        if cur < 0 {
            eprintln!(
                "setarch: personality(read): {}",
                std::io::Error::last_os_error()
            );
            return ExitCode::FAILURE;
        }
        cur as libc::c_ulong
    } else {
        match resolve_arch(name) {
            Some(p) => p,
            None => {
                eprintln!("setarch: unknown architecture: {name}");
                return ExitCode::from(2);
            }
        }
    };
    println!("0x{value:08x}");
    ExitCode::SUCCESS
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
    eprintln!(
        "setarch: exec {program}: {}",
        std::io::Error::last_os_error()
    );
    ExitCode::from(127)
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
        assert!(p & PER_LINUX32 != 0);

        a.limit_3gb = true;
        let p = apply_flags(PER_LINUX, &a);
        assert!(p & (libc::ADDR_LIMIT_3GB as libc::c_ulong) != 0);

        a.limit_4gb = true;
        let p = apply_flags(PER_LINUX, &a);
        // 4gb cancels 3gb
        assert!(p & (libc::ADDR_LIMIT_3GB as libc::c_ulong) == 0);
    }
}
