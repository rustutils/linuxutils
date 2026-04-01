use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_losetup::run(linuxutils_losetup::Args::parse())
}
