use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_kill::run(linuxutils_kill::Args::parse())
}
