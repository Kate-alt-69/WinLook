# wl_Lui

wl_Lui is the C89 companion to the Rust wl_ui family. It provides a small native
Win32 window lifecycle API and deliberately starts with the oldest common Win32
surface instead of requiring modern controls or the Windows App SDK.

## Baseline

- C90/C89-compatible public header and implementation.
- The baseline uses ANSI User32 entry points such as RegisterClassA, CreateWindowExA,
  GetMessageA and DispatchMessageA.
- No WinUI, WinRT, Windows App SDK, WebView2, hidden worker thread, process hook,
  Shell replacement, or code injection.
- The native backend uses only the classic Win32 desktop window/control surface.
- Newer visual effects and controls must be optional capability-gated extensions.

## Minimal example

    #include <wl_Lui.h>

    int main(void) {
        wl_lui_window_desc description;
        wl_lui_window *window;

        description.title = "My Win32 application";
        description.width = 900U;
        description.height = 640U;

        if (wl_lui_window_create(&description, &window) != WL_LUI_OK) {
            return 1;
        }

        wl_lui_window_show(window);
        wl_lui_window_run(window);
        wl_lui_window_destroy(window);
        return 0;
    }

Build with CMake using add_subdirectory(wl_Lui) or link the wl_Lui static library.
Portable contract tests run on non-Windows hosts; real window creation must be tested
on Windows 95/98/Me, NT-family releases, and modern Windows before those OS versions
are declared supported.

The initial implementation is a window shell, not a full controls/layout toolkit.
The API is intentionally small so backend-specific control sets can grow without
breaking the stable C-facing contract.
