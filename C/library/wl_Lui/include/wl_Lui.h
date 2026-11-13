#ifndef WL_LUI_H
#define WL_LUI_H

/*
 * wl_Lui: C89-compatible native desktop window and styling API.
 *
 * Public structs are versioned and fixed-layout. Call the *_init functions
 * before changing individual fields. Native backends expose their supported
 * style subset through wl_lui_style_capabilities().
 */

#ifdef __cplusplus
extern "C" {
#endif

#define WL_LUI_VERSION_MAJOR 0
#define WL_LUI_VERSION_MINOR 2
#define WL_LUI_VERSION_PATCH 0
#define WL_LUI_THEME_VERSION 1U
#define WL_LUI_STYLE_VERSION 1U
#define WL_LUI_FONT_FAMILY_CAPACITY 64U

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

#define WL_LUI_BACKEND_VERSION 1U

typedef struct wl_lui_color {
    unsigned char red;
    unsigned char green;
    unsigned char blue;
    unsigned char alpha;
} wl_lui_color;

typedef enum wl_lui_theme_scheme {
    WL_LUI_THEME_SYSTEM = 0,
    WL_LUI_THEME_LIGHT = 1,
    WL_LUI_THEME_DARK = 2,
    WL_LUI_THEME_HIGH_CONTRAST = 3
} wl_lui_theme_scheme;

typedef struct wl_lui_palette {
    wl_lui_color window;
    wl_lui_color surface;
    wl_lui_color surface_alt;
    wl_lui_color surface_elevated;
    wl_lui_color text_primary;
    wl_lui_color text_secondary;
    wl_lui_color text_disabled;
    wl_lui_color border;
    wl_lui_color border_strong;
    wl_lui_color accent;
    wl_lui_color accent_hover;
    wl_lui_color accent_pressed;
    wl_lui_color accent_text;
    wl_lui_color selection;
    wl_lui_color selection_text;
    wl_lui_color focus;
    wl_lui_color success;
    wl_lui_color success_surface;
    wl_lui_color warning;
    wl_lui_color warning_surface;
    wl_lui_color danger;
    wl_lui_color danger_surface;
    wl_lui_color info;
    wl_lui_color info_surface;
    wl_lui_color shadow;
} wl_lui_palette;

typedef struct wl_lui_theme {
    unsigned int version;
    unsigned int struct_size;
    wl_lui_theme_scheme scheme;
    wl_lui_palette light;
    wl_lui_palette dark;
    wl_lui_palette high_contrast;
    char font_family[WL_LUI_FONT_FAMILY_CAPACITY];
    unsigned int font_size;
    unsigned int font_weight;
    unsigned int spacing_unit;
    unsigned int corner_radius;
    unsigned int control_height;
    unsigned int focus_ring_width;
    unsigned int motion_fast_ms;
    unsigned int motion_normal_ms;
    unsigned int motion_slow_ms;
    unsigned int reduced_motion;
} wl_lui_theme;

/* Fields selected by wl_lui_style.flags. */
#define WL_LUI_STYLE_BACKGROUND 0x00000001UL
#define WL_LUI_STYLE_FOREGROUND 0x00000002UL
#define WL_LUI_STYLE_BORDER 0x00000004UL
#define WL_LUI_STYLE_PADDING 0x00000008UL
#define WL_LUI_STYLE_FONT 0x00000010UL
#define WL_LUI_STYLE_OPACITY 0x00000020UL
#define WL_LUI_STYLE_CORNER_RADIUS 0x00000040UL

typedef struct wl_lui_style {
    unsigned int version;
    unsigned int struct_size;
    unsigned long flags;
    wl_lui_color background;
    wl_lui_color foreground;
    wl_lui_color border;
    unsigned int border_width;
    unsigned int padding_top;
    unsigned int padding_right;
    unsigned int padding_bottom;
    unsigned int padding_left;
    unsigned int font_size;
    unsigned int font_weight;
    char font_family[WL_LUI_FONT_FAMILY_CAPACITY];
    unsigned int opacity_milli;
    unsigned int corner_radius;
} wl_lui_style;

/*
 * Optional host-renderer interface. The library copies this versioned table,
 * but user_data and any resources it references remain owned by the caller
 * until wl_lui_window_destroy returns. All callbacks must be synchronous.
 */
typedef struct wl_lui_backend {
    unsigned int version;
    unsigned int struct_size;
    unsigned long style_capabilities;
    void *user_data;
    wl_lui_result (*create)(
        void *user_data,
        const wl_lui_window_desc *description,
        void **native_handle
    );
    wl_lui_result (*set_theme)(
        void *user_data,
        void *native_handle,
        const wl_lui_theme *theme
    );
    wl_lui_result (*set_style)(
        void *user_data,
        void *native_handle,
        const wl_lui_style *style
    );
    wl_lui_result (*show)(void *user_data, void *native_handle);
    wl_lui_result (*run)(void *user_data, void *native_handle);
    void (*destroy)(void *user_data, void *native_handle);
} wl_lui_backend;

/* Return bits describing style fields supported by the built-in backend. */
unsigned long wl_lui_style_capabilities(void);

/* Return style fields supported by the backend attached to this window. */
unsigned long wl_lui_window_style_capabilities(const wl_lui_window *window);

/* Creates a window using a caller-provided platform/host backend. */
wl_lui_result wl_lui_window_create_with_backend(
    const wl_lui_window_desc *description,
    const wl_lui_backend *backend,
    wl_lui_window **window_out
);

/* Initializes usable system/light/dark/high-contrast design tokens. */
wl_lui_result wl_lui_theme_init(
    wl_lui_theme *theme,
    wl_lui_theme_scheme scheme
);

/* Initializes an empty component style with no overrides. */
wl_lui_result wl_lui_style_init(wl_lui_style *style);

/* Creates a hidden native window. The caller owns the returned handle. */
wl_lui_result wl_lui_window_create(
    const wl_lui_window_desc *description,
    wl_lui_window **window_out
);

/* Applies a complete shared theme; the renderer maps supported native tokens. */
wl_lui_result wl_lui_window_set_theme(
    wl_lui_window *window,
    const wl_lui_theme *theme
);

/* Applies a component/window override or returns WL_LUI_UNSUPPORTED. */
wl_lui_result wl_lui_window_set_style(
    wl_lui_window *window,
    const wl_lui_style *style
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
