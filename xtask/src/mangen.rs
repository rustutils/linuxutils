use clap::CommandFactory as _;
use clap_mangen::Man;
use linuxutils_common::man::ManContent;
use std::{fs, io::Write, process::ExitCode};

type ToolEntry = (&'static str, fn() -> clap::Command, &'static ManContent);

pub fn run(output: &str) -> ExitCode {
    if let Err(e) = fs::create_dir_all(output) {
        eprintln!("xtask mangen: failed to create output directory: {e}");
        return ExitCode::FAILURE;
    }

    let tools: &[ToolEntry] = &[
        (
            "blockdev",
            linuxutils_disk::blockdev::Args::command,
            &linuxutils_disk::blockdev::MAN,
        ),
        (
            "isosize",
            linuxutils_disk::isosize::Args::command,
            &linuxutils_disk::isosize::MAN,
        ),
        (
            "nologin",
            linuxutils_login::nologin::Args::command,
            &linuxutils_login::nologin::MAN,
        ),
        (
            "cal",
            linuxutils_misc::cal::Args::command,
            &linuxutils_misc::cal::MAN,
        ),
        (
            "copyfilerange",
            linuxutils_misc::copyfilerange::Args::command,
            &linuxutils_misc::copyfilerange::MAN,
        ),
        (
            "getopt",
            linuxutils_misc::getopt::Args::command,
            &linuxutils_misc::getopt::MAN,
        ),
        (
            "kill",
            linuxutils_misc::kill::Args::command,
            &linuxutils_misc::kill::MAN,
        ),
        (
            "logger",
            linuxutils_misc::logger::Args::command,
            &linuxutils_misc::logger::MAN,
        ),
        (
            "look",
            linuxutils_misc::look::Args::command,
            &linuxutils_misc::look::MAN,
        ),
        (
            "lslocks",
            linuxutils_misc::lslocks::Args::command,
            &linuxutils_misc::lslocks::MAN,
        ),
        (
            "mcookie",
            linuxutils_misc::mcookie::Args::command,
            &linuxutils_misc::mcookie::MAN,
        ),
        (
            "namei",
            linuxutils_misc::namei::Args::command,
            &linuxutils_misc::namei::MAN,
        ),
        (
            "rename",
            linuxutils_misc::rename::Args::command,
            &linuxutils_misc::rename::MAN,
        ),
        (
            "uuidgen",
            linuxutils_misc::uuidgen::Args::command,
            &linuxutils_misc::uuidgen::MAN,
        ),
        (
            "uuidparse",
            linuxutils_misc::uuidparse::Args::command,
            &linuxutils_misc::uuidparse::MAN,
        ),
        (
            "waitpid",
            linuxutils_misc::waitpid::Args::command,
            &linuxutils_misc::waitpid::MAN,
        ),
        (
            "whereis",
            linuxutils_misc::whereis::Args::command,
            &linuxutils_misc::whereis::MAN,
        ),
        (
            "chrt",
            linuxutils_sched::chrt::Args::command,
            &linuxutils_sched::chrt::MAN,
        ),
        (
            "ionice",
            linuxutils_sched::ionice::Args::command,
            &linuxutils_sched::ionice::MAN,
        ),
        (
            "taskset",
            linuxutils_sched::taskset::Args::command,
            &linuxutils_sched::taskset::MAN,
        ),
        (
            "blkdiscard",
            linuxutils_system::blkdiscard::Args::command,
            &linuxutils_system::blkdiscard::MAN,
        ),
        (
            "choom",
            linuxutils_system::choom::Args::command,
            &linuxutils_system::choom::MAN,
        ),
        (
            "ctrlaltdel",
            linuxutils_system::ctrlaltdel::Args::command,
            &linuxutils_system::ctrlaltdel::MAN,
        ),
        (
            "dmesg",
            linuxutils_system::dmesg::Args::command,
            &linuxutils_system::dmesg::MAN,
        ),
        (
            "eject",
            linuxutils_system::eject::Args::command,
            &linuxutils_system::eject::MAN,
        ),
        (
            "fadvise",
            linuxutils_system::fadvise::Args::command,
            &linuxutils_system::fadvise::MAN,
        ),
        (
            "fallocate",
            linuxutils_system::fallocate::Args::command,
            &linuxutils_system::fallocate::MAN,
        ),
        (
            "flock",
            linuxutils_system::flock::Args::command,
            &linuxutils_system::flock::MAN,
        ),
        (
            "fsfreeze",
            linuxutils_system::fsfreeze::Args::command,
            &linuxutils_system::fsfreeze::MAN,
        ),
        (
            "fstrim",
            linuxutils_system::fstrim::Args::command,
            &linuxutils_system::fstrim::MAN,
        ),
        (
            "ipcmk",
            linuxutils_system::ipcmk::Args::command,
            &linuxutils_system::ipcmk::MAN,
        ),
        (
            "ipcrm",
            linuxutils_system::ipcrm::Args::command,
            &linuxutils_system::ipcrm::MAN,
        ),
        (
            "ipcs",
            linuxutils_system::ipcs::Args::command,
            &linuxutils_system::ipcs::MAN,
        ),
        (
            "losetup",
            linuxutils_system::losetup::Args::command,
            &linuxutils_system::losetup::MAN,
        ),
        (
            "lscpu",
            linuxutils_system::lscpu::Args::command,
            &linuxutils_system::lscpu::MAN,
        ),
        (
            "lsipc",
            linuxutils_system::lsipc::Args::command,
            &linuxutils_system::lsipc::MAN,
        ),
        (
            "lsirq",
            linuxutils_system::lsirq::Args::command,
            &linuxutils_system::lsirq::MAN,
        ),
        (
            "lsmem",
            linuxutils_system::lsmem::Args::command,
            &linuxutils_system::lsmem::MAN,
        ),
        (
            "lsns",
            linuxutils_system::lsns::Args::command,
            &linuxutils_system::lsns::MAN,
        ),
        (
            "mountpoint",
            linuxutils_system::mountpoint::Args::command,
            &linuxutils_system::mountpoint::MAN,
        ),
        (
            "nsenter",
            linuxutils_system::nsenter::Args::command,
            &linuxutils_system::nsenter::MAN,
        ),
        (
            "pipesz",
            linuxutils_system::pipesz::Args::command,
            &linuxutils_system::pipesz::MAN,
        ),
        (
            "pivot_root",
            linuxutils_system::pivot_root::Args::command,
            &linuxutils_system::pivot_root::MAN,
        ),
        (
            "prlimit",
            linuxutils_system::prlimit::Args::command,
            &linuxutils_system::prlimit::MAN,
        ),
        (
            "renice",
            linuxutils_system::renice::Args::command,
            &linuxutils_system::renice::MAN,
        ),
        (
            "rfkill",
            linuxutils_system::rfkill::Args::command,
            &linuxutils_system::rfkill::MAN,
        ),
        (
            "setpgid",
            linuxutils_system::setpgid::Args::command,
            &linuxutils_system::setpgid::MAN,
        ),
        (
            "setsid",
            linuxutils_system::setsid::Args::command,
            &linuxutils_system::setsid::MAN,
        ),
        (
            "swapoff",
            linuxutils_system::swapoff::Args::command,
            &linuxutils_system::swapoff::MAN,
        ),
        (
            "swapon",
            linuxutils_system::swapon::Args::command,
            &linuxutils_system::swapon::MAN,
        ),
        (
            "unshare",
            linuxutils_system::unshare::Args::command,
            &linuxutils_system::unshare::MAN,
        ),
        (
            "mesg",
            linuxutils_term::mesg::Args::command,
            &linuxutils_term::mesg::MAN,
        ),
        (
            "wall",
            linuxutils_term::wall::Args::command,
            &linuxutils_term::wall::MAN,
        ),
        (
            "write",
            linuxutils_term::write::Args::command,
            &linuxutils_term::write::MAN,
        ),
        (
            "colrm",
            linuxutils_text::colrm::Args::command,
            &linuxutils_text::colrm::MAN,
        ),
        (
            "column",
            linuxutils_text::column::Args::command,
            &linuxutils_text::column::MAN,
        ),
        (
            "hexdump",
            linuxutils_text::hexdump::Args::command,
            &linuxutils_text::hexdump::MAN,
        ),
        (
            "line",
            linuxutils_text::line::Args::command,
            &linuxutils_text::line::MAN,
        ),
        (
            "rev",
            linuxutils_text::rev::Args::command,
            &linuxutils_text::rev::MAN,
        ),
    ];

    for (name, factory, extras) in tools {
        let path = format!("{output}/{name}.1");
        let mut f = match fs::File::create(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("xtask mangen: failed to create {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        if let Err(e) = render_man(&mut f, factory(), extras) {
            eprintln!("xtask mangen: failed to render {name}: {e}");
            return ExitCode::FAILURE;
        }
        println!("{path}");
    }

    ExitCode::SUCCESS
}

/// Compose a man page by interleaving clap_mangen's auto-rendered
/// sections with the tool's hand-written extras.
fn render_man(
    out: &mut impl Write,
    cmd: clap::Command,
    extras: &ManContent,
) -> std::io::Result<()> {
    // clap_mangen panics on render_version_section / render_authors_section
    // when these aren't set; mirror Man::render's internal gating.
    let has_version =
        cmd.get_long_version().is_some() || cmd.get_version().is_some();
    let has_author = cmd.get_author().is_some();

    let man = Man::new(cmd);

    man.render_title(out)?;
    man.render_name_section(out)?;
    man.render_synopsis_section(out)?;

    // DESCRIPTION: clap_mangen would emit a short blurb from clap's
    // `about`. Override with our richer prose if provided, fall back to
    // clap's auto version otherwise.
    if let Some(desc) = extras.description {
        writeln!(out, ".SH DESCRIPTION")?;
        writeln!(out, "{desc}")?;
    } else {
        man.render_description_section(out)?;
    }

    man.render_options_section(out)?;

    for (title, body) in extras.extra_sections {
        writeln!(out, ".SH {title}")?;
        writeln!(out, "{body}")?;
    }

    if has_version {
        man.render_version_section(out)?;
    }
    if has_author {
        man.render_authors_section(out)?;
    }
    Ok(())
}
