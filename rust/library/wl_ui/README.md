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

The shared application and styling types are imported from the crate root.
The portable `UiBackend` trait and `run_with_backend` entry point allow
application-owned renderers on platforms without a built-in adapter.
`wl_ui::animate` re-exports the portable `wl_animate` engine.
`UiApplication`, `UiPage`, `UiWindowConfig`, `UiError`, `UiTheme`,
`UiPalette`, `UiTypography`, `UiMetrics`, `UiEffects`, `UiColor`,
`UiStyle`, `UiSelector`, `UiStyleSheet` and typed layout/motion values.

The backend implementation crates remain private. Windows 11 deliberately
re-exports the native Windows Reactor component API from
`wl_ui::windows11` on Windows so application code can build native views
without depending on the implementation crate name. See
[`docs/wl_ui/`](../../../docs/wl_ui/README.md).

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

## Shared styling API

Every application model can carry a theme and ordered component rules:

    use wl_ui::{
        UiApplication, UiColor, UiComponent, UiEdges, UiPage, UiSelector,
        UiStyle, UiStyleSheet, UiTheme,
    };

    let styles = UiStyleSheet::new().rule(
        UiSelector::new(UiComponent::Button).class("primary"),
        UiStyle::new()
            .background(UiColor::rgb(0, 103, 192))
            .foreground(UiColor::rgb(255, 255, 255))
            .padding(UiEdges::xy(16.0, 10.0))
            .border_radius(8.0),
    );

    let app = UiApplication::new("Example")
        .page(UiPage::new("home", "Home", "Overview"))
        .theme(UiTheme::dark())
        .styles(styles);

The theme contains semantic light/dark/high-contrast palettes, typography,
spacing/radius/size tokens, shadows, focus and motion settings. The style model
provides validated component selectors, visual states, box/flex layout,
sizing, borders, gradients, text styles, transforms, transitions, bounded
keyframe animations and responsive media rules. Animation definitions are
typed and sanitized; Tauri uses CSS-native playback, while native hosts use
the shared animation engine and must advertise actual backend support.

Tauri 2 receives generated theme variables and selector CSS. The Windows 10
renderer maps solid semantic colors and typography to native Win32/GDI
resources. Windows 11 applies shared theme tokens to the WinUI shell and
re-exports Windows Reactor controls for app-authored native component trees.
A CSS property is not claimed to render on a native backend unless it is
actually mapped there.

Detailed API reference:
- [wl_ui design system and styling](../../../docs/wl_ui/styling.md)
- [renderer capability map](../../../docs/wl_ui/backends.md)
- [legacy C styling](../../../docs/wl_ui/C/styling.md)

## Internal crate layout

The public facade is the only top-level UI library directory:

- `core/` — platform-neutral application, navigation, theme and style contracts.
- `backends/win32/` — small baseline User32 renderer shared by classic Windows targets.
- `backends/windows7/`, `backends/windows10/`, `backends/windows11/` — opt-in platform implementations.

These remain separate Cargo packages so renderer feature boundaries and target
dependencies stay explicit, but their source and manifests live beneath
`rust/library/wl_ui/` instead of cluttering the shared-library root. Consumers
depend on `wl_ui`; the internal package names are not the primary public API.

The matching animation cores are separate portable libraries:
Rust `wl_animate` and C89 `wl_Lanimate`. Both are host-driven and intended
to share a single application/desktop-engine frame clock.

## Compatibility and toolchains

- **Windows 10:** standard Rust MSVC target + the Windows 10 Win32 backend. Build with the Windows 10 SDK / Visual Studio 2022 toolchain profile.
- **Windows 11:** WinUI 3 backend + the Windows 11 SDK / Visual Studio 2026 toolchain profile.
- **Windows 7/8/8.1:** experimental classic Win32 route. Use Rust 1.77's compatible standard library or build a custom Tier 3 `*-win7-windows-msvc` toolchain; the current standard `*-pc-windows-msvc` target still requires Windows 10 or newer.
- **Windows 95 through Vista:** use the separate C library `wl_Lui`, not the modern Rust standard-library target.
- **macOS/Linux/other hosts:** use Tauri 2 where its platform dependencies are available, or implement `UiBackend` and use the platform-neutral application model. The custom backend API is an extension contract, not a bundled GTK/Cocoa/SDL renderer.

A feature being available for compilation does not mean that every OS has passed a runtime compatibility test. See the compatibility matrix and per-version notes in `plan/compatibility/`.

## Dependency and size policy

The crate root contains only the platform-neutral model and feature declarations. Every backend is optional:
- the Win10 binary does not compile or link the WinUI 3 runtime;
- the Win11 binary does not compile the Tauri adapter;
- the Tauri adapter is not part of the root workspace and does not become a transitive dependency unless selected.

Use `default-features = false` in applications so future additions cannot silently change your runtime backend.

## Status

The facade, design-system and backend contracts remain alpha. The Win10 native renderer provides a reusable themed shell and page navigation baseline; it is not a full Win32 declarative controls/layout engine. Win11 exposes Windows Reactor for native custom controls, and Tauri hosts provide CSS-driven custom frontends. BGW IPC and actual system-setting actions remain separate work.
