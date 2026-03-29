use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_cal::run(rustutils_cal::Args::parse())
}
