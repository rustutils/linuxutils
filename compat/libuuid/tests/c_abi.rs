use std::{path::PathBuf, process::Command, sync::OnceLock};

/// Build `linuxutils-libuuid` once per test process. The package is
/// cdylib-only and the integration test doesn't reference its Rust
/// API, so cargo skips building it as a test prerequisite — we have
/// to ask for the artifact ourselves.
fn ensure_libuuid_built() {
    static BUILT: OnceLock<()> = OnceLock::new();
    BUILT.get_or_init(|| {
        let mut cmd = Command::new(env!("CARGO"));
        cmd.args(["build", "-p", "linuxutils-libuuid"]);
        // Match the profile the tests are running under so we look
        // in the same target subdirectory afterwards.
        if !cfg!(debug_assertions) {
            cmd.arg("--release");
        }
        let status = cmd.status().expect("failed to invoke cargo build");
        assert!(status.success(), "cargo build -p linuxutils-libuuid failed");
    });
}

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
    ensure_libuuid_built();

    // cargo sets OUT_DIR for build scripts but not integration tests.
    // The cdylib lands in target/<profile>/deps/ or target/<profile>/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();

    // Match the profile our test binary was compiled under.
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let dir = workspace_root.join("target").join(profile);
    if dir.join("libuuid.so").exists() {
        return dir;
    }
    panic!(
        "libuuid.so not found in {} after cargo build — \
         build of linuxutils-libuuid did not produce the expected artifact",
        dir.display()
    );
}

fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("include")
}

#[test]
fn generate_and_parse_roundtrip() {
    let output = run_c_test(include_str!("c/generate_and_parse_roundtrip.c"));
    assert_eq!(output.trim(), "OK");
}

#[test]
fn clear_and_is_null() {
    let output = run_c_test(include_str!("c/clear_and_is_null.c"));
    assert_eq!(output.trim(), "OK");
}

#[test]
fn type_and_variant() {
    let output = run_c_test(include_str!("c/type_and_variant.c"));
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "random: type=4 variant=1");
    assert_eq!(lines[1], "time: type=1 variant=1");
}

#[test]
fn copy_and_compare() {
    let output = run_c_test(include_str!("c/copy_and_compare.c"));
    assert_eq!(output.trim(), "OK");
}

#[test]
fn unparse_upper_and_lower() {
    let output = run_c_test(include_str!("c/unparse_upper_and_lower.c"));
    assert!(output.contains("lower: 550e8400-e29b-41d4-a716-446655440000"));
    assert!(output.contains("upper: 550E8400-E29B-41D4-A716-446655440000"));
}

#[test]
fn templates() {
    let output = run_c_test(include_str!("c/templates.c"));
    assert!(output.contains("dns: 6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
    assert!(output.contains("OK"));
}

#[test]
fn name_based_deterministic() {
    let output = run_c_test(include_str!("c/name_based_deterministic.c"));
    assert!(output.contains("OK"));
}
