use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_blockdev::run(linuxutils_blockdev::Args::parse())
}
