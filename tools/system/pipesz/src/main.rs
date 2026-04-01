use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_pipesz::run(linuxutils_pipesz::Args::parse())
}
