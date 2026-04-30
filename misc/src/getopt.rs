use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "getopt",
    about = "Parse command options (enhanced)",
    override_usage = "getopt optstring parameters\n       \
                      getopt [options] [--] optstring parameters\n       \
                      getopt [options] -o|--options optstring [--] parameters"
)]
pub struct Args {
    /// Allow long options to start with a single -
    #[arg(short = 'a', long = "alternative")]
    alternative: bool,

    /// Long options to recognize (comma-separated)
    #[arg(short = 'l', long = "longoptions", value_delimiter = ',')]
    longoptions: Vec<String>,

    /// Program name for error messages
    #[arg(short = 'n', long = "name")]
    name: Option<String>,

    /// Short options string to recognize
    #[arg(short = 'o', long = "options")]
    options: Option<String>,

    /// Disable error output from getopt
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Suppress normal output
    #[arg(short = 'Q', long = "quiet-output")]
    quiet_output: bool,

    /// Set quoting conventions for shell (sh, bash, csh, tcsh)
    #[arg(short = 's', long = "shell", default_value = "bash")]
    shell: String,

    /// Test for enhanced getopt version (returns exit code 4)
    #[arg(short = 'T', long = "test")]
    test: bool,

    /// Don't quote the output
    #[arg(short = 'u', long = "unquoted")]
    unquoted: bool,

    /// Parameters to parse
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    params: Vec<String>,
}

#[derive(Clone, Copy)]
enum ArgReq {
    None,
    Required,
    Optional,
}

struct ShortOpt {
    ch: char,
    arg_req: ArgReq,
}

struct LongOpt {
    name: String,
    arg_req: ArgReq,
}

#[derive(Clone, Copy)]
enum ScanMode {
    Permute,
    StopAtFirst,
    InPlace,
}

#[derive(Clone, Copy)]
enum Shell {
    Sh,
    Csh,
}

fn parse_optstring(s: &str) -> (Vec<ShortOpt>, ScanMode) {
    let mut opts = Vec::new();
    let mut mode = ScanMode::Permute;
    let mut chars = s.chars().peekable();

    if chars.peek() == Some(&'+') {
        mode = ScanMode::StopAtFirst;
        chars.next();
    } else if chars.peek() == Some(&'-') {
        mode = ScanMode::InPlace;
        chars.next();
    }

    // Leading : suppresses error reporting (handled separately via -q)
    if chars.peek() == Some(&':') {
        chars.next();
    }

    while let Some(ch) = chars.next() {
        let arg_req = if chars.peek() == Some(&':') {
            chars.next();
            if chars.peek() == Some(&':') {
                chars.next();
                ArgReq::Optional
            } else {
                ArgReq::Required
            }
        } else {
            ArgReq::None
        };
        opts.push(ShortOpt { ch, arg_req });
    }

    (opts, mode)
}

fn parse_longopt_spec(spec: &str) -> LongOpt {
    if let Some(name) = spec.strip_suffix("::") {
        LongOpt {
            name: name.to_string(),
            arg_req: ArgReq::Optional,
        }
    } else if let Some(name) = spec.strip_suffix(':') {
        LongOpt {
            name: name.to_string(),
            arg_req: ArgReq::Required,
        }
    } else {
        LongOpt {
            name: spec.to_string(),
            arg_req: ArgReq::None,
        }
    }
}

fn quote(s: &str, shell: Shell, unquoted: bool) -> String {
    if unquoted {
        return s.to_string();
    }
    match shell {
        Shell::Sh => {
            if s.is_empty() {
                return "''".to_string();
            }
            format!("'{}'", s.replace('\'', "'\\''"))
        }
        Shell::Csh => {
            if s.is_empty() {
                return "''".to_string();
            }
            let mut out = String::from("'");
            for ch in s.chars() {
                match ch {
                    '\'' => out.push_str("'\\''"),
                    '!' => out.push_str("\\!"),
                    '\n' => out.push_str("'\\\n'"),
                    _ => out.push(ch),
                }
            }
            out.push('\'');
            out
        }
    }
}

struct Ctx<'a> {
    long_opts: &'a [LongOpt],
    prog_name: &'a str,
    quiet: bool,
    shell: Shell,
    unquoted: bool,
}

impl Ctx<'_> {
    fn quote(&self, s: &str) -> String {
        quote(s, self.shell, self.unquoted)
    }
}

fn handle_long_option(
    name: &str,
    value: Option<&str>,
    ctx: &Ctx<'_>,
    args: &[String],
    i: &mut usize,
    output: &mut Vec<String>,
) -> bool {
    let matches: Vec<_> = ctx
        .long_opts
        .iter()
        .filter(|o| o.name.starts_with(name))
        .collect();

    match matches.len() {
        0 => {
            if !ctx.quiet {
                eprintln!("{}: unrecognized option '--{name}'", ctx.prog_name);
            }
            true
        }
        1 => {
            let opt = matches[0];
            output.push(format!("--{}", opt.name));
            match opt.arg_req {
                ArgReq::Required => {
                    if let Some(v) = value {
                        output.push(ctx.quote(v));
                    } else {
                        *i += 1;
                        if *i < args.len() {
                            output.push(ctx.quote(&args[*i]));
                        } else {
                            if !ctx.quiet {
                                eprintln!(
                                    "{}: option '--{}' requires an argument",
                                    ctx.prog_name, opt.name
                                );
                            }
                            return true;
                        }
                    }
                }
                ArgReq::Optional => {
                    output.push(ctx.quote(value.unwrap_or("")));
                }
                ArgReq::None => {}
            }
            false
        }
        _ => {
            if !ctx.quiet {
                let names: Vec<_> =
                    matches.iter().map(|o| format!("'--{}'", o.name)).collect();
                eprintln!(
                    "{}: option '--{name}' is ambiguous; possibilities: {}",
                    ctx.prog_name,
                    names.join(" ")
                );
            }
            true
        }
    }
}

fn parse_user_args(
    short_opts: &[ShortOpt],
    ctx: &Ctx<'_>,
    scan_mode: ScanMode,
    alternative: bool,
    args: &[String],
) -> (Vec<String>, bool) {
    let mut output: Vec<String> = Vec::new();
    let mut non_opts: Vec<String> = Vec::new();
    let mut errors = false;
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg == "--" {
            non_opts.extend_from_slice(&args[i + 1..]);
            break;
        }

        if arg.starts_with("--") && arg.len() > 2 {
            let rest = &arg[2..];
            let (name, value) = match rest.split_once('=') {
                Some((n, v)) => (n, Some(v)),
                None => (rest, None),
            };
            if handle_long_option(name, value, ctx, args, &mut i, &mut output) {
                errors = true;
            }
        } else if arg.starts_with('-') && arg.len() > 1 {
            let rest = &arg[1..];

            // Try alternative long option (single - prefix)
            if alternative && rest.len() > 1 && !rest.starts_with('-') {
                let (name, value) = match rest.split_once('=') {
                    Some((n, v)) => (n, Some(v)),
                    None => (rest, None),
                };
                let matches: Vec<_> = ctx
                    .long_opts
                    .iter()
                    .filter(|o| o.name.starts_with(name))
                    .collect();
                if matches.len() == 1 {
                    if handle_long_option(
                        name,
                        value,
                        ctx,
                        args,
                        &mut i,
                        &mut output,
                    ) {
                        errors = true;
                    }
                    i += 1;
                    continue;
                }
            }

            // Short options
            let chars: Vec<char> = rest.chars().collect();
            let mut j = 0;
            while j < chars.len() {
                let ch = chars[j];
                if let Some(opt) = short_opts.iter().find(|o| o.ch == ch) {
                    output.push(format!("-{ch}"));
                    match opt.arg_req {
                        ArgReq::Required => {
                            if j + 1 < chars.len() {
                                let val: String =
                                    chars[j + 1..].iter().collect();
                                output.push(ctx.quote(&val));
                                j = chars.len();
                            } else {
                                i += 1;
                                if i < args.len() {
                                    output.push(ctx.quote(&args[i]));
                                } else {
                                    if !ctx.quiet {
                                        eprintln!(
                                            "{}: option requires an argument -- '{ch}'",
                                            ctx.prog_name
                                        );
                                    }
                                    errors = true;
                                }
                            }
                        }
                        ArgReq::Optional => {
                            if j + 1 < chars.len() {
                                let val: String =
                                    chars[j + 1..].iter().collect();
                                output.push(ctx.quote(&val));
                                j = chars.len();
                            } else {
                                output.push(ctx.quote(""));
                            }
                        }
                        ArgReq::None => {}
                    }
                } else {
                    if !ctx.quiet {
                        eprintln!(
                            "{}: invalid option -- '{ch}'",
                            ctx.prog_name
                        );
                    }
                    errors = true;
                }
                j += 1;
            }
        } else {
            match scan_mode {
                ScanMode::StopAtFirst => {
                    non_opts.push(arg.clone());
                    non_opts.extend(args[i + 1..].iter().cloned());
                    break;
                }
                ScanMode::InPlace => {
                    output.push(ctx.quote(arg));
                }
                ScanMode::Permute => {
                    non_opts.push(arg.clone());
                }
            }
        }

        i += 1;
    }

    output.push("--".to_string());
    for no in &non_opts {
        output.push(ctx.quote(no));
    }

    (output, errors)
}

pub fn run(args: Args) -> ExitCode {
    if args.test {
        return ExitCode::from(4);
    }

    let shell = match args.shell.as_str() {
        "sh" | "bash" => Shell::Sh,
        "csh" | "tcsh" => Shell::Csh,
        other => {
            eprintln!("getopt: unknown shell: {other}");
            return ExitCode::from(2);
        }
    };

    let (optstring, user_args) = if let Some(ref opts) = args.options {
        (opts.as_str(), args.params.as_slice())
    } else if !args.params.is_empty() {
        (args.params[0].as_str(), &args.params[1..])
    } else {
        eprintln!("getopt: missing optstring argument");
        return ExitCode::from(2);
    };

    let (short_opts, scan_mode) = parse_optstring(optstring);

    let scan_mode = if std::env::var("POSIXLY_CORRECT").is_ok() {
        ScanMode::StopAtFirst
    } else {
        scan_mode
    };

    let long_opts: Vec<LongOpt> = args
        .longoptions
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| parse_longopt_spec(s))
        .collect();

    let ctx = Ctx {
        long_opts: &long_opts,
        prog_name: args.name.as_deref().unwrap_or("getopt"),
        quiet: args.quiet,
        shell,
        unquoted: args.unquoted,
    };

    let (output, errors) = parse_user_args(
        &short_opts,
        &ctx,
        scan_mode,
        args.alternative,
        user_args,
    );

    if !args.quiet_output {
        println!(" {}", output.join(" "));
    }

    if errors {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
