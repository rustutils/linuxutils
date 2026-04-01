use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_fstrim::run(linuxutils_fstrim::Args::parse())
}
