use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_wall::run(linuxutils_wall::Args::parse())
}
