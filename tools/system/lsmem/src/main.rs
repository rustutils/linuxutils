use clap::Parser;

fn main() -> std::process::ExitCode {
    rustutils_lsmem::run(rustutils_lsmem::Args::parse())
}
