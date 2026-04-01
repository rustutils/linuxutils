use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_swapoff::run(linuxutils_swapoff::Args::parse())
}
