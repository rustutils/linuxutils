use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_eject::run(linuxutils_eject::Args::parse())
}
