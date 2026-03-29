# prlimit

Get and set process resource limits.

Reads limits using `getrlimit(2)` for the current process, or
`/proc/<pid>/limits` for other processes. Sets limits using `setrlimit(2)`
for self or `prlimit(2)` for other PIDs (requires `CAP_SYS_RESOURCE`).

Resource flags (`--core`, `--nofile`, etc.) accept an optional `=limit` value
in `soft:hard`, `soft:`, `:hard`, or plain `value` format (use `unlimited` for
no limit). Without `=limit`, the flag selects that resource for display only.
