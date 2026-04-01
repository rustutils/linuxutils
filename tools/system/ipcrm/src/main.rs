use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_ipcrm::run(linuxutils_ipcrm::Args::parse())
}
