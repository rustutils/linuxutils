use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_lsirq::run(rustutils_lsirq::Args::parse())
}
