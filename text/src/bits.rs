use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::{ArgAction, Parser};
use std::{
    io::{self, BufRead},
    process::ExitCode,
};

const DEFAULT_WIDTH: usize = 8192;

/// Convert bit masks from/to various formats.
///
/// Each positional argument is a "group" of bits in one of the
/// recognised input formats (hex `0x...`, grouped hex `,xxxxxxxx,...`,
/// or list `1,3,5-10`). Groups are combined left-to-right, OR'ing by
/// default; a leading `&`, `^`, or `~` selects AND, XOR, or ANDN
/// instead. With no positional arguments, groups are read from stdin
/// one per line.
#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(name = "bits", about = "Convert bit masks from/to various formats")]
pub struct Args {
    /// Print result as a hexadecimal mask (default)
    #[arg(short = 'm', long, action = ArgAction::SetTrue, group = "mode")]
    mask: bool,

    /// Print result as 32-bit comma-separated hex groups
    #[arg(short = 'g', long, action = ArgAction::SetTrue, group = "mode")]
    grouped_mask: bool,

    /// Print result as a binary mask
    #[arg(short = 'b', long, action = ArgAction::SetTrue, group = "mode")]
    binary: bool,

    /// Print result as a list of bit IDs (with ranges)
    #[arg(short = 'l', long, action = ArgAction::SetTrue, group = "mode")]
    list: bool,

    /// Maximum number of bits handled; bits above this are truncated
    #[arg(short = 'w', long, default_value_t = DEFAULT_WIDTH)]
    width: usize,

    /// Mask or list groups (read from stdin if none given)
    masks: Vec<String>,
}

#[derive(Copy, Clone)]
enum Op {
    Or,
    And,
    Xor,
    AndNot,
}

#[derive(Copy, Clone)]
enum Mode {
    Mask,
    Grouped,
    Binary,
    List,
}

fn select_mode(a: &Args) -> Mode {
    if a.list {
        Mode::List
    } else if a.binary {
        Mode::Binary
    } else if a.grouped_mask {
        Mode::Grouped
    } else {
        Mode::Mask
    }
}

fn set_bit(buf: &mut Vec<u32>, bit: usize, width: usize) {
    if bit >= width {
        return;
    }
    let idx = bit / 32;
    let off = bit % 32;
    if idx >= buf.len() {
        buf.resize(idx + 1, 0);
    }
    buf[idx] |= 1u32 << off;
}

fn parse_hex(s: &str, buf: &mut Vec<u32>, width: usize) -> Result<(), String> {
    if s.is_empty() {
        return Err("empty hex value".into());
    }
    let mut bit: i64 = (s.len() as i64) * 4 - 1;
    for c in s.chars() {
        let d = c
            .to_digit(16)
            .ok_or_else(|| format!("invalid hex digit: {c}"))?;
        for shift in (0..4).rev() {
            if (d >> shift) & 1 == 1 && bit >= 0 {
                set_bit(buf, bit as usize, width);
            }
            bit -= 1;
        }
    }
    Ok(())
}

fn parse_grouped_hex(
    s: &str,
    buf: &mut Vec<u32>,
    width: usize,
) -> Result<(), String> {
    let groups: Vec<&str> = s.split(',').collect();
    let n = groups.len();
    for (i, g) in groups.iter().enumerate() {
        if g.is_empty() {
            return Err(format!("empty group in '{s}'"));
        }
        let v = u32::from_str_radix(g, 16)
            .map_err(|_| format!("invalid hex group: {g}"))?;
        let base = (n - 1 - i) * 32;
        for b in 0..32 {
            if (v >> b) & 1 == 1 {
                set_bit(buf, base + b, width);
            }
        }
    }
    Ok(())
}

fn parse_list(s: &str, buf: &mut Vec<u32>, width: usize) -> Result<(), String> {
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((lo, hi)) = part.split_once('-') {
            let lo: usize = lo
                .trim()
                .parse()
                .map_err(|_| format!("invalid bit id: {lo}"))?;
            let hi: usize = hi
                .trim()
                .parse()
                .map_err(|_| format!("invalid bit id: {hi}"))?;
            if lo > hi {
                return Err(format!("inverted range: {lo}-{hi}"));
            }
            for b in lo..=hi {
                set_bit(buf, b, width);
            }
        } else {
            let b: usize = part
                .parse()
                .map_err(|_| format!("invalid bit id: {part}"))?;
            set_bit(buf, b, width);
        }
    }
    Ok(())
}

fn parse_group(s: &str, width: usize) -> Result<(Op, Vec<u32>), String> {
    let (op, body) = match s.chars().next() {
        Some('&') => (Op::And, &s[1..]),
        Some('^') => (Op::Xor, &s[1..]),
        Some('~') => (Op::AndNot, &s[1..]),
        _ => (Op::Or, s),
    };
    if body.is_empty() {
        return Err("empty group".into());
    }
    let mut buf = Vec::new();
    if let Some(rest) =
        body.strip_prefix("0x").or_else(|| body.strip_prefix("0X"))
    {
        parse_hex(rest, &mut buf, width)?;
    } else if let Some(rest) = body.strip_prefix(',') {
        parse_grouped_hex(rest, &mut buf, width)?;
    } else {
        parse_list(body, &mut buf, width)?;
    }
    Ok((op, buf))
}

fn truncate_to_width(buf: &mut Vec<u32>, width: usize) {
    let max_words = width.div_ceil(32);
    if buf.len() > max_words {
        buf.truncate(max_words);
    }
    if !width.is_multiple_of(32) {
        if let Some(last) = buf.last_mut() {
            let keep = width % 32;
            *last &= (1u32 << keep) - 1;
        }
    }
}

fn combine(acc: &mut Vec<u32>, group: &[u32], op: Op, width: usize) {
    let n = acc.len().max(group.len());
    acc.resize(n, 0);
    for (i, slot) in acc.iter_mut().enumerate().take(n) {
        let g = group.get(i).copied().unwrap_or(0);
        match op {
            Op::Or => *slot |= g,
            Op::And => *slot &= g,
            Op::Xor => *slot ^= g,
            Op::AndNot => *slot &= !g,
        }
    }
    truncate_to_width(acc, width);
}

fn highest_set_bit(buf: &[u32]) -> Option<usize> {
    for (i, &w) in buf.iter().enumerate().rev() {
        if w != 0 {
            for b in (0..32).rev() {
                if (w >> b) & 1 == 1 {
                    return Some(i * 32 + b);
                }
            }
        }
    }
    None
}

fn fmt_mask(buf: &[u32]) -> String {
    if buf.iter().all(|&w| w == 0) {
        return "0x0".into();
    }
    let mut s = String::from("0x");
    let mut started = false;
    for &w in buf.iter().rev() {
        if !started {
            if w == 0 {
                continue;
            }
            s.push_str(&format!("{w:x}"));
            started = true;
        } else {
            s.push_str(&format!("{w:08x}"));
        }
    }
    s
}

fn fmt_grouped(buf: &[u32]) -> String {
    if buf.iter().all(|&w| w == 0) {
        return "0".into();
    }
    let mut groups: Vec<String> = Vec::new();
    let mut started = false;
    for &w in buf.iter().rev() {
        if !started {
            if w == 0 {
                continue;
            }
            groups.push(format!("{w:x}"));
            started = true;
        } else {
            groups.push(format!("{w:08x}"));
        }
    }
    groups.join(",")
}

fn fmt_binary(buf: &[u32]) -> String {
    let max_bit = match highest_set_bit(buf) {
        Some(b) => b,
        None => return "0b0".into(),
    };
    let mut s = String::from("0b");
    for i in (0..=max_bit).rev() {
        let idx = i / 32;
        let off = i % 32;
        let bit = (buf[idx] >> off) & 1;
        s.push(if bit == 1 { '1' } else { '0' });
        if i != 0 && i % 4 == 0 {
            s.push('_');
        }
    }
    s
}

fn fmt_list(buf: &[u32]) -> String {
    let mut bits: Vec<usize> = Vec::new();
    for (i, &w) in buf.iter().enumerate() {
        for b in 0..32 {
            if (w >> b) & 1 == 1 {
                bits.push(i * 32 + b);
            }
        }
    }
    if bits.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    let mut start = bits[0];
    let mut prev = bits[0];
    // Match upstream: only compress runs of 3 or more consecutive bits.
    let push_run = |out: &mut String, start: usize, prev: usize| {
        let push_one = |out: &mut String, n: usize| {
            if !out.is_empty() {
                out.push(',');
            }
            out.push_str(&n.to_string());
        };
        if prev - start >= 2 {
            if !out.is_empty() {
                out.push(',');
            }
            out.push_str(&format!("{start}-{prev}"));
        } else {
            for n in start..=prev {
                push_one(out, n);
            }
        }
    };
    for &b in &bits[1..] {
        if b == prev + 1 {
            prev = b;
        } else {
            push_run(&mut out, start, prev);
            start = b;
            prev = b;
        }
    }
    push_run(&mut out, start, prev);
    out
}

fn render(buf: &[u32], mode: Mode) -> String {
    match mode {
        Mode::Mask => fmt_mask(buf),
        Mode::Grouped => fmt_grouped(buf),
        Mode::Binary => fmt_binary(buf),
        Mode::List => fmt_list(buf),
    }
}

fn collect_groups(args: &Args) -> Result<Vec<String>, String> {
    if !args.masks.is_empty() {
        return Ok(args.masks.clone());
    }
    let stdin = io::stdin();
    let mut groups = Vec::new();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| format!("stdin: {e}"))?;
        let line = line.trim().to_string();
        if !line.is_empty() {
            groups.push(line);
        }
    }
    Ok(groups)
}

pub fn run(args: Args) -> ExitCode {
    if args.width == 0 {
        eprintln!("bits: --width must be greater than 0");
        return ExitCode::FAILURE;
    }
    let mode = select_mode(&args);

    let groups = match collect_groups(&args) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("bits: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut acc: Vec<u32> = Vec::new();
    for g in &groups {
        let (op, group) = match parse_group(g, args.width) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("bits: '{g}': {e}");
                return ExitCode::FAILURE;
            }
        };
        combine(&mut acc, &group, op, args.width);
    }

    println!("{}", render(&acc, mode));
    ExitCode::SUCCESS
}
