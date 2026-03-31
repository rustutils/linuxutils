use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    linuxutils_whereis::run(linuxutils_whereis::Args::parse_from(&args[1..]))
}
