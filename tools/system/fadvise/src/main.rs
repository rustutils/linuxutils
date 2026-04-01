use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_fadvise::run(linuxutils_fadvise::Args::parse())
}
