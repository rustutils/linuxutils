use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_blkdiscard::run(linuxutils_blkdiscard::Args::parse())
}
