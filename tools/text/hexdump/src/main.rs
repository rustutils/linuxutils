use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_hexdump::run(linuxutils_hexdump::Args::parse())
}
