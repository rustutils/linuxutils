use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_ipcmk::run(linuxutils_ipcmk::Args::parse())
}
