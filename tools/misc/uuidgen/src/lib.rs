use clap::Parser;
use std::process::ExitCode;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "uuidgen", version, about = "Create a new UUID value")]
pub struct Args {
    /// Generate a random-based UUID (v4)
    #[arg(short = 'r', long = "random")]
    random: bool,

    /// Generate a time-based UUID (v1)
    #[arg(short = 't', long = "time")]
    time: bool,

    /// Generate a time-based UUID (v6), lexicographically sortable
    #[arg(short = '6', long = "time-v6")]
    time_v6: bool,

    /// Generate a time-based UUID (v7), lexicographically sortable
    #[arg(short = '7', long = "time-v7")]
    time_v7: bool,

    /// Use MD5 as the hash algorithm (v3)
    #[arg(short = 'm', long = "md5", requires = "namespace", requires = "name")]
    md5: bool,

    /// Use SHA1 as the hash algorithm (v5)
    #[arg(short = 's', long = "sha1", requires = "namespace", requires = "name")]
    sha1: bool,

    /// Namespace UUID or alias (@dns, @url, @oid, @x500)
    #[arg(short = 'n', long = "namespace")]
    namespace: Option<String>,

    /// Name to hash
    #[arg(short = 'N', long = "name")]
    name: Option<String>,

    /// Number of UUIDs to generate
    #[arg(short = 'C', long = "count", default_value = "1")]
    count: usize,

    /// Interpret name as a hexadecimal string
    #[arg(short = 'x', long = "hex")]
    hex: bool,
}

pub fn run(args: Args) -> ExitCode {
    for _ in 0..args.count {
        let u = match generate(&args) {
            Ok(u) => u,
            Err(e) => {
                eprintln!("uuidgen: {e}");
                return ExitCode::FAILURE;
            }
        };
        println!("{}", u.as_hyphenated());
    }
    ExitCode::SUCCESS
}

fn generate(args: &Args) -> Result<Uuid, String> {
    if args.md5 || args.sha1 {
        let ns = parse_namespace(args.namespace.as_deref().unwrap())?;
        let name_str = args.name.as_deref().unwrap();
        let name_bytes = if args.hex {
            hex_decode(name_str)?
        } else {
            name_str.as_bytes().to_vec()
        };
        if args.md5 {
            Ok(Uuid::new_v3(&ns, &name_bytes))
        } else {
            Ok(Uuid::new_v5(&ns, &name_bytes))
        }
    } else if args.time {
        Ok(generate_v1())
    } else if args.time_v6 {
        let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
        let node = random_node_id();
        Ok(Uuid::new_v6(ts, &node))
    } else if args.time_v7 {
        let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
        Ok(Uuid::new_v7(ts))
    } else {
        // Default: random (v4).
        Ok(Uuid::new_v4())
    }
}

fn parse_namespace(ns: &str) -> Result<Uuid, String> {
    match ns {
        "@dns" => Ok(Uuid::NAMESPACE_DNS),
        "@url" => Ok(Uuid::NAMESPACE_URL),
        "@oid" => Ok(Uuid::NAMESPACE_OID),
        "@x500" => Ok(Uuid::NAMESPACE_X500),
        other => Uuid::parse_str(other).map_err(|e| format!("invalid namespace: {e}")),
    }
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.replace([' ', '-', ':'], "");
    if s.len() % 2 != 0 {
        return Err("hex string must have even length".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("invalid hex: {e}")))
        .collect()
}

fn generate_v1() -> Uuid {
    let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
    let node = random_node_id();
    Uuid::new_v1(ts, &node)
}

fn random_node_id() -> [u8; 6] {
    let u = Uuid::new_v4();
    let mut node = [0u8; 6];
    node.copy_from_slice(&u.as_bytes()[..6]);
    node[0] |= 0x01; // multicast bit
    node
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_generates_v4() {
        let args = Args {
            random: false, time: false, time_v6: false, time_v7: false,
            md5: false, sha1: false, namespace: None, name: None,
            count: 1, hex: false,
        };
        let u = generate(&args).unwrap();
        assert_eq!(u.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn sha1_is_deterministic() {
        let args = Args {
            random: false, time: false, time_v6: false, time_v7: false,
            md5: false, sha1: true,
            namespace: Some("@dns".to_string()),
            name: Some("example.com".to_string()),
            count: 1, hex: false,
        };
        let u1 = generate(&args).unwrap();
        let u2 = generate(&args).unwrap();
        assert_eq!(u1, u2);
        assert_eq!(u1.get_version(), Some(uuid::Version::Sha1));
    }

    #[test]
    fn md5_is_deterministic() {
        let args = Args {
            random: false, time: false, time_v6: false, time_v7: false,
            md5: true, sha1: false,
            namespace: Some("@dns".to_string()),
            name: Some("example.com".to_string()),
            count: 1, hex: false,
        };
        let u1 = generate(&args).unwrap();
        let u2 = generate(&args).unwrap();
        assert_eq!(u1, u2);
        assert_eq!(u1.get_version(), Some(uuid::Version::Md5));
    }

    #[test]
    fn namespace_aliases() {
        assert_eq!(parse_namespace("@dns").unwrap(), Uuid::NAMESPACE_DNS);
        assert_eq!(parse_namespace("@url").unwrap(), Uuid::NAMESPACE_URL);
        assert_eq!(parse_namespace("@oid").unwrap(), Uuid::NAMESPACE_OID);
        assert_eq!(parse_namespace("@x500").unwrap(), Uuid::NAMESPACE_X500);
    }

    #[test]
    fn hex_decode_works() {
        assert_eq!(hex_decode("48656c6c6f").unwrap(), b"Hello");
        assert!(hex_decode("zz").is_err());
        assert!(hex_decode("abc").is_err());
    }
}
