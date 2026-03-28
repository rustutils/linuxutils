use std::{
    io::{self, Write},
    path::Path,
};

const DEFAULT_MESSAGE: &str = "This account is currently not available.";
const NOLOGIN_TXT: &str = "/etc/nologin.txt";

/// Print the nologin message to `writer`.
///
/// If `/etc/nologin.txt` exists, its contents are used. Otherwise, the default
/// message is printed.
pub fn nologin(writer: &mut impl Write) -> io::Result<()> {
    let path = Path::new(NOLOGIN_TXT);
    if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        write!(writer, "{contents}")?;
    } else {
        writeln!(writer, "{DEFAULT_MESSAGE}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_message() {
        let mut buf = Vec::new();
        // This test will use the default message unless /etc/nologin.txt
        // happens to exist on the test machine, which is fine either way.
        nologin(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(!output.is_empty());
    }
}
