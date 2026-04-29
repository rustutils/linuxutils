use linuxutils_common::man::ManContent;

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    io::{self, Read, Write},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "hexdump",
    about = "Display file contents in hexadecimal, decimal, octal, or ascii"
)]
pub struct Args {
    /// One-byte octal display
    #[arg(short = 'b', long = "one-byte-octal")]
    one_byte_octal: bool,

    /// One-byte hex display
    #[arg(short = 'X', long = "one-byte-hex")]
    one_byte_hex: bool,

    /// One-byte character display
    #[arg(short = 'c', long = "one-byte-char")]
    one_byte_char: bool,

    /// Canonical hex+ASCII display
    #[arg(short = 'C', long)]
    canonical: bool,

    /// Two-byte decimal display
    #[arg(short = 'd', long = "two-bytes-decimal")]
    two_bytes_decimal: bool,

    /// Two-byte octal display
    #[arg(short = 'o', long = "two-bytes-octal")]
    two_bytes_octal: bool,

    /// Two-byte hexadecimal display
    #[arg(short = 'x', long = "two-bytes-hex")]
    two_bytes_hex: bool,

    /// Interpret only length bytes of input
    #[arg(short = 'n', long)]
    length: Option<u64>,

    /// Skip offset bytes from the beginning
    #[arg(short, long)]
    skip: Option<u64>,

    /// Display all data (don't squeeze identical lines)
    #[arg(short = 'v', long = "no-squeezing")]
    no_squeezing: bool,

    /// Files to display
    #[arg()]
    files: Vec<String>,
}

#[derive(Clone, Copy)]
enum Mode {
    Default,
    Canonical,
    OneByteOctal,
    OneByteHex,
    OneByteChar,
    TwoBytesDecimal,
    TwoBytesOctal,
    TwoBytesHex,
}

fn parse_suffix_value(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Try with hex prefix.
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).ok();
    }
    // Try with suffix.
    let (num, mult) = if let Some(n) =
        s.strip_suffix("KiB").or_else(|| s.strip_suffix("K"))
    {
        (n, 1024u64)
    } else if let Some(n) =
        s.strip_suffix("MiB").or_else(|| s.strip_suffix("M"))
    {
        (n, 1024 * 1024)
    } else if let Some(n) =
        s.strip_suffix("GiB").or_else(|| s.strip_suffix("G"))
    {
        (n, 1024 * 1024 * 1024)
    } else {
        (s, 1)
    };
    num.trim().parse::<u64>().ok().map(|n| n * mult)
}

struct InputReader {
    sources: Vec<Box<dyn Read>>,
    current: usize,
    remaining: Option<u64>,
}

impl InputReader {
    fn new(
        files: &[String],
        skip: Option<u64>,
        length: Option<u64>,
    ) -> io::Result<Self> {
        let mut sources: Vec<Box<dyn Read>> = if files.is_empty() {
            vec![Box::new(io::stdin())]
        } else {
            let mut v: Vec<Box<dyn Read>> = Vec::new();
            for f in files {
                v.push(Box::new(std::fs::File::open(f)?));
            }
            v
        };

        // Skip bytes.
        if let Some(skip) = skip {
            let mut to_skip = skip;
            let mut buf = [0u8; 8192];
            while to_skip > 0 && !sources.is_empty() {
                let n = (to_skip as usize).min(buf.len());
                match sources[0].read(&mut buf[..n]) {
                    Ok(0) => {
                        sources.remove(0);
                    }
                    Ok(read) => to_skip -= read as u64,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(Self {
            sources,
            current: 0,
            remaining: length,
        })
    }

    fn read_exact_or_eof(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let max = if let Some(rem) = self.remaining {
            buf.len().min(rem as usize)
        } else {
            buf.len()
        };

        if max == 0 {
            return Ok(0);
        }

        let mut total = 0;
        while total < max && self.current < self.sources.len() {
            match self.sources[self.current].read(&mut buf[total..max]) {
                Ok(0) => self.current += 1,
                Ok(n) => total += n,
                Err(e) => return Err(e),
            }
        }

        if let Some(ref mut rem) = self.remaining {
            *rem -= total as u64;
        }
        Ok(total)
    }
}

fn format_canonical(
    data: &[u8],
    offset: u64,
    out: &mut dyn Write,
) -> io::Result<()> {
    write!(out, "{offset:08x}  ")?;
    for i in 0..16 {
        if i < data.len() {
            write!(out, "{:02x} ", data[i])?;
        } else {
            write!(out, "   ")?;
        }
        if i == 7 {
            write!(out, " ")?;
        }
    }
    write!(out, " |")?;
    for &c in data.iter().take(16) {
        if c.is_ascii_graphic() || c == b' ' {
            write!(out, "{}", c as char)?;
        } else {
            write!(out, ".")?;
        }
    }
    writeln!(out, "|")
}

fn format_one_byte_octal(
    data: &[u8],
    offset: u64,
    out: &mut dyn Write,
) -> io::Result<()> {
    write!(out, "{offset:07x}")?;
    for &b in data.iter().take(16) {
        write!(out, " {:03o}", b)?;
    }
    writeln!(out)
}

fn format_one_byte_hex(
    data: &[u8],
    offset: u64,
    out: &mut dyn Write,
) -> io::Result<()> {
    write!(out, "{offset:07x}")?;
    for &b in data.iter().take(16) {
        write!(out, " {:02x}", b)?;
    }
    writeln!(out)
}

fn format_one_byte_char(
    data: &[u8],
    offset: u64,
    out: &mut dyn Write,
) -> io::Result<()> {
    write!(out, "{offset:07x}")?;
    for &b in data.iter().take(16) {
        let s = match b {
            0 => " \\0".to_string(),
            7 => " \\a".to_string(),
            8 => " \\b".to_string(),
            9 => " \\t".to_string(),
            10 => " \\n".to_string(),
            11 => " \\v".to_string(),
            12 => " \\f".to_string(),
            13 => " \\r".to_string(),
            b if b.is_ascii_graphic() || b == b' ' => {
                format!("   {}", b as char)
            }
            b => format!(" {:03o}", b),
        };
        write!(out, "{s}")?;
    }
    writeln!(out)
}

fn format_two_bytes(
    data: &[u8],
    offset: u64,
    out: &mut dyn Write,
    offset_fmt: OffsetFmt,
    value_fmt: &str,
) -> io::Result<()> {
    match offset_fmt {
        OffsetFmt::Hex7 => write!(out, "{offset:07x}")?,
    }
    let mut i = 0;
    while i + 1 < data.len() && i < 16 {
        let val = u16::from_le_bytes([data[i], data[i + 1]]);
        match value_fmt {
            "x4" => write!(out, "    {:04x}", val)?,
            "x4d" => write!(out, " {:04x}", val)?, // default: tighter
            "d5" => write!(out, "   {:05}", val)?,
            "o6" => write!(out, "  {:06o}", val)?,
            _ => write!(out, " {:04x}", val)?,
        }
        i += 2;
    }
    if i < data.len() && i < 16 {
        let val = data[i] as u16;
        match value_fmt {
            "x4" | "x4d" => write!(out, "    {:04x}", val)?,
            "d5" => write!(out, "   {:05}", val)?,
            "o6" => write!(out, "  {:06o}", val)?,
            _ => write!(out, " {:04x}", val)?,
        }
    }
    // Pad to standard width for short final lines.
    writeln!(out)
}

#[derive(Clone, Copy)]
enum OffsetFmt {
    Hex7,
}

pub fn run(args: Args) -> ExitCode {
    let mode = if args.canonical {
        Mode::Canonical
    } else if args.one_byte_octal {
        Mode::OneByteOctal
    } else if args.one_byte_hex {
        Mode::OneByteHex
    } else if args.one_byte_char {
        Mode::OneByteChar
    } else if args.two_bytes_decimal {
        Mode::TwoBytesDecimal
    } else if args.two_bytes_octal {
        Mode::TwoBytesOctal
    } else if args.two_bytes_hex {
        Mode::TwoBytesHex
    } else {
        Mode::Default
    };

    let skip = args
        .skip
        .or_else(|| args.skip.and_then(|s| parse_suffix_value(&s.to_string())));

    let mut reader = match InputReader::new(&args.files, skip, args.length) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("hexdump: {e}");
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut offset = skip.unwrap_or(0);
    let mut buf = [0u8; 16];
    let mut prev_line: Option<Vec<u8>> = None;
    let mut squeezed = false;

    let format_line =
        |data: &[u8], offset: u64, out: &mut dyn Write| -> io::Result<()> {
            match mode {
                Mode::Canonical => format_canonical(data, offset, out),
                Mode::OneByteOctal => format_one_byte_octal(data, offset, out),
                Mode::OneByteHex => format_one_byte_hex(data, offset, out),
                Mode::OneByteChar => format_one_byte_char(data, offset, out),
                Mode::TwoBytesHex => {
                    format_two_bytes(data, offset, out, OffsetFmt::Hex7, "x4")
                }
                Mode::TwoBytesDecimal => {
                    format_two_bytes(data, offset, out, OffsetFmt::Hex7, "d5")
                }
                Mode::TwoBytesOctal => {
                    format_two_bytes(data, offset, out, OffsetFmt::Hex7, "o6")
                }
                Mode::Default => {
                    format_two_bytes(data, offset, out, OffsetFmt::Hex7, "x4d")
                }
            }
        };

    loop {
        let n = match reader.read_exact_or_eof(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("hexdump: {e}");
                return ExitCode::FAILURE;
            }
        };

        if n == 0 {
            break;
        }

        let line = &buf[..n];

        // Squeezing: skip identical lines.
        if !args.no_squeezing
            && let Some(ref prev) = prev_line
            && prev == line
            && n == 16
        {
            if !squeezed {
                let _ = writeln!(out, "*");
                squeezed = true;
            }
            offset += n as u64;
            continue;
        }
        squeezed = false;
        prev_line = Some(line.to_vec());

        if let Err(e) = format_line(line, offset, &mut out) {
            eprintln!("hexdump: {e}");
            return ExitCode::FAILURE;
        }

        offset += n as u64;
    }

    // Print final offset.
    let _ = match mode {
        Mode::Canonical => writeln!(out, "{offset:08x}"),
        _ => writeln!(out, "{offset:07x}"),
    };

    ExitCode::SUCCESS
}
