# WinLook — Windows 11 application

This directory contains the Windows 11-specific application entry point. The current binary starts the shared Manager UI from rust/library/wl_ui.

Build from the repository root on Windows:

```sh
cargo build --release -p winlook-win11
```

The `winlook.exe` binary starts the WinUI 3 Manager. BGW is a separate,
UI-independent executable shared by Windows 10 and Windows 11:

```sh
python build.py build bgw
```

This builds `winlook.bg.exe` under `build/rust/bgw/winlook-bgw/release/`.
The initial BGW slice monitors the Explorer desktop-host process only.
Windows startup registration, Manager IPC, persistent configuration, the
in-process Shell bridge and actual desktop presentation remain future stages.
