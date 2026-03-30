use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    rustutils_whereis::run(rustutils_whereis::Args::parse_from(&args[1..]))
}
