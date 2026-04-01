use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_write::run(linuxutils_write::Args::parse())
}
