use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    linuxutils_misc::whereis::run(linuxutils_misc::whereis::Args::parse_from(
        &args[1..],
    ))
}
