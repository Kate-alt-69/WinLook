# wl_Lui

`wl_Lui` is the C89 companion to the Rust `wl_ui` family. It provides a
versioned, fixed-layout theme/style API with a built-in Win32 window backend and a caller-provided portable backend interface.
The public header remains C89/C90-compatible and does not require C++ or C99.

## Quick start

    #include <wl_Lui.h>

    int main(void) {
        wl_lui_window_desc description;
        wl_lui_window *window;
        wl_lui_theme theme;

        description.title = "My native application";
        description.width = 900U;
        description.height = 640U;

        if (wl_lui_window_create(&description, &window) != WL_LUI_OK) {
            return 1;
        }

        wl_lui_theme_init(&theme, WL_LUI_THEME_DARK);
        if (wl_lui_window_set_theme(window, &theme) != WL_LUI_OK) {
            wl_lui_window_destroy(window);
            return 2;
        }

        wl_lui_window_show(window);
        wl_lui_window_run(window);
        wl_lui_window_destroy(window);
        return 0;
    }

## Styling contract

Call `wl_lui_theme_init` or `wl_lui_style_init` before changing fields. Theme
values include light, dark and high-contrast semantic palettes, typography,
spacing, control sizing, focus and motion preferences. A C theme can be shared
between windows and changed at runtime with `wl_lui_window_set_theme`.

`wl_lui_style` is an opt-in override with a bitmask. The active backend reports
supported fields through `wl_lui_style_capabilities()`; do not assume that a
field is rendered just because it exists in the ABI. The current Win32 baseline
supports window background, foreground and font overrides. Border, padding,
opacity and corner radius are reserved in the stable style contract but return
`WL_LUI_UNSUPPORTED` until the selected native renderer implements them.
The built-in non-Windows fallback reports no native style capabilities. Applications can provide a host backend with `wl_lui_window_create_with_backend` and query its capabilities per window using `wl_lui_window_style_capabilities`. The library copies the callback table; the caller must keep `user_data` and its resources alive until the window is destroyed.

## Runtime animation companion

`wl_Lui` provides the native window/theme/style contract; it does not create
per-control animation timers or automatically animate every style field.
Pair it with the separate C89 `wl_Lanimate` static library when an application
needs host-driven tweens/keyframe tracks:

```cmake
target_link_libraries(my_app PRIVATE WinLook::Lui WinLook::Lanimate)
```

Use one `wl_Lanimate_frame_clock` for the visible scene and pass the same
emitted `tick.delta_ms` to all active animation controllers. The host owns the
event loop and applies sampled values only to properties supported by its
selected backend. See [wl_Lanimate](../wl_Lanimate/README.md) and the
[shared animation runtime guide](../../../docs/wl_ui/animation.md).

## Portable animation integration

The companion C89 library is `C/library/wl_Lanimate`. It is intentionally a
separate dependency: `wl_Lui` supplies the window/theme/style contract, while
`wl_Lanimate` samples property values. A custom `wl_lui_backend` owns its host
event loop and can keep one `wl_Lanimate_frame_clock` for the whole UI scene,
advance all active animations from each emitted delta, then call
`wl_lui_window_set_style` only for changed values. Neither library creates a
timer or worker per widget.

The built-in Win32 backend still has a deliberately small capability set:
background, foreground and font. Border, padding, opacity and radius remain
unsupported there, so the animation library cannot make those properties
appear on that backend. A custom backend may implement additional fields and
advertise them through `style_capabilities`. The built-in blocking
`wl_lui_window_run` loop does not expose a generic frame callback; use a
custom backend/event loop when the application needs to combine message
dispatch with host-driven animation updates.

Link `wl_Lui` and `wl_Lanimate` as separate CMake targets (or static
libraries). The latter is portable C89 and can also be used without any UI
library.

## Compatibility and limits

- No WinUI, WinRT, Windows App SDK, WebView2, hidden worker thread, process hook,
  Shell replacement, or code injection.
- The built-in renderer uses the classic Win32 desktop surface; custom backends are supplied by the host through a versioned C89 callback table.
- The C backend applies native GDI colors and fonts; it does not emulate CSS
  gradients, blur, web layout engines, or unsupported modern visual effects.
- Build with CMake using `add_subdirectory(wl_Lui)` or link the `wl_Lui` static library.
- Portable contract tests run on non-Windows hosts. Real window creation and
  appearance must be tested on each claimed Windows release/VM.

See [the legacy wl_Lui styling guide](../../../docs/wl_ui/C/styling.md).
