use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_fsfreeze::run(linuxutils_fsfreeze::Args::parse())
}
