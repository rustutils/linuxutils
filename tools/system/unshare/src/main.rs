use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_unshare::run(linuxutils_unshare::Args::parse())
}
