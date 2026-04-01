use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_nsenter::run(linuxutils_nsenter::Args::parse())
}
