# Vendored util-linux test suite

This directory is a copy of upstream util-linux's `tests/` tree,
imported as a behavioural oracle for our reimplementations. We run
upstream's `run.sh` against our `target/debug/` binaries. Tests
that pass give us regression coverage, and tests that fail form
the roadmap for a 1.0 release.

## Source

- Upstream: <https://github.com/util-linux/util-linux>
- Path: `tests/` at the upstream repo root
- Pinned commit: `2308d4c07f74d3149d9bb127afb85ce617ecad88` (2026-04-30)

## License

This directory is **not** under the workspace's MIT license. The
upstream test files are GPL-2.0+, with some pieces also LGPL. The
canonical GPL-2.0 text is in `LICENSE-GPL-2.0`. Each individual
test script also carries the upstream copyright header.

This directory is **not published** as part of any of our crates.
It lives outside the published Cargo manifest tree by design, so
we don't propagate the GPL boundary into anything downstream
consumers link against.

## Layout

```
tests/util-linux/
├── LICENSE-GPL-2.0          canonical license text
├── README.md                this file
├── run.sh                   upstream test orchestrator
├── functions.sh             `ts_*` helper library
├── commands.sh              `TS_CMD_*` defaults
├── ts/<tool>/<test>         individual test scripts
├── ts/<tool>/files/         test input fixtures
├── expected/ts/<tool>/      expected outputs to diff against
└── helpers/test_*.c         C helper programs (we don't build
                             them, so tests that need them skip)
```

## Running

From the workspace root:

```sh
just test-util-linux
```

This builds every workspace binary into `target/debug/` and
invokes `tests/util-linux-runner.sh`, which sets `top_builddir`
and the `TS_CMD_*` variables so the upstream `run.sh` resolves
binary names against our build.

Tests for tools we haven't implemented yet (most of them) will
auto-skip via `ts_check_test_command`. Tests requiring root,
specific block devices, or kernel features the runner can't find
also self-skip.

## Updating

1. `git -C /tmp clone --depth 1 https://github.com/util-linux/util-linux.git`
2. `rm -rf tests/util-linux/{run.sh,functions.sh,commands.sh,ts,helpers,expected,Makemodule.am}`
3. `cp -r /tmp/util-linux/tests/{run.sh,functions.sh,commands.sh,ts,helpers,expected,Makemodule.am} tests/util-linux/`
4. Update the pinned commit hash in this README.
5. Re-run `just test-util-linux` and check whether the pass list
   grew or shrank.
