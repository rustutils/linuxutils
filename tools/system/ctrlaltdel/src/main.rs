use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_ctrlaltdel::run(linuxutils_ctrlaltdel::Args::parse())
}
