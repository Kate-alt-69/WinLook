# wl_ui renderer and compatibility map

Renderer selection is an explicit Cargo feature and module import. Enabling
more than one feature permits an application to make a deliberate choice; it
does not auto-detect an OS or silently fall back.

## Native Windows 10

`wl_ui::windows10` uses classic Win32/User32. It does not include Windows App
SDK, WinUI 3, WebView2, Windows Reactor or Tauri in the Win10 dependency graph.
The renderer currently provides the application window, page navigation and
page content surface. The shared theme maps to GDI brushes/font and
`WM_CTLCOLOR*` handling.

This backend is intentionally the low-dependency option. It does not interpret
the complete CSS style-sheet model as if Win32 were a browser.

## Native Windows 11

`wl_ui::windows11` is the native WinUI 3 / Windows Reactor path and requires
the pinned Windows App SDK/Windows App Runtime profile already documented in
the repository's build plan. On Windows, the public module re-exports the
Windows Reactor component API so applications can create their own
`Component`, `View`, controls and layouts while sharing `UiTheme` and
design tokens from `wl_ui`.

The shared manager shell uses theme colors, typography, content padding,
surface borders and rounded corners. App-authored WinUI components should use
the exported native control methods for detailed native states and behavior.

## Tauri 2 (Linux and optional webview apps)

`wl_ui::tauri2` attaches to an existing Tauri 2 `Builder`. The host owns its
frontend assets, plugins, commands, permissions, capabilities and build
configuration. The adapter publishes the app/page model and generated theme
CSS before the frontend runs.

On Linux, Tauri uses WebKitGTK. This is optional and is not pulled into the
native Win32 or WinUI dependency trees. It is a small cross-platform bridge,
not a promise that every machine already has WebKitGTK development libraries
or the runtime dependencies installed.

## Legacy Win7 / Win8 / Win8.1 Rust modules

These modules remain experimental classic Win32 aliases and are constrained
by the selected Rust standard library/toolchain baseline. A feature compile is
not runtime compatibility evidence. See `plan/compatibility/`.

## Legacy C: wl_Lui

`C/library/wl_Lui` is a C89-compatible Win32 window/theme baseline. It can be
used by old toolchains and is kept separate from modern Rust dependencies.
The current C style capability query reports only supported fields; see
[the legacy guide](C/styling.md).

## Runtime verification

Feature availability and compiler success do not prove a Windows release's
runtime behavior. Actual OS/VM testing is required before adding a version to
the supported runtime matrix.
