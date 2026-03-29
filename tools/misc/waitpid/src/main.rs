use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_waitpid::run(rustutils_waitpid::Args::parse())
}
