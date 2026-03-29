use std::ffi::CStr;
use std::fmt::Write as _;
use uuid::Uuid;

// uuid_t in C is unsigned char[16], so we work with *mut u8 / *const u8.

/// Zero out a UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_clear(uu: *mut u8) {
    unsafe { std::ptr::write_bytes(uu, 0, 16) };
}

/// Compare two UUIDs. Returns -1, 0, or 1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_compare(uu1: *const u8, uu2: *const u8) -> i32 {
    let a = unsafe { std::slice::from_raw_parts(uu1, 16) };
    let b = unsafe { std::slice::from_raw_parts(uu2, 16) };
    a.cmp(b) as i32
}

/// Copy a UUID from src to dst.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_copy(dst: *mut u8, src: *const u8) {
    unsafe { std::ptr::copy_nonoverlapping(src, dst, 16) };
}

/// Generate a UUID (random/v4).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate(out: *mut u8) {
    unsafe { uuid_generate_random(out) };
}

/// Generate a random (v4) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_random(out: *mut u8) {
    let u = Uuid::new_v4();
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Generate a time-based (v1) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_time(out: *mut u8) {
    let u = generate_v1();
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Generate a time-based (v1) UUID.
/// Returns 0 if the UUID is globally unique (MAC-based), -1 if not (random node).
/// We always use a random node ID, so we always return -1.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_time_safe(out: *mut u8) -> i32 {
    unsafe { uuid_generate_time(out) };
    -1
}

/// Generate a reordered time-based (v6) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_time_v6(out: *mut u8) {
    let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
    let node_id = random_node_id();
    let u = Uuid::new_v6(ts, &node_id);
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Generate a Unix epoch time-based (v7) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_time_v7(out: *mut u8) {
    let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
    let u = Uuid::new_v7(ts);
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Generate a name-based MD5 (v3) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_md5(
    out: *mut u8,
    ns: *const u8,
    name: *const i8,
    len: usize,
) {
    let ns_bytes: [u8; 16] = unsafe { std::slice::from_raw_parts(ns, 16) }
        .try_into()
        .unwrap();
    let ns = Uuid::from_bytes(ns_bytes);
    let name = unsafe { std::slice::from_raw_parts(name as *const u8, len) };
    let u = Uuid::new_v3(&ns, name);
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Generate a name-based SHA1 (v5) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_generate_sha1(
    out: *mut u8,
    ns: *const u8,
    name: *const i8,
    len: usize,
) {
    let ns_bytes: [u8; 16] = unsafe { std::slice::from_raw_parts(ns, 16) }
        .try_into()
        .unwrap();
    let ns = Uuid::from_bytes(ns_bytes);
    let name = unsafe { std::slice::from_raw_parts(name as *const u8, len) };
    let u = Uuid::new_v5(&ns, name);
    unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), out, 16) };
}

/// Check if a UUID is the nil (all zeros) UUID.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_is_null(uu: *const u8) -> i32 {
    let bytes = unsafe { std::slice::from_raw_parts(uu, 16) };
    if bytes.iter().all(|&b| b == 0) { 1 } else { 0 }
}

/// Parse a UUID string into a uuid_t. Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_parse(input: *const i8, uu: *mut u8) -> i32 {
    let s = match unsafe { CStr::from_ptr(input) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match Uuid::parse_str(s) {
        Ok(u) => {
            unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), uu, 16) };
            0
        }
        Err(_) => -1,
    }
}

/// Parse a UUID string from a range [in_start, in_end) into a uuid_t.
/// Returns 0 on success, -1 on failure.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_parse_range(
    in_start: *const i8,
    in_end: *const i8,
    uu: *mut u8,
) -> i32 {
    let len = unsafe { in_end.offset_from(in_start) };
    if len < 0 {
        return -1;
    }
    let bytes = unsafe { std::slice::from_raw_parts(in_start as *const u8, len as usize) };
    let s = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match Uuid::parse_str(s) {
        Ok(u) => {
            unsafe { std::ptr::copy_nonoverlapping(u.as_bytes().as_ptr(), uu, 16) };
            0
        }
        Err(_) => -1,
    }
}

/// Format a UUID as a lowercase hyphenated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_unparse(uu: *const u8, out: *mut i8) {
    unsafe { uuid_unparse_lower(uu, out) };
}

/// Format a UUID as a lowercase hyphenated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_unparse_lower(uu: *const u8, out: *mut i8) {
    let bytes: [u8; 16] = unsafe { std::slice::from_raw_parts(uu, 16) }
        .try_into()
        .unwrap();
    let u = Uuid::from_bytes(bytes);
    let mut buf = String::with_capacity(36);
    write!(buf, "{}", u.as_hyphenated()).unwrap();
    let out = unsafe { std::slice::from_raw_parts_mut(out as *mut u8, 37) };
    out[..36].copy_from_slice(buf.as_bytes());
    out[36] = 0;
}

/// Format a UUID as an uppercase hyphenated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_unparse_upper(uu: *const u8, out: *mut i8) {
    let bytes: [u8; 16] = unsafe { std::slice::from_raw_parts(uu, 16) }
        .try_into()
        .unwrap();
    let u = Uuid::from_bytes(bytes);
    let mut buf = String::with_capacity(36);
    write!(buf, "{}", u.as_hyphenated().to_string().to_uppercase()).unwrap();
    let out = unsafe { std::slice::from_raw_parts_mut(out as *mut u8, 37) };
    out[..36].copy_from_slice(buf.as_bytes());
    out[36] = 0;
}

/// Extract the time from a time-based UUID.
/// Returns the time as a time_t, and optionally fills ret_tv with the timeval.
/// Returns -1 if the UUID is not a time-based type.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_time(
    uu: *const u8,
    ret_tv: *mut libc::timeval,
) -> libc::time_t {
    let bytes: [u8; 16] = unsafe { std::slice::from_raw_parts(uu, 16) }
        .try_into()
        .unwrap();
    let u = Uuid::from_bytes(bytes);

    let (secs, nanos) = match u.get_timestamp() {
        Some(ts) => ts.to_unix(),
        None => return -1,
    };

    if !ret_tv.is_null() {
        unsafe {
            (*ret_tv).tv_sec = secs as libc::time_t;
            (*ret_tv).tv_usec = (nanos / 1000) as libc::suseconds_t;
        }
    }

    secs as libc::time_t
}

/// Return the UUID version (type) number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_type(uu: *const u8) -> i32 {
    // Version is stored in the upper nibble of byte 6.
    let byte6 = unsafe { *uu.add(6) };
    ((byte6 >> 4) & 0x0f) as i32
}

/// Return the UUID variant number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_variant(uu: *const u8) -> i32 {
    // Variant is determined by the high bits of byte 8.
    let byte8 = unsafe { *uu.add(8) };

    if (byte8 & 0x80) == 0 {
        return 0; // UUID_VARIANT_NCS
    }
    if (byte8 & 0xc0) == 0x80 {
        return 1; // UUID_VARIANT_DCE
    }
    if (byte8 & 0xe0) == 0xc0 {
        return 2; // UUID_VARIANT_MICROSOFT
    }
    3 // UUID_VARIANT_OTHER
}

/// Get a predefined UUID template by alias.
/// Known aliases: "dns", "url", "oid", "x500", "nil"
/// Returns null if the alias is unknown.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uuid_get_template(alias: *const i8) -> *const u8 {
    let s = match unsafe { CStr::from_ptr(alias) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null(),
    };

    match s {
        "dns" => &TEMPLATE_DNS as *const [u8; 16] as *const u8,
        "url" => &TEMPLATE_URL as *const [u8; 16] as *const u8,
        "oid" => &TEMPLATE_OID as *const [u8; 16] as *const u8,
        "x500" => &TEMPLATE_X500 as *const [u8; 16] as *const u8,
        "nil" => &TEMPLATE_NIL as *const [u8; 16] as *const u8,
        _ => std::ptr::null(),
    }
}

// Well-known namespace UUIDs.
static TEMPLATE_DNS: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
static TEMPLATE_URL: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x11, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
static TEMPLATE_OID: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x12, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
static TEMPLATE_X500: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x14, 0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30, 0xc8,
];
static TEMPLATE_NIL: [u8; 16] = [0u8; 16];

// --- Internal helpers ---

fn generate_v1() -> Uuid {
    let ts = uuid::timestamp::Timestamp::now(uuid::NoContext);
    let node_id = random_node_id();
    Uuid::new_v1(ts, &node_id)
}

fn random_node_id() -> [u8; 6] {
    let u = Uuid::new_v4();
    let bytes = u.as_bytes();
    let mut node = [0u8; 6];
    node.copy_from_slice(&bytes[..6]);
    // Set the multicast bit to indicate this is not a real MAC address.
    node[0] |= 0x01;
    node
}
