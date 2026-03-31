use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_lsmem::run(linuxutils_lsmem::Args::parse())
}
