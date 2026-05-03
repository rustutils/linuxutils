use clap::{ArgMatches, Command, CommandFactory as _, FromArgMatches as _};
use std::process::ExitCode;

fn tool_subcommands(cmd: Command) -> Command {
    cmd.subcommand(linuxutils_disk::blockdev::Args::command())
        .subcommand(linuxutils_disk::isosize::Args::command())
        .subcommand(linuxutils_login::nologin::Args::command())
        .subcommand(linuxutils_misc::cal::Args::command())
        .subcommand(linuxutils_misc::copyfilerange::Args::command())
        .subcommand(linuxutils_misc::getopt::Args::command())
        .subcommand(linuxutils_misc::kill::Args::command())
        .subcommand(linuxutils_misc::logger::Args::command())
        .subcommand(linuxutils_misc::look::Args::command())
        .subcommand(linuxutils_misc::lslocks::Args::command())
        .subcommand(linuxutils_misc::mcookie::Args::command())
        .subcommand(linuxutils_misc::namei::Args::command())
        .subcommand(linuxutils_misc::rename::Args::command())
        .subcommand(linuxutils_misc::uuidgen::Args::command())
        .subcommand(linuxutils_misc::uuidparse::Args::command())
        .subcommand(linuxutils_misc::waitpid::Args::command())
        .subcommand(linuxutils_misc::whereis::Args::command())
        .subcommand(linuxutils_sched::chrt::Args::command())
        .subcommand(linuxutils_sched::ionice::Args::command())
        .subcommand(linuxutils_sched::taskset::Args::command())
        .subcommand(linuxutils_system::blkdiscard::Args::command())
        .subcommand(linuxutils_system::choom::Args::command())
        .subcommand(linuxutils_system::ctrlaltdel::Args::command())
        .subcommand(linuxutils_system::dmesg::Args::command())
        .subcommand(linuxutils_system::eject::Args::command())
        .subcommand(linuxutils_system::fadvise::Args::command())
        .subcommand(linuxutils_system::fallocate::Args::command())
        .subcommand(linuxutils_system::flock::Args::command())
        .subcommand(linuxutils_system::fsfreeze::Args::command())
        .subcommand(linuxutils_system::fstrim::Args::command())
        .subcommand(linuxutils_system::ipcmk::Args::command())
        .subcommand(linuxutils_system::ipcrm::Args::command())
        .subcommand(linuxutils_system::ipcs::Args::command())
        .subcommand(linuxutils_system::losetup::Args::command())
        .subcommand(linuxutils_system::lscpu::Args::command())
        .subcommand(linuxutils_system::lsipc::Args::command())
        .subcommand(linuxutils_system::lsirq::Args::command())
        .subcommand(linuxutils_system::lsmem::Args::command())
        .subcommand(linuxutils_system::lsns::Args::command())
        .subcommand(linuxutils_system::mountpoint::Args::command())
        .subcommand(linuxutils_system::nsenter::Args::command())
        .subcommand(linuxutils_system::pipesz::Args::command())
        .subcommand(linuxutils_system::pivot_root::Args::command())
        .subcommand(linuxutils_system::prlimit::Args::command())
        .subcommand(linuxutils_system::renice::Args::command())
        .subcommand(linuxutils_system::rfkill::Args::command())
        .subcommand(linuxutils_system::setarch::Args::command())
        .subcommand(linuxutils_system::setpgid::Args::command())
        .subcommand(linuxutils_system::setsid::Args::command())
        .subcommand(linuxutils_system::swapoff::Args::command())
        .subcommand(linuxutils_system::swapon::Args::command())
        .subcommand(linuxutils_system::unshare::Args::command())
        .subcommand(linuxutils_system::wdctl::Args::command())
        .subcommand(linuxutils_term::mesg::Args::command())
        .subcommand(linuxutils_term::wall::Args::command())
        .subcommand(linuxutils_term::write::Args::command())
        .subcommand(linuxutils_text::colrm::Args::command())
        .subcommand(linuxutils_text::column::Args::command())
        .subcommand(linuxutils_text::hexdump::Args::command())
        .subcommand(linuxutils_text::line::Args::command())
        .subcommand(linuxutils_text::rev::Args::command())
}

fn dispatch(name: &str, m: &ArgMatches) -> ExitCode {
    match name {
        "blockdev" => linuxutils_disk::blockdev::run(
            linuxutils_disk::blockdev::Args::from_arg_matches(m).unwrap(),
        ),
        "isosize" => linuxutils_disk::isosize::run(
            linuxutils_disk::isosize::Args::from_arg_matches(m).unwrap(),
        ),
        "nologin" => linuxutils_login::nologin::run(
            linuxutils_login::nologin::Args::from_arg_matches(m).unwrap(),
        ),
        "cal" => linuxutils_misc::cal::run(
            linuxutils_misc::cal::Args::from_arg_matches(m).unwrap(),
        ),
        "copyfilerange" => linuxutils_misc::copyfilerange::run(
            linuxutils_misc::copyfilerange::Args::from_arg_matches(m).unwrap(),
        ),
        "getopt" => linuxutils_misc::getopt::run(
            linuxutils_misc::getopt::Args::from_arg_matches(m).unwrap(),
        ),
        "kill" => linuxutils_misc::kill::run(
            linuxutils_misc::kill::Args::from_arg_matches(m).unwrap(),
        ),
        "logger" => linuxutils_misc::logger::run(
            linuxutils_misc::logger::Args::from_arg_matches(m).unwrap(),
        ),
        "look" => linuxutils_misc::look::run(
            linuxutils_misc::look::Args::from_arg_matches(m).unwrap(),
        ),
        "lslocks" => linuxutils_misc::lslocks::run(
            linuxutils_misc::lslocks::Args::from_arg_matches(m).unwrap(),
        ),
        "mcookie" => linuxutils_misc::mcookie::run(
            linuxutils_misc::mcookie::Args::from_arg_matches(m).unwrap(),
        ),
        "namei" => linuxutils_misc::namei::run(
            linuxutils_misc::namei::Args::from_arg_matches(m).unwrap(),
        ),
        "rename" => linuxutils_misc::rename::run(
            linuxutils_misc::rename::Args::from_arg_matches(m).unwrap(),
        ),
        "uuidgen" => linuxutils_misc::uuidgen::run(
            linuxutils_misc::uuidgen::Args::from_arg_matches(m).unwrap(),
        ),
        "uuidparse" => linuxutils_misc::uuidparse::run(
            linuxutils_misc::uuidparse::Args::from_arg_matches(m).unwrap(),
        ),
        "waitpid" => linuxutils_misc::waitpid::run(
            linuxutils_misc::waitpid::Args::from_arg_matches(m).unwrap(),
        ),
        "whereis" => linuxutils_misc::whereis::run(
            linuxutils_misc::whereis::Args::from_arg_matches(m).unwrap(),
        ),
        "chrt" => linuxutils_sched::chrt::run(
            linuxutils_sched::chrt::Args::from_arg_matches(m).unwrap(),
        ),
        "ionice" => linuxutils_sched::ionice::run(
            linuxutils_sched::ionice::Args::from_arg_matches(m).unwrap(),
        ),
        "taskset" => linuxutils_sched::taskset::run(
            linuxutils_sched::taskset::Args::from_arg_matches(m).unwrap(),
        ),
        "blkdiscard" => linuxutils_system::blkdiscard::run(
            linuxutils_system::blkdiscard::Args::from_arg_matches(m).unwrap(),
        ),
        "choom" => linuxutils_system::choom::run(
            linuxutils_system::choom::Args::from_arg_matches(m).unwrap(),
        ),
        "ctrlaltdel" => linuxutils_system::ctrlaltdel::run(
            linuxutils_system::ctrlaltdel::Args::from_arg_matches(m).unwrap(),
        ),
        "dmesg" => linuxutils_system::dmesg::run(
            linuxutils_system::dmesg::Args::from_arg_matches(m).unwrap(),
        ),
        "eject" => linuxutils_system::eject::run(
            linuxutils_system::eject::Args::from_arg_matches(m).unwrap(),
        ),
        "fadvise" => linuxutils_system::fadvise::run(
            linuxutils_system::fadvise::Args::from_arg_matches(m).unwrap(),
        ),
        "fallocate" => linuxutils_system::fallocate::run(
            linuxutils_system::fallocate::Args::from_arg_matches(m).unwrap(),
        ),
        "flock" => linuxutils_system::flock::run(
            linuxutils_system::flock::Args::from_arg_matches(m).unwrap(),
        ),
        "fsfreeze" => linuxutils_system::fsfreeze::run(
            linuxutils_system::fsfreeze::Args::from_arg_matches(m).unwrap(),
        ),
        "fstrim" => linuxutils_system::fstrim::run(
            linuxutils_system::fstrim::Args::from_arg_matches(m).unwrap(),
        ),
        "ipcmk" => linuxutils_system::ipcmk::run(
            linuxutils_system::ipcmk::Args::from_arg_matches(m).unwrap(),
        ),
        "ipcrm" => linuxutils_system::ipcrm::run(
            linuxutils_system::ipcrm::Args::from_arg_matches(m).unwrap(),
        ),
        "ipcs" => linuxutils_system::ipcs::run(
            linuxutils_system::ipcs::Args::from_arg_matches(m).unwrap(),
        ),
        "losetup" => linuxutils_system::losetup::run(
            linuxutils_system::losetup::Args::from_arg_matches(m).unwrap(),
        ),
        "lscpu" => linuxutils_system::lscpu::run(
            linuxutils_system::lscpu::Args::from_arg_matches(m).unwrap(),
        ),
        "lsipc" => linuxutils_system::lsipc::run(
            linuxutils_system::lsipc::Args::from_arg_matches(m).unwrap(),
        ),
        "lsirq" => linuxutils_system::lsirq::run(
            linuxutils_system::lsirq::Args::from_arg_matches(m).unwrap(),
        ),
        "lsmem" => linuxutils_system::lsmem::run(
            linuxutils_system::lsmem::Args::from_arg_matches(m).unwrap(),
        ),
        "lsns" => linuxutils_system::lsns::run(
            linuxutils_system::lsns::Args::from_arg_matches(m).unwrap(),
        ),
        "mountpoint" => linuxutils_system::mountpoint::run(
            linuxutils_system::mountpoint::Args::from_arg_matches(m).unwrap(),
        ),
        "nsenter" => linuxutils_system::nsenter::run(
            linuxutils_system::nsenter::Args::from_arg_matches(m).unwrap(),
        ),
        "pipesz" => linuxutils_system::pipesz::run(
            linuxutils_system::pipesz::Args::from_arg_matches(m).unwrap(),
        ),
        "pivot_root" => linuxutils_system::pivot_root::run(
            linuxutils_system::pivot_root::Args::from_arg_matches(m).unwrap(),
        ),
        "prlimit" => linuxutils_system::prlimit::run(
            linuxutils_system::prlimit::Args::from_arg_matches(m).unwrap(),
        ),
        "renice" => linuxutils_system::renice::run(
            linuxutils_system::renice::Args::from_arg_matches(m).unwrap(),
        ),
        "rfkill" => linuxutils_system::rfkill::run(
            linuxutils_system::rfkill::Args::from_arg_matches(m).unwrap(),
        ),
        "setarch" => linuxutils_system::setarch::run(
            linuxutils_system::setarch::Args::from_arg_matches(m).unwrap(),
        ),
        "setpgid" => linuxutils_system::setpgid::run(
            linuxutils_system::setpgid::Args::from_arg_matches(m).unwrap(),
        ),
        "setsid" => linuxutils_system::setsid::run(
            linuxutils_system::setsid::Args::from_arg_matches(m).unwrap(),
        ),
        "swapoff" => linuxutils_system::swapoff::run(
            linuxutils_system::swapoff::Args::from_arg_matches(m).unwrap(),
        ),
        "swapon" => linuxutils_system::swapon::run(
            linuxutils_system::swapon::Args::from_arg_matches(m).unwrap(),
        ),
        "unshare" => linuxutils_system::unshare::run(
            linuxutils_system::unshare::Args::from_arg_matches(m).unwrap(),
        ),
        "wdctl" => linuxutils_system::wdctl::run(
            linuxutils_system::wdctl::Args::from_arg_matches(m).unwrap(),
        ),
        "mesg" => linuxutils_term::mesg::run(
            linuxutils_term::mesg::Args::from_arg_matches(m).unwrap(),
        ),
        "wall" => linuxutils_term::wall::run(
            linuxutils_term::wall::Args::from_arg_matches(m).unwrap(),
        ),
        "write" => linuxutils_term::write::run(
            linuxutils_term::write::Args::from_arg_matches(m).unwrap(),
        ),
        "colrm" => linuxutils_text::colrm::run(
            linuxutils_text::colrm::Args::from_arg_matches(m).unwrap(),
        ),
        "column" => linuxutils_text::column::run(
            linuxutils_text::column::Args::from_arg_matches(m).unwrap(),
        ),
        "hexdump" => linuxutils_text::hexdump::run(
            linuxutils_text::hexdump::Args::from_arg_matches(m).unwrap(),
        ),
        "line" => linuxutils_text::line::run(
            linuxutils_text::line::Args::from_arg_matches(m).unwrap(),
        ),
        "rev" => linuxutils_text::rev::run(
            linuxutils_text::rev::Args::from_arg_matches(m).unwrap(),
        ),
        _ => unreachable!("clap ensures a valid subcommand"),
    }
}

fn main() -> ExitCode {
    let cmd = Command::new("linuxutils").multicall(true);
    let cmd = tool_subcommands(cmd);
    let cmd = cmd.subcommand(tool_subcommands(
        Command::new("linuxutils").subcommand_required(true),
    ));

    let matches = cmd.get_matches();

    let (name, m) = match matches.subcommand() {
        Some(("linuxutils", m)) => m.subcommand().unwrap(),
        Some((name, m)) => (name, m),
        None => unreachable!("clap ensures a valid subcommand"),
    };

    dispatch(name, m)
}
