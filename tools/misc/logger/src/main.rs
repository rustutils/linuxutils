use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_logger::run(linuxutils_logger::Args::parse())
}
