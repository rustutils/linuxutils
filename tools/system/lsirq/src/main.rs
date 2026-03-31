use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_lsirq::run(linuxutils_lsirq::Args::parse())
}
