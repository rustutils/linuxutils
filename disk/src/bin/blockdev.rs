use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_disk::blockdev::run(linuxutils_disk::blockdev::Args::parse())
}
