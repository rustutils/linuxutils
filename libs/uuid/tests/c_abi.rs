use std::path::PathBuf;
use std::process::Command;

/// Build a C test program, link it against our libuuid.so, run it, and
/// return its stdout.
fn run_c_test(c_source: &str) -> String {
    let tmp = tempfile::tempdir().unwrap();

    // Write the C source to a file.
    let c_path = tmp.path().join("test.c");
    std::fs::write(&c_path, c_source).unwrap();

    // Find the built libuuid.so.
    let lib_dir = find_lib_dir();

    // Compile.
    let bin_path = tmp.path().join("test");
    let compile = Command::new("cc")
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            &format!("-L{}", lib_dir.display()),
            "-luuid",
            &format!("-I{}", include_dir().display()),
            "-Wl,-rpath",
            &format!("-Wl,{}", lib_dir.display()),
        ])
        .output()
        .expect("failed to run cc");

    assert!(
        compile.status.success(),
        "C compilation failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    // Run.
    let run = Command::new(&bin_path)
        .env("LD_LIBRARY_PATH", &lib_dir)
        .output()
        .expect("failed to run test binary");

    assert!(
        run.status.success(),
        "Test binary failed (exit {}):\nstdout: {}\nstderr: {}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );

    String::from_utf8(run.stdout).unwrap()
}

fn find_lib_dir() -> PathBuf {
    // cargo sets OUT_DIR for build scripts but not integration tests.
    // The cdylib lands in target/<profile>/deps/ or target/<profile>/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();

    // Try debug first, then release.
    for profile in ["debug", "release"] {
        let dir = workspace_root.join("target").join(profile);
        if dir.join("libuuid.so").exists() {
            return dir;
        }
    }
    panic!(
        "libuuid.so not found — run `cargo build -p rustutils-libuuid` first"
    );
}

fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("include")
}

#[test]
fn generate_and_parse_roundtrip() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include <string.h>
#include "uuid.h"

int main() {
    uuid_t u1, u2;
    char buf[UUID_STR_LEN];

    uuid_generate_random(u1);
    uuid_unparse(u1, buf);

    // Must be 36 chars (8-4-4-4-12).
    if (strlen(buf) != 36) {
        fprintf(stderr, "bad length: %zu\n", strlen(buf));
        return 1;
    }

    // Parse it back and compare.
    if (uuid_parse(buf, u2) != 0) {
        fprintf(stderr, "parse failed\n");
        return 1;
    }
    if (uuid_compare(u1, u2) != 0) {
        fprintf(stderr, "roundtrip mismatch\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
"#,
    );
    assert_eq!(output.trim(), "OK");
}

#[test]
fn clear_and_is_null() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u;
    uuid_generate(u);

    if (uuid_is_null(u)) {
        fprintf(stderr, "generated uuid should not be null\n");
        return 1;
    }

    uuid_clear(u);

    if (!uuid_is_null(u)) {
        fprintf(stderr, "cleared uuid should be null\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
"#,
    );
    assert_eq!(output.trim(), "OK");
}

#[test]
fn type_and_variant() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u;

    uuid_generate_random(u);
    printf("random: type=%d variant=%d\n", uuid_type(u), uuid_variant(u));

    uuid_generate_time(u);
    printf("time: type=%d variant=%d\n", uuid_type(u), uuid_variant(u));

    return 0;
}
"#,
    );
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "random: type=4 variant=1");
    assert_eq!(lines[1], "time: type=1 variant=1");
}

#[test]
fn copy_and_compare() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u1, u2;
    uuid_generate(u1);
    uuid_copy(u2, u1);

    if (uuid_compare(u1, u2) != 0) {
        fprintf(stderr, "copy should be equal\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
"#,
    );
    assert_eq!(output.trim(), "OK");
}

#[test]
fn unparse_upper_and_lower() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include <ctype.h>
#include <string.h>
#include "uuid.h"

int main() {
    uuid_t u;
    char lower[UUID_STR_LEN], upper[UUID_STR_LEN];

    uuid_parse("550e8400-e29b-41d4-a716-446655440000", u);
    uuid_unparse_lower(u, lower);
    uuid_unparse_upper(u, upper);

    printf("lower: %s\n", lower);
    printf("upper: %s\n", upper);

    // Verify case.
    for (int i = 0; i < 36; i++) {
        if (isalpha(lower[i]) && !islower(lower[i])) return 1;
        if (isalpha(upper[i]) && !isupper(upper[i])) return 1;
    }

    return 0;
}
"#,
    );
    assert!(output.contains("lower: 550e8400-e29b-41d4-a716-446655440000"));
    assert!(output.contains("upper: 550E8400-E29B-41D4-A716-446655440000"));
}

#[test]
fn templates() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include <stdlib.h>
#include "uuid.h"

int main() {
    const uuid_t *dns = uuid_get_template("dns");
    const uuid_t *nil = uuid_get_template("nil");
    const uuid_t *bad = uuid_get_template("nonexistent");

    if (dns == NULL) { fprintf(stderr, "dns is null\n"); return 1; }
    if (nil == NULL) { fprintf(stderr, "nil is null\n"); return 1; }
    if (bad != NULL) { fprintf(stderr, "bad should be null\n"); return 1; }

    char buf[UUID_STR_LEN];
    uuid_unparse_lower(*dns, buf);
    printf("dns: %s\n", buf);

    if (!uuid_is_null(*nil)) {
        fprintf(stderr, "nil template should be null\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
"#,
    );
    assert!(output.contains("dns: 6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
    assert!(output.contains("OK"));
}

#[test]
fn name_based_deterministic() {
    let output = run_c_test(
        r#"
#include <stdio.h>
#include <string.h>
#include "uuid.h"

int main() {
    const uuid_t *dns = uuid_get_template("dns");
    uuid_t u1, u2;
    char buf1[UUID_STR_LEN], buf2[UUID_STR_LEN];

    // v5 (SHA1) should be deterministic.
    uuid_generate_sha1(u1, *dns, "example.com", 11);
    uuid_generate_sha1(u2, *dns, "example.com", 11);

    uuid_unparse(u1, buf1);
    uuid_unparse(u2, buf2);

    printf("%s\n", buf1);

    if (strcmp(buf1, buf2) != 0) {
        fprintf(stderr, "SHA1 not deterministic\n");
        return 1;
    }
    if (uuid_type(u1) != 5) {
        fprintf(stderr, "expected type 5, got %d\n", uuid_type(u1));
        return 1;
    }

    printf("OK\n");
    return 0;
}
"#,
    );
    assert!(output.contains("OK"));
}
