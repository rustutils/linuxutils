//! C ABI tests for the libblkid cdylib. Each test compiles a small
//! C program against `libblkid.so`, runs it (with
//! `LIBBLKID_ROOT_DIR` pointed at a fixture tree where appropriate),
//! and checks stdout.

use std::{path::PathBuf, process::Command, sync::OnceLock};

fn ensure_libblkid_built() {
    static BUILT: OnceLock<()> = OnceLock::new();
    BUILT.get_or_init(|| {
        let mut cmd = Command::new(env!("CARGO"));
        cmd.args(["build", "-p", "linuxutils-libblkid"]);
        if !cfg!(debug_assertions) {
            cmd.arg("--release");
        }
        let status = cmd.status().expect("failed to invoke cargo build");
        assert!(
            status.success(),
            "cargo build -p linuxutils-libblkid failed"
        );
    });
}

fn find_lib_dir() -> PathBuf {
    ensure_libblkid_built();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest.parent().unwrap().parent().unwrap();
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let dir = workspace_root.join("target").join(profile);
    if dir.join("libblkid.so").exists() {
        return dir;
    }
    panic!(
        "libblkid.so not found in {} after cargo build — \
         build of linuxutils-libblkid did not produce the expected artifact",
        dir.display()
    );
}

fn include_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("include")
}

/// Build, link, run a C program. `root_dir` is set as
/// `LIBBLKID_ROOT_DIR` so the cdylib resolves against that fixture
/// tree instead of the host's `/dev/disk/`.
fn run_c_test(c_source: &str, root_dir: Option<&std::path::Path>) -> String {
    let tmp = tempfile::tempdir().unwrap();
    let c_path = tmp.path().join("test.c");
    std::fs::write(&c_path, c_source).unwrap();

    let lib_dir = find_lib_dir();
    let bin_path = tmp.path().join("test");
    let compile = Command::new("cc")
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            &format!("-L{}", lib_dir.display()),
            "-lblkid",
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

    let mut runner = Command::new(&bin_path);
    runner.env("LD_LIBRARY_PATH", &lib_dir);
    if let Some(root) = root_dir {
        runner.env("LIBBLKID_ROOT_DIR", root);
    }
    let run = runner.output().expect("failed to run test binary");

    assert!(
        run.status.success(),
        "Test binary failed (exit {}):\nstdout: {}\nstderr: {}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr),
    );
    String::from_utf8(run.stdout).unwrap()
}

/// Build a fake `/dev/disk`-shaped tree under `root` with `by-label`
/// and `by-uuid` symlinks pointing at one device. Returns the
/// canonical device path the test C program should expect.
fn build_fixture(root: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::symlink;
    std::fs::create_dir_all(root.join("by-label")).unwrap();
    std::fs::create_dir_all(root.join("by-uuid")).unwrap();
    let dev_path = root.join("devices").join("sda1");
    std::fs::create_dir_all(dev_path.parent().unwrap()).unwrap();
    std::fs::write(&dev_path, "").unwrap();
    symlink(&dev_path, root.join("by-label").join("test-root")).unwrap();
    symlink(&dev_path, root.join("by-uuid").join("abcd-1234")).unwrap();
    std::fs::canonicalize(&dev_path).unwrap()
}

#[test]
fn version_string() {
    let output = run_c_test(include_str!("c/version_string.c"), None);
    assert!(output.contains("version: "));
    assert!(output.contains("date: "));
}

#[test]
fn parse_tag_string() {
    let output = run_c_test(include_str!("c/parse_tag_string.c"), None);
    assert_eq!(output.trim(), "OK");
}

#[test]
fn cache_lifecycle() {
    let output = run_c_test(include_str!("c/cache_lifecycle.c"), None);
    assert_eq!(output.trim(), "OK");
}

#[test]
fn evaluate_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let canonical = build_fixture(tmp.path());
    let output = run_c_test(include_str!("c/evaluate_tag.c"), Some(tmp.path()));
    let expected = canonical.to_string_lossy();
    assert!(
        output.contains(&format!("resolved: {expected}")),
        "missing resolved line — got:\n{output}"
    );
    assert!(output.contains(&format!("spec-resolved: {expected}")));
    assert!(output.contains("OK"));
}

#[test]
fn iterate_devices_and_tags() {
    let tmp = tempfile::tempdir().unwrap();
    let canonical = build_fixture(tmp.path());
    let output = run_c_test(
        include_str!("c/iterate_devices_and_tags.c"),
        Some(tmp.path()),
    );
    assert!(output.contains(&format!("dev: {}", canonical.display())));
    assert!(output.contains("tag: LABEL=test-root"));
    assert!(output.contains("tag: UUID=abcd-1234"));
    assert!(output.contains("count: 1"));
}

#[test]
fn get_dev_and_has_tag() {
    let tmp = tempfile::tempdir().unwrap();
    let canonical = build_fixture(tmp.path());
    let output =
        run_c_test(include_str!("c/get_dev_and_has_tag.c"), Some(tmp.path()));
    assert!(output.contains(&format!("path: {}", canonical.display())));
    assert!(output.contains("has-label-eq: 1"));
    assert!(output.contains("has-uuid-any: 1"));
    assert!(output.contains("has-other: 0"));
    assert!(output.contains("uuid: abcd-1234"));
    assert!(output.contains("OK"));
}
