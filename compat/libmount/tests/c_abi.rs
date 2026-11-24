//! C ABI tests for the libmount cdylib. Mirrors the pattern used in
//! `compat/libuuid/tests/c_abi.rs`: each test compiles a small C
//! program against our `libmount.so`, runs it, and checks stdout.

use std::{path::PathBuf, process::Command, sync::OnceLock};

/// Build `linuxutils-libmount` once per test process. The package
/// is cdylib-only and the integration test doesn't reference its
/// Rust API, so cargo skips building it as a test prerequisite.
/// We have to ask for the artifact ourselves.
fn ensure_libmount_built() {
    static BUILT: OnceLock<()> = OnceLock::new();
    BUILT.get_or_init(|| {
        let mut cmd = Command::new(env!("CARGO"));
        cmd.args(["build", "-p", "linuxutils-libmount"]);
        if !cfg!(debug_assertions) {
            cmd.arg("--release");
        }
        let status = cmd.status().expect("failed to invoke cargo build");
        assert!(
            status.success(),
            "cargo build -p linuxutils-libmount failed"
        );
    });
}

/// Build a C test program against our libmount.so, run it, and
/// return its stdout.
fn run_c_test(c_source: &str) -> String {
    run_c_test_with_extra(c_source, &[])
}

/// Same as `run_c_test` but writes additional fixture files into
/// the test's tempdir first. Their absolute paths are substituted
/// into the C source via simple text replacement (`/*F0*/`,
/// `/*F1*/`, ...) so the C program can hardcode paths.
fn run_c_test_with_extra(c_source: &str, fixtures: &[(&str, &str)]) -> String {
    let tmp = tempfile::tempdir().unwrap();

    let mut source = c_source.to_string();
    for (i, (_name, contents)) in fixtures.iter().enumerate() {
        let path = tmp.path().join(format!("fixture_{i}"));
        std::fs::write(&path, contents).unwrap();
        let placeholder = format!("/*F{i}*/");
        source = source.replace(
            &placeholder,
            path.to_str().expect("fixture path must be UTF-8"),
        );
    }

    let c_path = tmp.path().join("test.c");
    std::fs::write(&c_path, source).unwrap();

    let lib_dir = find_lib_dir();
    let bin_path = tmp.path().join("test");
    let compile = Command::new("cc")
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            &format!("-L{}", lib_dir.display()),
            "-lmount",
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
    ensure_libmount_built();

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();

    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let dir = workspace_root.join("target").join(profile);
    if dir.join("libmount.so").exists() {
        return dir;
    }
    panic!(
        "libmount.so not found in {} after cargo build — \
         build of linuxutils-libmount did not produce the expected artifact",
        dir.display()
    );
}

fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("include")
}

#[test]
fn version_string() {
    let output = run_c_test(include_str!("c/version_string.c"));
    assert!(output.starts_with("version: "));
}

#[test]
fn empty_fs_lifecycle() {
    let output = run_c_test(include_str!("c/empty_fs_lifecycle.c"));
    assert_eq!(output.trim(), "OK");
}

#[test]
fn parse_fstab_and_iterate() {
    let output = run_c_test_with_extra(
        include_str!("c/parse_fstab_and_iterate.c"),
        &[(
            "fstab",
            "# comment\n\
             UUID=abcd /        ext4    defaults 0 1\n\
             /dev/sda1 /home    ext4    rw,relatime 0 2\n\
             tmpfs     /tmp     tmpfs   defaults 0 0\n",
        )],
    );
    assert!(output.contains("nents=3"));
    assert!(output.contains("entry: UUID=abcd / ext4"));
    assert!(output.contains("entry: /dev/sda1 /home ext4"));
    assert!(output.contains("entry: tmpfs /tmp tmpfs"));
}

#[test]
fn find_target_round_trip() {
    let output = run_c_test_with_extra(
        include_str!("c/find_target_round_trip.c"),
        &[(
            "fstab",
            "/dev/sda1 /home    ext4    rw,relatime 0 2\n\
             /dev/sda2 /        ext4    defaults    0 1\n",
        )],
    );
    assert!(output.contains("source: /dev/sda1"));
    assert!(output.contains("fstype: ext4"));
    assert!(output.contains("OK"));
}

#[test]
fn parse_mountinfo_vfs_options() {
    let output = run_c_test_with_extra(
        include_str!("c/parse_mountinfo_vfs_options.c"),
        &[(
            "mountinfo",
            "29 1 8:1 / / rw,relatime shared:1 - ext4 /dev/sda1 rw,errors=remount-ro\n",
        )],
    );
    assert!(output.contains("vfs: rw,relatime"));
    assert!(output.contains("fs: rw,errors=remount-ro"));
    assert!(output.contains("id: 29"));
    assert!(output.contains("parent: 1"));
}

#[test]
fn iter_direction_is_remembered() {
    let output = run_c_test_with_extra(
        include_str!("c/iter_direction_is_remembered.c"),
        &[(
            "fstab",
            "/dev/a /first  ext4 defaults 0 0\n\
             /dev/b /second ext4 defaults 0 0\n\
             /dev/c /third  ext4 defaults 0 0\n",
        )],
    );
    assert!(output.contains("first-fwd: /first"));
    assert!(output.contains("first-bwd: /third"));
}
