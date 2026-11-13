# wl_ui — application UI foundation

`wl_ui` is WinLook's reusable UI foundation for applications that want a
shared application model, design system and renderer-facing styling API. It is
designed for explicit backend selection: Windows applications use native
Windows renderers; Linux can opt into the Tauri 2/WebKitGTK bridge without
pulling a webview into a Windows 10 native build.

## Install

Use one dependency and select only the backend your application intends to use:

```toml
[dependencies]
wl_ui = {
    path = "../../rust/library/wl_ui",
    default-features = false,
    features = ["windows10"]
}
```

Switch `windows10` to `windows11` or `tauri2` for those renderer choices.
Use `default-features = false` so adding a backend to the crate never changes
your application's runtime by accident.

## Public module paths

| Public import | Feature | Intended renderer |
|---|---|---|
| `wl_ui::windows10` | `windows10` | Native Win32/User32 |
| `wl_ui::windows11` | `windows11` | Native WinUI 3 / Windows Reactor |
| `wl_ui::tauri2` | `tauri2` | Existing Tauri 2 host; WebKitGTK on Linux |
| `wl_ui::windows7` | `windows7` | Experimental classic Win32 |
| `wl_ui::windows8` | `windows8` | Classic Win32 compatibility alias |
| `wl_ui::windows81` | `windows81` | Classic Win32 compatibility alias |

The application model and style vocabulary are exported from the crate root:
`UiApplication`, `UiPage`, `UiWindowConfig`, `UiTheme`, `UiPalette`,
`UiTypography`, `UiMetrics`, `UiEffects`, `UiStyle`, `UiSelector`,
`UiStyleSheet`, and related typed values.

## Example

```rust
use wl_ui::{
    UiApplication, UiColor, UiComponent, UiMediaCondition, UiPage,
    UiSelector, UiStyle, UiStyleSheet, UiTheme, UiEdges,
};
use wl_ui::windows10;

fn main() -> Result<(), wl_ui::UiError> {
    let styles = UiStyleSheet::new()
        .rule(
            UiSelector::new(UiComponent::Card).class("settings-card"),
            UiStyle::new()
                .background(UiColor::rgb(255, 255, 255))
                .foreground(UiColor::rgb(30, 35, 44))
                .padding(UiEdges::all(20.0))
                .border_radius(12.0),
        )
        .media_rule(
            UiSelector::new(UiComponent::Card).class("settings-card"),
            UiStyle::new().padding(UiEdges::all(28.0)),
            UiMediaCondition::MinWidth(900.0),
        );

    let app = UiApplication::new("Example")
        .page(UiPage::new("home", "Home", "Application overview"))
        .initial_page("home")
        .theme(UiTheme::dark())
        .styles(styles);

    windows10::run_with(app)
}
```

## Documentation

- [Styling and design-system API](styling.md)
- [Renderer and compatibility map](backends.md)
- [Legacy C API overview](C/README.md)
- [Legacy C theme and style API](C/styling.md)

The styling API is deliberately typed and validates values before a renderer
uses them. Tauri/WebKit receives CSS variables and generated selector rules.
Native renderers translate supported semantic values to native resources; CSS
properties that a native renderer cannot express are not silently emulated.

## Product boundary

This is the reusable shell, design-token and styling foundation. It does not
claim that every backend has identical controls or can reproduce every CSS
effect. Windows 10 currently styles the shell/window, navigation surface,
content surface, text and font with native GDI/Win32 resources. Windows 11
exposes Windows Reactor controls through `wl_ui::windows11` on Windows.
Tauri is the CSS-capable route for Linux and other supported Tauri targets.

Backend-specific controls, accessibility and compatibility are documented in
[backends.md](backends.md); test actual target OS versions before claiming
runtime support.
