use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_lsipc::run(linuxutils_lsipc::Args::parse())
}
