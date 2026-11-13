# wl_ui_tauri2

Optional Tauri 2 host adapter for the platform-neutral wl_ui_core application model.

This crate does not replace or reconfigure an application's Tauri runtime. It attaches a
wl-ui webview window to an existing tauri::Builder, preserving the host's plugins,
commands, permissions, capabilities, and frontend build.

## Use

Add wl_ui and the Tauri 2 runtime to the consuming application's Cargo manifest. Then configure build.frontendDist to a directory containing index.html.

    use wl_ui::{UiApplication, UiPage};

    let ui = UiApplication::new("Example")
        .page(UiPage::new("home", "Home", "Application overview"))
        .page(UiPage::new("settings", "Settings", "Application preferences"))
        .initial_page("home");

    let builder = wl_ui::tauri2::attach(tauri::Builder::default(), ui)
        .expect("valid UI model");

    builder
        .run(tauri::generate_context!())
        .expect("Tauri application failed");

The frontend receives the page metadata as window.__WL_UI_BOOTSTRAP__ before
its application scripts execute. The host remains responsible for rendering the
frontend and for configuring Tauri permissions and capabilities.

## Platform notes

- Windows uses the system WebView2 runtime; this adapter does not bundle Chromium.
- macOS uses the system WKWebView.
- Linux uses the platform WebKitGTK webview and requires its system packages.
- Tauri 2.12 no longer officially supports Windows 7. Use the native Win32 backend
  or wl_Lui for older Windows systems.

This package is intentionally excluded from the root Cargo workspace so the native
Win10/Win11 builds do not resolve Tauri or WebView dependencies.
