# wl_ui

Shared native WinUI 3 shell and page composition for the WinLook Manager.

## Prototype scope

- Persistent left NavigationView with Home, Appearance, Audio & Media, File Explorer, Icons, Wallpaper, Window Manager, Displays, Notifications, Templates, Extensions, Compatibility, Diagnostics and Settings.
- Search filters navigation destinations.
- Back navigation, pane open/close and a live System/Light/Dark window-theme selector.
- Reusable page headers and information cards.
- Honest disconnected/preview states: no system settings are applied and BGW is not connected yet.

The modern UI prototype uses the pinned Windows Reactor preview 0.107.1 package (Rust 1.95+) backed by native WinUI 3. Its API may change; keep it isolated behind wl_ui and do not expose Reactor types to the application or BGW APIs. It uses the framework-dependent Windows App Runtime model; install/runtime packaging is a separate deployment task.

## Build and run

From the repository root on Windows 10 version 1809 or later / Windows 11 with the stable MSVC Rust toolchain. Install the matching **Windows App Runtime 2.5.1 (x64)** from the [official Windows App SDK downloads](https://learn.microsoft.com/en-us/windows/apps/windows-app-sdk/downloads) first; this is a framework-dependent prototype and does not bundle that runtime.

Run the Windows 10 or Windows 11 Manager target with:

```sh
cargo run -p winlook-win10
# or
cargo run -p winlook-win11
```

The root Cargo.lock pins the complete workspace dependency graph. To run the validation suite, use cargo fmt --all -- --check, cargo test -p wl_ui, cargo clippy -p wl_ui --all-targets -- -D warnings, and build each app with a separate target directory so both product outputs can be named winlook.exe. Do not copy only the EXE to a clean machine and assume it is self-contained.

## Not implemented yet

BGW IPC, persisted Manager settings, real engine controls, system capability probing, wallpaper previews, resource telemetry and the legacy C/Windows 95 UI are not part of this shell prototype. Pages are navigation/layout scaffolds until their backend capabilities exist.
