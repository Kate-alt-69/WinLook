# WinLook — Windows 11 application

This directory contains the Windows 11-specific application entry point. The current binary starts the shared Manager UI from rust/library/wl_ui.

Build from the repository root on Windows:

```sh
cargo build --release -p winlook-win11
```

The current binary is a UI prototype only. BGW startup, IPC, Windows 11 capability adapters and the full application runtime are future implementation stages.
