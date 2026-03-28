use std::{
    fs::File,
    io::{self, Read},
};

/// Generate a 128-bit random hexadecimal cookie.
///
/// If `extra_file` is provided, its contents (up to `max_size` bytes) are
/// mixed into the randomness by XORing with the random bytes.
pub fn mcookie(
    extra_file: Option<&mut dyn Read>,
    max_size: Option<u64>,
    verbose: bool,
) -> io::Result<String> {
    let mut bytes = [0u8; 16];

    // Primary source: getrandom / /dev/urandom
    getrandom(&mut bytes)?;
    if verbose {
        eprintln!("Got 16 bytes from getrandom/urandom");
    }

    // Optional extra file source
    if let Some(reader) = extra_file {
        let mut extra = Vec::new();
        match max_size {
            Some(max) => {
                reader.take(max).read_to_end(&mut extra)?;
            }
            None => {
                reader.read_to_end(&mut extra)?;
            }
        }
        if verbose {
            eprintln!("Got {} bytes from file", extra.len());
        }
        for (i, &b) in extra.iter().enumerate() {
            bytes[i % 16] ^= b;
        }
    }

    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn getrandom(buf: &mut [u8]) -> io::Result<()> {
    let mut f = File::open("/dev/urandom")?;
    f.read_exact(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_32_hex_chars() {
        let cookie = mcookie(None, None, false).unwrap();
        assert_eq!(cookie.len(), 32);
        assert!(cookie.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn two_cookies_differ() {
        let a = mcookie(None, None, false).unwrap();
        let b = mcookie(None, None, false).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn extra_file_mixed_in() {
        let mut extra = &b"extra randomness here"[..];
        let cookie = mcookie(Some(&mut extra), None, false).unwrap();
        assert_eq!(cookie.len(), 32);
        assert!(cookie.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn max_size_limits_read() {
        let mut extra = &b"extra randomness here"[..];
        let cookie = mcookie(Some(&mut extra), Some(4), false).unwrap();
        assert_eq!(cookie.len(), 32);
        assert!(cookie.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
