use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_swapon::run(linuxutils_swapon::Args::parse())
}
