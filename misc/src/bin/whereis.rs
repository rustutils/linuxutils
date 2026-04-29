use linuxutils_misc::whereis::*;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    run(Args::parse_from(&args[1..]))
}
