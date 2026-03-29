use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_lsns::run(rustutils_lsns::Args::parse())
}
