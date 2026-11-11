# WinLook — Windows 10 application

This directory contains the Windows 10-specific application entry point. The current binary starts the shared Manager UI from rust/library/wl_ui.

Build from the repository root on Windows:

```sh
cargo build --release -p winlook-win10
```

The current binary is a UI prototype only. BGW startup, IPC, Windows 10 capability adapters and the full application runtime are future implementation stages.
