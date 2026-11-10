# wl_cpuv

Portable admission and accounting core for CPU work owned by WinLook.

- Protects a reserved critical-work allowance.
- Bounds in-flight admitted work.
- Tracks estimated versus measured abstract work units.
- Supports cancellation/refund and accounting-window reset.
- Reports pressure without creating threads or changing OS scheduling.

The host must define the cost unit, measure actual CPU work where supported, execute the admitted task and synchronize shared access. Wall-clock duration is not CPU time. This library cannot throttle unrelated processes.

## Test

```sh
cargo test --manifest-path rust/library/wl_cpuv/Cargo.toml
cargo clippy --manifest-path rust/library/wl_cpuv/Cargo.toml --all-targets -- -D warnings
```
