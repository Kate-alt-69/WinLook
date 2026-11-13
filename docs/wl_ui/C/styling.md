# wl_Lui C89 theme and styling API

`wl_Lui` exposes fixed-layout public structs and functions in
`C/library/wl_Lui/include/wl_Lui.h`. The API is C89/C90-compatible; it does
not require C++, C99 integer types, heap-owned CSS text or a web runtime.

## Initialize before editing

Always call the initializer before changing public fields:

```c
#include <wl_Lui.h>

int main(void) {
    wl_lui_window_desc description;
    wl_lui_window *window;
    wl_lui_theme theme;

    description.title = "Legacy settings";
    description.width = 900U;
    description.height = 640U;

    if (wl_lui_window_create(&description, &window) != WL_LUI_OK) {
        return 1;
    }

    if (wl_lui_theme_init(&theme, WL_LUI_THEME_DARK) != WL_LUI_OK) {
        wl_lui_window_destroy(window);
        return 2;
    }
    if (wl_lui_window_set_theme(window, &theme) != WL_LUI_OK) {
        wl_lui_window_destroy(window);
        return 3;
    }

    wl_lui_window_show(window);
    wl_lui_window_run(window);
    wl_lui_window_destroy(window);
    return 0;
}
```

## Theme structure

`wl_lui_theme` contains:

- `version` and `struct_size` for ABI validation.
- `scheme`: system, light, dark or high contrast.
- Three complete `wl_lui_palette` values.
- Font family (fixed 64-byte buffer), size and weight.
- Spacing unit, corner radius, control height and focus-ring width.
- Fast/normal/slow motion durations and reduced-motion preference.

The palette provides window, surface/alternate/elevated surface, primary,
secondary and disabled text, border, accent/hover/pressed, selection/focus,
success, warning, danger, info and shadow roles.

`wl_lui_theme_init(&theme, WL_LUI_THEME_SYSTEM)` initializes all presets.
For old/classic Win32 targets, `SYSTEM` currently resolves the default
palette to light rather than querying an OS theme setting. Choose light/dark/
high-contrast explicitly when deterministic colors matter.

## Style override structure

`wl_lui_style` contains a version/size pair, a bitmask, background/foreground/
border colors, border width, four padding values, font family/size/weight,
opacity and corner radius. Set only the properties named by `flags`. Values
for unselected fields are ignored; for example, a background-only override does
not require a valid font or opacity value.

```c
wl_lui_style style;

wl_lui_style_init(&style);
style.flags = WL_LUI_STYLE_BACKGROUND | WL_LUI_STYLE_FOREGROUND;
style.background.red = 25U;
style.background.green = 27U;
style.background.blue = 34U;
style.background.alpha = 255U;
style.foreground.red = 245U;
style.foreground.green = 246U;
style.foreground.blue = 248U;
style.foreground.alpha = 255U;

if (wl_lui_window_set_style(window, &style) == WL_LUI_UNSUPPORTED) {
    /* The selected platform does not implement one of the requested fields. */
}
```

## Capability flags

Call `wl_lui_style_capabilities()` for the built-in backend, or `wl_lui_window_style_capabilities(window)` for the selected window backend before choosing optional style fields.
The current native Win32 implementation supports:

- `WL_LUI_STYLE_BACKGROUND`
- `WL_LUI_STYLE_FOREGROUND`
- `WL_LUI_STYLE_FONT`

The current baseline returns `WL_LUI_UNSUPPORTED` for a style that requests
border, padding, opacity or corner-radius overrides. Those fields are
reserved in the public style structure but are not advertised as implemented
yet. The non-Windows contract stub reports zero native style capabilities.

This is intentional: C does not claim a border or blur was applied merely
because a field exists in the ABI. Future control backends can add their
capability flags without changing the meaning of the existing ones.

## ABI and lifetime rules

- Public structures are initialized by functions and copied by value during
  `wl_lui_window_set_theme` / `wl_lui_window_set_style`; the caller may release
  or reuse its local variable after the call returns.
- Strings must be null-terminated within `WL_LUI_FONT_FAMILY_CAPACITY`.
- Theme version and struct-size values must match the API contract.
- Invalid theme tokens and selected style-field sizes/weights/opacity are
  rejected before native resource creation. Unselected style fields are ignored.
- Window, theme and style calls are synchronous. Do not call them concurrently
  from arbitrary threads unless the host application serializes UI access.
- The current native backend uses GDI brushes/fonts and releases its owned
  resources when the window is destroyed.

## Custom host backends

A non-Windows host can supply a versioned `wl_lui_backend` table to
`wl_lui_window_create_with_backend`. The callbacks own native creation,
theme/style application, show/run and destruction. The table is copied, but
`user_data` and any resources it references must remain valid until
`wl_lui_window_destroy` returns. All callbacks are synchronous and the host
must serialize UI calls according to its own threading rules.

Each backend declares its supported style flags. Requests for unsupported
fields return `WL_LUI_UNSUPPORTED`; a custom backend does not inherit the
Win32 implementation or claim native rendering merely by accepting the
shared data structures. The default `wl_lui_window_create` still selects the
built-in Win32 backend where available.

## Platform limits

This API uses the classic Win32 desktop surface and C89 syntax. It does not
provide CSS, gradients, blur/acrylic, CSS grid/flex, HTML, WebView2 or modern
WinUI controls. Treat C89 compile tests, native Windows builds and real
legacy-OS runtime tests as separate verification gates.
