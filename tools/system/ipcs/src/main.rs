use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_ipcs::run(linuxutils_ipcs::Args::parse())
}
