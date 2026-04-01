use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_copyfilerange::run(linuxutils_copyfilerange::Args::parse())
}
