# wl_gpuv

Portable GPU resource/submission admission core for WinLook.

- Tracks WinLook-owned resource leases against a dynamic memory budget.
- Protects a configurable critical reserve.
- Bounds in-flight submissions and uses explicit completion tickets.
- Handles budget shrink without pretending existing allocations disappeared.

This crate does not create D3D/DXGI resources, query adapters, guarantee residency, submit commands or release driver allocations. The renderer must release its real resource before releasing its lease. Submission completion must be reported only after the relevant GPU fence/completion signal. A Windows adapter can feed OS-reported DXGI budgets in a later integration phase.

## Test

```sh
cargo test --manifest-path rust/library/wl_gpuv/Cargo.toml
cargo clippy --manifest-path rust/library/wl_gpuv/Cargo.toml --all-targets -- -D warnings
```
