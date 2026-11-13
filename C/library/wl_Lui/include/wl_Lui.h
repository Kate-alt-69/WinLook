#ifndef WL_LUI_H
#define WL_LUI_H

/*
 * wl_Lui: small C89-compatible native Win32 window-shell API.
 *
 * The baseline uses ANSI User32 entry points so applications can be compiled
 * for older Win32 target environments. Newer controls and visual effects belong
 * in separate capability-checked extensions.
 */

#ifdef __cplusplus
extern "C" {
#endif

#define WL_LUI_VERSION_MAJOR 0
#define WL_LUI_VERSION_MINOR 1
#define WL_LUI_VERSION_PATCH 0

typedef enum wl_lui_result {
    WL_LUI_OK = 0,
    WL_LUI_INVALID_ARGUMENT = 1,
    WL_LUI_OUT_OF_MEMORY = 2,
    WL_LUI_UNSUPPORTED = 3,
    WL_LUI_PLATFORM_ERROR = 4
} wl_lui_result;

typedef struct wl_lui_window_desc {
    const char *title;
    unsigned int width;
    unsigned int height;
} wl_lui_window_desc;

typedef struct wl_lui_window wl_lui_window;

/* Creates a hidden native window. The caller owns the returned handle. */
wl_lui_result wl_lui_window_create(
    const wl_lui_window_desc *description,
    wl_lui_window **window_out
);

/* Shows and updates a previously created window. */
wl_lui_result wl_lui_window_show(wl_lui_window *window);

/* Runs the Win32 message loop until the window is closed. */
wl_lui_result wl_lui_window_run(wl_lui_window *window);

/* Destroys the native window and releases the library-owned handle. */
void wl_lui_window_destroy(wl_lui_window *window);

/* Returns a stable, human-readable description for a result code. */
const char *wl_lui_result_string(wl_lui_result result);

#ifdef __cplusplus
}
#endif

#endif
