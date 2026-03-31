use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_lsns::run(linuxutils_lsns::Args::parse())
}
