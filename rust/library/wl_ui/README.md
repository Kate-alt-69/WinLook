# wl_ui

**wl_ui** is a public Rust UI/application-model crate with optional renderer modules. Consumers use one dependency and import the backend they want, like importing a package subpath in Node.js.

## Public module paths

| Import path | Cargo feature | Renderer |
|---|---|---|
| `wl_ui::windows10` | `windows10` | Lightweight native Win32 / User32 |
| `wl_ui::windows11` | `windows11` | Native WinUI 3 / Windows Reactor |
| `wl_ui::tauri2` | `tauri2` | Existing Tauri 2 application |
| `wl_ui::windows7` | `windows7` | Experimental classic Win32 / custom Rust target |
| `wl_ui::windows8` | `windows8` | Alias of the classic Win32 backend |
| `wl_ui::windows81` | `windows81` | Alias of the classic Win32 backend |

The shared types are always imported from the crate root:

- `wl_ui::UiApplication`
- `wl_ui::UiPage`
- `wl_ui::UiWindowConfig`
- `wl_ui::UiError`

The OS renderer crates are private implementation dependencies. Application code does not import their crate names or any Windows Reactor types.

## Windows 10 example

Cargo.toml:

    [dependencies]
    wl_ui = { version = "0.1.0-alpha.1", default-features = false, features = ["windows10"] }

src/main.rs:

    use wl_ui::windows10::{self, UiApplication, UiPage};

    fn main() -> Result<(), wl_ui::UiError> {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Application overview"))
            .page(UiPage::new("settings", "Settings", "Application preferences"))
            .initial_page("home");

        windows10::run_with(app)
    }

Or use `windows10::run()` for the standard WinLook Manager model.

This backend is intentionally small: classic native Win32 controls only. It has no dependency on Windows App SDK, WinUI 3, Windows Reactor, WebView2, or Tauri.

## Windows 11 example

Cargo.toml:

    [dependencies]
    wl_ui = { version = "0.1.0-alpha.1", default-features = false, features = ["windows11"] }

Then use the same application model:

    use wl_ui::windows11;

    fn main() -> Result<(), wl_ui::UiError> {
        windows11::run()
    }

The Windows 11 renderer uses the pinned Windows Reactor preview dependency and requires the compatible Windows App Runtime to be installed or packaged. It has a higher runtime and installer footprint than the Win32 renderer.

## Tauri 2 example

Cargo.toml:

    [dependencies]
    tauri = { version = "=2.12.5" }
    wl_ui = { version = "0.1.0-alpha.1", default-features = false, features = ["tauri2"] }

Then attach the shared model to the application's existing Tauri builder:

    use wl_ui::{UiApplication, UiPage};
    use wl_ui::tauri2;

    fn main() {
        let app = UiApplication::new("Example")
            .page(UiPage::new("home", "Home", "Application overview"))
            .initial_page("home");

        let builder = tauri2::attach(tauri::Builder::default(), app)
            .expect("valid UI model");

        builder
            .run(tauri::generate_context!())
            .expect("Tauri application failed");
    }

The application remains responsible for its Tauri frontend, `build.frontendDist`, plugins, commands, capabilities, and permissions. The adapter publishes the shared page/window model to `window.__WL_UI_BOOTSTRAP__` before the frontend scripts run. Windows uses the system WebView2 runtime, macOS uses WKWebView, and Linux uses WebKitGTK. Tauri is opt-in; native Win32 consumers do not compile or ship it.

## Selecting a renderer

Features are independent. A program may enable more than one module and choose the renderer in its own code. Enabling a feature does not silently select a different backend at runtime.

    wl_ui = {
        version = "0.1.0-alpha.1",
        default-features = false,
        features = ["windows10", "tauri2"]
    }

Then call either `wl_ui::windows10` or `wl_ui::tauri2`. Do not enable a renderer just because a library happens to be installed; select it deliberately in the consuming application's manifest.

## Compatibility and toolchains

- **Windows 10:** standard Rust MSVC target + the Windows 10 Win32 backend. Build with the Windows 10 SDK / Visual Studio 2022 toolchain profile.
- **Windows 11:** WinUI 3 backend + the Windows 11 SDK / Visual Studio 2026 toolchain profile.
- **Windows 7/8/8.1:** experimental classic Win32 route. Use Rust 1.77's compatible standard library or build a custom Tier 3 `*-win7-windows-msvc` toolchain; the current standard `*-pc-windows-msvc` target still requires Windows 10 or newer.
- **Windows 95 through Vista:** use the separate C library `wl_Lui`, not the modern Rust standard-library target.
- **macOS/Linux:** use the optional Tauri 2 module for a webview-based cross-platform application, or use the application model without a renderer.

A feature being available for compilation does not mean that every OS has passed a runtime compatibility test. See the compatibility matrix and per-version notes in `plan/compatibility/`.

## Dependency and size policy

The crate root contains only the platform-neutral model and feature declarations. Every backend is optional:
- the Win10 binary does not compile or link the WinUI 3 runtime;
- the Win11 binary does not compile the Tauri adapter;
- the Tauri adapter is not part of the root workspace and does not become a transitive dependency unless selected.

Use `default-features = false` in applications so future additions cannot silently change your runtime backend.

## Status

The facade and backend split are in alpha. The Win32 backend currently demonstrates the shell/window lifecycle and page navigation model; it is not yet a full production controls/layout toolkit. BGW IPC and actual system-setting actions remain separate work.
