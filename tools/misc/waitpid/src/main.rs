use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_waitpid::run(linuxutils_waitpid::Args::parse())
}
