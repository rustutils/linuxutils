use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_cal::run(linuxutils_cal::Args::parse())
}
