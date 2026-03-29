use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_hexdump::run(rustutils_hexdump::Args::parse())
}
