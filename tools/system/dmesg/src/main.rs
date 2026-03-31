use clap::Parser;

fn main() -> std::process::ExitCode {
    linuxutils_dmesg::run(linuxutils_dmesg::Args::parse())
}
