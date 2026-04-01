use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_pivot_root::run(linuxutils_pivot_root::Args::parse())
}
