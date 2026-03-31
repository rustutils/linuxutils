use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_getopt::run(linuxutils_getopt::Args::parse())
}
