use clap::CommandFactory as _;
use clap_mangen::Man;
use std::{fs, process::ExitCode};

fn main() -> ExitCode {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/man".to_string());

    if let Err(e) = fs::create_dir_all(&output) {
        eprintln!("mangen: failed to create output directory: {e}");
        return ExitCode::FAILURE;
    }

    let tools: Vec<clap::Command> = vec![
        linuxutils_cal::Args::command(),
        linuxutils_choom::Args::command(),
        linuxutils_colrm::Args::command(),
        linuxutils_flock::Args::command(),
        linuxutils_hexdump::Args::command(),
        linuxutils_ionice::Args::command(),
        linuxutils_kill::Args::command(),
        linuxutils_line::Args::command(),
        linuxutils_look::Args::command(),
        linuxutils_lsirq::Args::command(),
        linuxutils_lslocks::Args::command(),
        linuxutils_lsmem::Args::command(),
        linuxutils_lsns::Args::command(),
        linuxutils_mcookie::Args::command(),
        linuxutils_mesg::Args::command(),
        linuxutils_namei::Args::command(),
        linuxutils_mountpoint::Args::command(),
        linuxutils_nologin::Args::command(),
        linuxutils_prlimit::Args::command(),
        linuxutils_renice::Args::command(),
        linuxutils_setpgid::Args::command(),
        linuxutils_setsid::Args::command(),
        linuxutils_rev::Args::command(),
        linuxutils_rfkill::Args::command(),
        linuxutils_uuidgen::Args::command(),
        linuxutils_uuidparse::Args::command(),
        linuxutils_waitpid::Args::command(),
    ];

    for cmd in tools {
        let name = cmd.get_name().to_string();
        let man = Man::new(cmd);
        let path = format!("{output}/{name}.1");

        let mut buf = Vec::new();
        if let Err(e) = man.render(&mut buf) {
            eprintln!("mangen: failed to render {name}: {e}");
            return ExitCode::FAILURE;
        }

        if let Err(e) = fs::write(&path, buf) {
            eprintln!("mangen: failed to write {path}: {e}");
            return ExitCode::FAILURE;
        }

        println!("{path}");
    }

    ExitCode::SUCCESS
}
