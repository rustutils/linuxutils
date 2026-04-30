use linuxutils_common::{HELP_WIDTH_MAX, man::ManContent};

pub const MAN: ManContent = ManContent::empty();

use clap::Parser;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    process::ExitCode,
};

const PVD_OFFSET: u64 = 0x8000;
const PVD_TYPE_PRIMARY: u8 = 1;
const CD001_MAGIC: &[u8; 5] = b"CD001";

#[derive(Parser, Debug)]
#[command(max_term_width = HELP_WIDTH_MAX)]
#[command(
    name = "isosize",
    about = "Output the length of an iso9660 filesystem",
    override_usage = "isosize [options] <iso9660_image_file> ..."
)]
pub struct Args {
    /// Show sector count and sector size
    #[arg(short = 'x', long = "sectors")]
    sectors: bool,

    /// Divisor for the output size (when not using --sectors)
    #[arg(
        short = 'd',
        long = "divisor",
        value_name = "number",
        default_value_t = 1
    )]
    divisor: u64,

    /// ISO 9660 image files
    #[arg(required = true)]
    files: Vec<String>,
}

fn isosize(path: &str) -> Result<(u32, u16), String> {
    let mut file = File::open(path)
        .map_err(|e| format!("isosize: failed to open '{}': {}", path, e))?;

    let mut pvd = [0u8; 132];
    file.seek(SeekFrom::Start(PVD_OFFSET))
        .map_err(|e| format!("isosize: failed to seek in '{}': {}", path, e))?;
    file.read_exact(&mut pvd).map_err(|e| {
        format!("isosize: failed to read from '{}': {}", path, e)
    })?;

    if pvd[0] != PVD_TYPE_PRIMARY || &pvd[1..6] != CD001_MAGIC {
        return Err(format!(
            "isosize: '{}': might not be an iso9660 filesystem",
            path
        ));
    }

    let block_count = u32::from_le_bytes([pvd[80], pvd[81], pvd[82], pvd[83]]);
    let block_size = u16::from_le_bytes([pvd[128], pvd[129]]);

    Ok((block_count, block_size))
}

pub fn run(args: Args) -> ExitCode {
    if args.divisor == 0 {
        eprintln!("isosize: invalid divisor: 0");
        return ExitCode::from(1);
    }

    let multiple = args.files.len() > 1;
    let mut failures = 0usize;

    for path in &args.files {
        match isosize(path) {
            Ok((block_count, block_size)) => {
                let prefix = if multiple {
                    format!("{}\t", path)
                } else {
                    String::new()
                };

                if args.sectors {
                    println!(
                        "{}sector count: {}, sector size: {}",
                        prefix, block_count, block_size
                    );
                } else {
                    let size = block_size as u64 * block_count as u64;
                    println!("{}{}", prefix, size / args.divisor);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                failures += 1;
            }
        }
    }

    if failures == args.files.len() {
        ExitCode::from(32)
    } else if failures > 0 {
        ExitCode::from(64)
    } else {
        ExitCode::SUCCESS
    }
}
