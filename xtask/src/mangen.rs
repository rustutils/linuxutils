use clap::CommandFactory as _;
use clap_mangen::Man;
use std::{fs, process::ExitCode};

pub fn run(output: &str) -> ExitCode {
    if let Err(e) = fs::create_dir_all(output) {
        eprintln!("xtask mangen: failed to create output directory: {e}");
        return ExitCode::FAILURE;
    }

    let tools: Vec<clap::Command> = vec![
        linuxutils_disk::blockdev::Args::command(),
        linuxutils_disk::isosize::Args::command(),
        linuxutils_login::nologin::Args::command(),
        linuxutils_misc::cal::Args::command(),
        linuxutils_misc::copyfilerange::Args::command(),
        linuxutils_misc::getopt::Args::command(),
        linuxutils_misc::kill::Args::command(),
        linuxutils_misc::logger::Args::command(),
        linuxutils_misc::look::Args::command(),
        linuxutils_misc::lslocks::Args::command(),
        linuxutils_misc::mcookie::Args::command(),
        linuxutils_misc::namei::Args::command(),
        linuxutils_misc::rename::Args::command(),
        linuxutils_misc::uuidgen::Args::command(),
        linuxutils_misc::uuidparse::Args::command(),
        linuxutils_misc::waitpid::Args::command(),
        linuxutils_misc::whereis::Args::command(),
        linuxutils_sched::chrt::Args::command(),
        linuxutils_sched::ionice::Args::command(),
        linuxutils_sched::taskset::Args::command(),
        linuxutils_system::blkdiscard::Args::command(),
        linuxutils_system::choom::Args::command(),
        linuxutils_system::ctrlaltdel::Args::command(),
        linuxutils_system::dmesg::Args::command(),
        linuxutils_system::eject::Args::command(),
        linuxutils_system::fadvise::Args::command(),
        linuxutils_system::fallocate::Args::command(),
        linuxutils_system::flock::Args::command(),
        linuxutils_system::fsfreeze::Args::command(),
        linuxutils_system::fstrim::Args::command(),
        linuxutils_system::ipcmk::Args::command(),
        linuxutils_system::ipcrm::Args::command(),
        linuxutils_system::ipcs::Args::command(),
        linuxutils_system::losetup::Args::command(),
        linuxutils_system::lscpu::Args::command(),
        linuxutils_system::lsipc::Args::command(),
        linuxutils_system::lsirq::Args::command(),
        linuxutils_system::lsmem::Args::command(),
        linuxutils_system::lsns::Args::command(),
        linuxutils_system::mountpoint::Args::command(),
        linuxutils_system::nsenter::Args::command(),
        linuxutils_system::pipesz::Args::command(),
        linuxutils_system::pivot_root::Args::command(),
        linuxutils_system::prlimit::Args::command(),
        linuxutils_system::renice::Args::command(),
        linuxutils_system::rfkill::Args::command(),
        linuxutils_system::setpgid::Args::command(),
        linuxutils_system::setsid::Args::command(),
        linuxutils_system::swapoff::Args::command(),
        linuxutils_system::swapon::Args::command(),
        linuxutils_system::unshare::Args::command(),
        linuxutils_term::mesg::Args::command(),
        linuxutils_term::wall::Args::command(),
        linuxutils_term::write::Args::command(),
        linuxutils_text::colrm::Args::command(),
        linuxutils_text::column::Args::command(),
        linuxutils_text::hexdump::Args::command(),
        linuxutils_text::line::Args::command(),
        linuxutils_text::rev::Args::command(),
    ];

    for cmd in tools {
        let name = cmd.get_name().to_string();
        let man = Man::new(cmd);
        let path = format!("{output}/{name}.1");

        let mut buf = Vec::new();
        if let Err(e) = man.render(&mut buf) {
            eprintln!("xtask mangen: failed to render {name}: {e}");
            return ExitCode::FAILURE;
        }

        if let Err(e) = fs::write(&path, buf) {
            eprintln!("xtask mangen: failed to write {path}: {e}");
            return ExitCode::FAILURE;
        }

        println!("{path}");
    }

    ExitCode::SUCCESS
}
