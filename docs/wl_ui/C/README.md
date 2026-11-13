# wl_Lui — legacy C UI foundation

`wl_Lui` is the C89 companion library for applications that need a classic
native Win32 window without a Rust runtime, C++ wrapper or embedded browser.
It lives at `C/library/wl_Lui`.

The public ABI is C89/C90-compatible. Initialize public structs through
`wl_lui_theme_init` and `wl_lui_style_init`; the structs carry version and
size fields so consumers and the library can validate the API contract.

## Capabilities

- Create, show, run and destroy a native window.
- Initialize light, dark and high-contrast semantic palettes.
- Apply a theme to a window.
- Apply supported window style overrides.
- Query supported native style flags before choosing a style.

`wl_Lui` currently provides a window lifecycle and base visual contract, not
a complete legacy control/layout toolkit. The modern Rust and C APIs share
the concepts and semantic theme roles, not identical implementation details.

See [styling.md](styling.md) for the full public theme/style structures, flags,
example and current backend support. The portable animation counterpart is
[`wl_Lanimate`](../../../C/library/wl_Lanimate/README.md).
