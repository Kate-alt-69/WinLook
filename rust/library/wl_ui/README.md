# wl_ui

Shared native WinUI 3 shell and page composition for the WinLook Manager.

## Prototype scope

- Persistent left NavigationView with Home, Appearance, Audio & Media, File Explorer, Icons, Wallpaper, Window Manager, Displays, Notifications, Templates, Extensions, Compatibility, Diagnostics and Settings.
- Search filters navigation destinations.
- Back navigation, pane open/close and a live System/Light/Dark window-theme selector.
- Reusable page headers and information cards.
- Honest disconnected/preview states: no system settings are applied and BGW is not connected yet.

The modern UI prototype uses Windows Reactor 0.100.0 (Rust 1.95+) backed by native WinUI 3. It uses the framework-dependent Windows App Runtime model; install/runtime packaging is a separate deployment task.

## Build

From the repository root on Windows with the stable MSVC Rust toolchain:

```sh
cargo fmt --all -- --check
cargo test -p wl_ui
cargo clippy -p wl_ui --all-targets -- -D warnings
cargo build --release -p winlook-win10 -p winlook-win11
```

The built executables require the matching Windows App Runtime to be installed. Do not copy only the EXE to a clean machine and assume it is self-contained.

## Not implemented yet

BGW IPC, persisted Manager settings, real engine controls, system capability probing, wallpaper previews, resource telemetry and the legacy C/Windows 95 UI are not part of this shell prototype. Pages are navigation/layout scaffolds until their backend capabilities exist.
