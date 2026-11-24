//! Test harness for running linuxutils binaries against fake `/proc`,
//! `/sys`, or other system paths.
//!
//! Uses Linux user and mount namespaces (`unshare -rUm`) to bind-mount
//! one or more fixture directories over their target paths, then execs
//! the binary under test. No host root required. The namespace tears
//! itself down when the child exits.
//!
//! Tools whose CLI is featured-gated should access their bin via cargo's
//! `CARGO_BIN_EXE_<name>` env var (set automatically for tests in the
//! same crate as the bin), and point at fixture directories that mirror
//! the layout the binary would see at the corresponding system paths.
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! let out = testutil::Run::new(env!("CARGO_BIN_EXE_lslocks"))
//!     .mount("tests/fixtures/basic/proc", "/proc")
//!     .args(&["--noheadings"])
//!     .env("COLUMNS", "100")
//!     .output()
//!     .unwrap();
//! assert!(out.status.success());
//! ```

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// One bind-mount to set up before execing the target binary.
#[derive(Debug, Clone)]
pub struct Mount {
    /// Source directory on the host.
    pub source: PathBuf,
    /// Target absolute path the source will be visible as.
    pub target: &'static str,
}

/// Builder for spawning a binary inside an unshared user and mount
/// namespace with bind-mounted fixtures and an optional env overlay.
pub struct Run {
    binary: PathBuf,
    mounts: Vec<Mount>,
    args: Vec<String>,
    envs: Vec<(String, String)>,
}

impl Run {
    /// Start building a run of `binary`.
    pub fn new(binary: impl AsRef<Path>) -> Self {
        Self {
            binary: binary.as_ref().to_path_buf(),
            mounts: Vec::new(),
            args: Vec::new(),
            envs: Vec::new(),
        }
    }

    /// Bind-mount `source` over `target` inside the namespace.
    pub fn mount(
        mut self,
        source: impl AsRef<Path>,
        target: &'static str,
    ) -> Self {
        self.mounts.push(Mount {
            source: source.as_ref().to_path_buf(),
            target,
        });
        self
    }

    /// Append CLI arguments. Can be called repeatedly.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.args
            .extend(args.into_iter().map(|a| a.as_ref().to_owned()));
        self
    }

    /// Set an environment variable for the spawned binary.
    pub fn env(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    /// Spawn the binary and capture its `Output`.
    pub fn output(self) -> std::io::Result<Output> {
        let arg_refs: Vec<&str> =
            self.args.iter().map(String::as_str).collect();
        let env_refs: Vec<(&str, &str)> = self
            .envs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        run_with_envs(&self.binary, &self.mounts, &arg_refs, &env_refs)
    }

    /// Spawn, assert success, return stdout as a UTF-8 String.
    #[track_caller]
    pub fn run_ok(self) -> String {
        let out = self
            .output()
            .expect("failed to spawn unshare; is util-linux installed?");
        if !out.status.success() {
            panic!(
                "binary exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
                out.status,
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr),
            );
        }
        String::from_utf8(out.stdout).expect("stdout was not valid UTF-8")
    }
}

/// Spawn `binary args...` inside an unshared user and mount namespace,
/// with each entry of `mounts` bind-mounted into place. Returns the
/// captured `Output` on completion.
pub fn run(
    binary: impl AsRef<Path>,
    mounts: &[Mount],
    args: &[&str],
) -> std::io::Result<Output> {
    run_with_envs(binary.as_ref(), mounts, args, &[])
}

fn run_with_envs(
    binary: &Path,
    mounts: &[Mount],
    args: &[&str],
    envs: &[(&str, &str)],
) -> std::io::Result<Output> {
    // The script bind-mounts each (source, target) pair, then execs the
    // target binary. Args are passed positionally rather than
    // string-interpolated, so they're safe regardless of contents.
    //
    // Argv layout passed to sh: <num_mounts> <src1> <tgt1> <src2> <tgt2>
    // ... <binary> <arg1> <arg2> ...
    let script = r#"
n="$1"; shift
i=0
while [ "$i" -lt "$n" ]; do
    src="$1"; tgt="$2"; shift 2
    mount --bind "$src" "$tgt" \
        || { echo "mount --bind $src $tgt failed: $?" >&2; exit 100; }
    i=$((i + 1))
done
exec "$@"
"#;

    let mut cmd = Command::new("unshare");
    cmd.args([
        "--user",
        "--map-root-user",
        "--mount",
        "--",
        "sh",
        "-c",
        script,
        "--",
    ]);
    cmd.arg(mounts.len().to_string());
    for m in mounts {
        let src = m.source.canonicalize()?;
        cmd.arg(src);
        cmd.arg(m.target);
    }
    cmd.arg(binary);
    for a in args {
        cmd.arg(a);
    }
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output()
}

/// Convenience: a single `/proc` bind-mount.
pub fn proc_only(source: impl AsRef<Path>) -> Vec<Mount> {
    vec![Mount {
        source: source.as_ref().to_path_buf(),
        target: "/proc",
    }]
}

/// Convenience: a single `/sys` bind-mount.
pub fn sys_only(source: impl AsRef<Path>) -> Vec<Mount> {
    vec![Mount {
        source: source.as_ref().to_path_buf(),
        target: "/sys",
    }]
}

/// Run, assert success, and return stdout as a UTF-8 String. Panics
/// with a useful message on non-zero exit, including stderr.
#[track_caller]
pub fn run_ok(
    binary: impl AsRef<Path>,
    mounts: &[Mount],
    args: &[&str],
) -> String {
    let out = run(&binary, mounts, args)
        .expect("failed to spawn unshare; is util-linux installed?");
    if !out.status.success() {
        panic!(
            "binary exited with status {:?}\nstdout:\n{}\nstderr:\n{}",
            out.status,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
    }
    String::from_utf8(out.stdout).expect("stdout was not valid UTF-8")
}

/// Returns true if the harness can run here: the `unshare` program is
/// installed, and it can create an unprivileged user and mount namespace.
/// Tests should skip themselves when this returns false, as it does in
/// sandboxes such as Nix builds and some CI runners.
///
/// The check runs `unshare` once and the result is reused.
pub fn unshare_supported() -> bool {
    static SUPPORTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *SUPPORTED.get_or_init(|| {
        Command::new("unshare")
            .args(["--user", "--map-root-user", "--mount", "--", "true"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}
