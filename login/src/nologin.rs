use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent {
    description: Some(include_str!("../man/nologin/description.man")),
    extra_sections: &[
        ("EXAMPLES", include_str!("../man/nologin/examples.man")),
        ("FILES", include_str!("../man/nologin/files.man")),
        ("EXIT STATUS", include_str!("../man/nologin/exit_status.man")),
        ("SEE ALSO", include_str!("../man/nologin/see_also.man")),
    ],
};

use clap::Parser;
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

const DEFAULT_MESSAGE: &str = "This account is currently not available.";
const NOLOGIN_TXT: &str = "/etc/nologin.txt";

/// Politely refuse a login.
///
/// Displays a message that an account is not available and exits non-zero.
/// Intended as a replacement shell field to deny login access to an account.
#[derive(Parser)]
#[command(name = "nologin", version, about = "Politely refuse a login")]
pub struct Args {
    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    command: Option<String>,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    init_file: Option<PathBuf>,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    interactive: bool,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    login: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    noprofile: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    norc: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    posix: bool,

    /// Ignored (shell compatibility).
    #[arg(long, hide = true)]
    rcfile: Option<PathBuf>,

    /// Ignored (shell compatibility).
    #[arg(short, long, hide = true)]
    restricted: bool,
}

pub fn run(_args: Args) -> ExitCode {
    let mut stderr = io::stderr().lock();

    if let Err(e) = nologin(&mut stderr) {
        eprintln!("nologin: {e}");
    }

    ExitCode::from(1)
}

/// Print the nologin message to `writer`.
///
/// If `/etc/nologin.txt` exists, its contents are used. Otherwise, the default
/// message is printed.
pub fn nologin(writer: &mut impl Write) -> io::Result<()> {
    let path = Path::new(NOLOGIN_TXT);
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        write!(writer, "{contents}")?;
    } else {
        writeln!(writer, "{DEFAULT_MESSAGE}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_message() {
        let mut buf = Vec::new();
        nologin(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
    }
}
