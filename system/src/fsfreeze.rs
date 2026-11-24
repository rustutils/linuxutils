use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty().section(8);

use clap::Parser;
use std::{fs::File, io, os::unix::io::AsRawFd, process::ExitCode};

const FIFREEZE: libc::c_ulong = 0xc0045877;
const FITHAW: libc::c_ulong = 0xc0045878;

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "fsfreeze",
    about = "Suspend or resume access to a filesystem"
)]
pub struct Args {
    /// Freeze the filesystem
    #[arg(short = 'f', long, conflicts_with = "unfreeze")]
    freeze: bool,

    /// Unfreeze the filesystem
    #[arg(short = 'u', long, conflicts_with = "freeze")]
    unfreeze: bool,

    /// Mountpoint of the filesystem
    mountpoint: String,
}

fn fsfreeze_ioctl(path: &str, ioctl_nr: libc::c_ulong) -> io::Result<()> {
    let f = File::open(path)?;
    let arg: libc::c_int = 0;
    let ret =
        unsafe { libc::ioctl(f.as_raw_fd(), ioctl_nr as libc::Ioctl, &arg) };
    if ret < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub fn run(args: Args) -> ExitCode {
    if !args.freeze && !args.unfreeze {
        eprintln!("fsfreeze: one of --freeze or --unfreeze must be specified");
        return ExitCode::FAILURE;
    }

    let (ioctl_nr, action) = if args.freeze {
        (FIFREEZE, "freeze")
    } else {
        (FITHAW, "unfreeze")
    };

    if let Err(e) = fsfreeze_ioctl(&args.mountpoint, ioctl_nr) {
        eprintln!("fsfreeze: failed to {action} {}: {e}", args.mountpoint);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
