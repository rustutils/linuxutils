use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_lscpu::run(linuxutils_lscpu::Args::parse())
}
