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
        rustutils_colrm::Args::command(),
        rustutils_line::Args::command(),
        rustutils_look::Args::command(),
        rustutils_lsmem::Args::command(),
        rustutils_mcookie::Args::command(),
        rustutils_mesg::Args::command(),
        rustutils_mountpoint::Args::command(),
        rustutils_nologin::Args::command(),
        rustutils_setpgid::Args::command(),
        rustutils_setsid::Args::command(),
        rustutils_rev::Args::command(),
        rustutils_uuidgen::Args::command(),
        rustutils_uuidparse::Args::command(),
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
