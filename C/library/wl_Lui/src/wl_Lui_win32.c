#ifdef _WIN32

#include "wl_Lui_internal.h"

#include <windows.h>
#include <stdlib.h>
#include <string.h>

#ifndef OPAQUE
#define OPAQUE 2
#endif

#define WL_LUI_CLASS_NAME "WinLookLuiWindow"
#define WL_LUI_STATE_PROPERTY "WinLookLui.NativeState"

typedef struct wl_lui_native_window {
    HWND hwnd;
    HBRUSH background_brush;
    HBRUSH surface_brush;
    HFONT font;
    wl_lui_color background;
    wl_lui_color foreground;
} wl_lui_native_window;

static int wl_lui_class_registered = 0;

static COLORREF wl_lui_colorref(wl_lui_color color) {
    return RGB(color.red, color.green, color.blue);
}

static const wl_lui_palette *wl_lui_theme_palette(const wl_lui_theme *theme) {
    if (theme->scheme == WL_LUI_THEME_DARK) {
        return &theme->dark;
    }
    if (theme->scheme == WL_LUI_THEME_HIGH_CONTRAST) {
        return &theme->high_contrast;
    }
    return &theme->light;
}

static wl_lui_native_window *wl_lui_state(HWND hwnd) {
    return (wl_lui_native_window *)GetPropA(hwnd, WL_LUI_STATE_PROPERTY);
}

static LRESULT CALLBACK wl_lui_window_proc(
    HWND hwnd,
    UINT message,
    WPARAM wparam,
    LPARAM lparam
) {
    wl_lui_native_window *state;
    HDC dc;

    (void)lparam;
    state = wl_lui_state(hwnd);

    if (message == WM_ERASEBKGND && state != NULL && state->background_brush != NULL) {
        RECT rect;
        if (GetClientRect(hwnd, &rect)) {
            FillRect((HDC)wparam, &rect, state->background_brush);
        }
        return 1;
    }

    if ((message == WM_CTLCOLORSTATIC || message == WM_CTLCOLOREDIT
            || message == WM_CTLCOLORLISTBOX || message == WM_CTLCOLORBTN)
        && state != NULL) {
        dc = (HDC)wparam;
        SetTextColor(dc, wl_lui_colorref(state->foreground));
        if (state->surface_brush != NULL) {
            SetBkColor(dc, wl_lui_colorref(state->background));
            SetBkMode(dc, OPAQUE);
            return (LRESULT)state->surface_brush;
        }
    }

    if (message == WM_DESTROY) {
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcA(hwnd, message, wparam, lparam);
}

static wl_lui_result wl_lui_register_window_class(HINSTANCE instance) {
    WNDCLASSA window_class;
    ATOM registered;

    if (wl_lui_class_registered) {
        return WL_LUI_OK;
    }

    memset(&window_class, 0, sizeof(window_class));
    window_class.lpfnWndProc = wl_lui_window_proc;
    window_class.hInstance = instance;
    window_class.hCursor = LoadCursorA(NULL, IDC_ARROW);
    window_class.hbrBackground = NULL;
    window_class.lpszClassName = WL_LUI_CLASS_NAME;

    registered = RegisterClassA(&window_class);
    if (registered == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return WL_LUI_PLATFORM_ERROR;
    }

    wl_lui_class_registered = 1;
    return WL_LUI_OK;
}

static void wl_lui_release_gdi(wl_lui_native_window *native_window) {
    if (native_window->background_brush != NULL) {
        DeleteObject(native_window->background_brush);
        native_window->background_brush = NULL;
    }
    if (native_window->surface_brush != NULL) {
        DeleteObject(native_window->surface_brush);
        native_window->surface_brush = NULL;
    }
    if (native_window->font != NULL) {
        DeleteObject(native_window->font);
        native_window->font = NULL;
    }
}

static BOOL CALLBACK wl_lui_set_child_font(HWND child, LPARAM font) {
    SendMessageA(child, WM_SETFONT, (WPARAM)font, TRUE);
    return TRUE;
}

static wl_lui_result wl_lui_update_font(
    wl_lui_native_window *native_window,
    const char *family,
    unsigned int size,
    unsigned int weight
) {
    HFONT font;
    HWND hwnd;
    hwnd = native_window->hwnd;
    if (family == NULL || family[0] == '\0' || size == 0U || size > 256U
        || weight == 0U || weight > 1000U) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    font = CreateFontA(
        -(int)size, 0, 0, 0, (int)weight, FALSE, FALSE, FALSE,
        DEFAULT_CHARSET, OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY, DEFAULT_PITCH | FF_DONTCARE, family
    );
    if (font == NULL) {
        return WL_LUI_PLATFORM_ERROR;
    }
    if (native_window->font != NULL) {
        DeleteObject(native_window->font);
    }
    native_window->font = font;
    if (hwnd != NULL) {
        SendMessageA(hwnd, WM_SETFONT, (WPARAM)font, TRUE);
        EnumChildWindows(hwnd, wl_lui_set_child_font, (LPARAM)font);
    }
    return WL_LUI_OK;
}

unsigned long wl_lui_style_capabilities(void) {
    return WL_LUI_STYLE_BACKGROUND | WL_LUI_STYLE_FOREGROUND | WL_LUI_STYLE_FONT;
}

wl_lui_result wl_lui_platform_create(
    const wl_lui_window_desc *description,
    void **native_handle
) {
    HINSTANCE instance;
    HWND hwnd;
    wl_lui_native_window *native_window;
    wl_lui_result result;

    if (description == NULL || native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    *native_handle = NULL;

    instance = GetModuleHandleA(NULL);
    result = wl_lui_register_window_class(instance);
    if (result != WL_LUI_OK) {
        return result;
    }

    hwnd = CreateWindowExA(
        0, WL_LUI_CLASS_NAME, description->title, WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, (int)description->width, (int)description->height,
        NULL, NULL, instance, NULL
    );
    if (hwnd == NULL) {
        return WL_LUI_PLATFORM_ERROR;
    }

    native_window = (wl_lui_native_window *)calloc(1U, sizeof(wl_lui_native_window));
    if (native_window == NULL) {
        DestroyWindow(hwnd);
        return WL_LUI_OUT_OF_MEMORY;
    }
    native_window->hwnd = hwnd;
    native_window->background.red = 0U;
    native_window->background.green = 0U;
    native_window->background.blue = 0U;
    native_window->background.alpha = 255U;
    native_window->foreground.red = 255U;
    native_window->foreground.green = 255U;
    native_window->foreground.blue = 255U;
    native_window->foreground.alpha = 255U;
    if (!SetPropA(hwnd, WL_LUI_STATE_PROPERTY, (HANDLE)native_window)) {
        free(native_window);
        DestroyWindow(hwnd);
        return WL_LUI_PLATFORM_ERROR;
    }
    *native_handle = native_window;
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_set_theme(void *native_handle, const wl_lui_theme *theme) {
    wl_lui_native_window *native_window;
    const wl_lui_palette *palette;
    HBRUSH background;
    HBRUSH surface;
    wl_lui_result result;

    if (native_handle == NULL || theme == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    native_window = (wl_lui_native_window *)native_handle;
    palette = wl_lui_theme_palette(theme);
    background = CreateSolidBrush(wl_lui_colorref(palette->window));
    surface = CreateSolidBrush(wl_lui_colorref(palette->surface));
    if (background == NULL || surface == NULL) {
        if (background != NULL) DeleteObject(background);
        if (surface != NULL) DeleteObject(surface);
        return WL_LUI_PLATFORM_ERROR;
    }
    result = wl_lui_update_font(native_window, theme->font_family, theme->font_size, theme->font_weight);
    if (result != WL_LUI_OK) {
        DeleteObject(background);
        DeleteObject(surface);
        return result;
    }
    if (native_window->background_brush != NULL) DeleteObject(native_window->background_brush);
    if (native_window->surface_brush != NULL) DeleteObject(native_window->surface_brush);
    native_window->background_brush = background;
    native_window->surface_brush = surface;
    native_window->background = palette->surface;
    native_window->foreground = palette->text_primary;
    InvalidateRect(native_window->hwnd, NULL, TRUE);
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_set_style(void *native_handle, const wl_lui_style *style) {
    wl_lui_native_window *native_window;
    HBRUSH background;
    HBRUSH surface;
    wl_lui_color next_background;
    wl_lui_color next_foreground;

    if (native_handle == NULL || style == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & ~wl_lui_style_capabilities()) != 0UL) {
        return WL_LUI_UNSUPPORTED;
    }
    native_window = (wl_lui_native_window *)native_handle;
    if (style->flags == 0UL) {
        return WL_LUI_OK;
    }
    next_background = native_window->background;
    next_foreground = native_window->foreground;
    if ((style->flags & WL_LUI_STYLE_BACKGROUND) != 0UL) {
        next_background = style->background;
    }
    if ((style->flags & WL_LUI_STYLE_FOREGROUND) != 0UL) {
        next_foreground = style->foreground;
    }

    background = CreateSolidBrush(wl_lui_colorref(next_background));
    surface = CreateSolidBrush(wl_lui_colorref(next_background));
    if (background == NULL || surface == NULL) {
        if (background != NULL) DeleteObject(background);
        if (surface != NULL) DeleteObject(surface);
        return WL_LUI_PLATFORM_ERROR;
    }
    if ((style->flags & WL_LUI_STYLE_FONT) != 0UL) {
        if (wl_lui_update_font(native_window, style->font_family, style->font_size,
                style->font_weight) != WL_LUI_OK) {
            DeleteObject(background);
            DeleteObject(surface);
            return WL_LUI_PLATFORM_ERROR;
        }
    }

    if (native_window->background_brush != NULL) DeleteObject(native_window->background_brush);
    if (native_window->surface_brush != NULL) DeleteObject(native_window->surface_brush);
    native_window->background_brush = background;
    native_window->surface_brush = surface;
    native_window->background = next_background;
    native_window->foreground = next_foreground;
    InvalidateRect(native_window->hwnd, NULL, TRUE);
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_show(void *native_handle) {
    wl_lui_native_window *native_window;
    if (native_handle == NULL) return WL_LUI_INVALID_ARGUMENT;
    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd == NULL) return WL_LUI_PLATFORM_ERROR;
    ShowWindow(native_window->hwnd, SW_SHOW);
    UpdateWindow(native_window->hwnd);
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_run(void *native_handle) {
    wl_lui_native_window *native_window;
    MSG message;
    BOOL result;
    if (native_handle == NULL) return WL_LUI_INVALID_ARGUMENT;
    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd == NULL) return WL_LUI_PLATFORM_ERROR;
    while ((result = GetMessageA(&message, NULL, 0, 0)) > 0) {
        TranslateMessage(&message);
        DispatchMessageA(&message);
    }
    return result == -1 ? WL_LUI_PLATFORM_ERROR : WL_LUI_OK;
}

void wl_lui_platform_destroy(void *native_handle) {
    wl_lui_native_window *native_window;
    if (native_handle == NULL) return;
    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd != NULL) {
        RemovePropA(native_window->hwnd, WL_LUI_STATE_PROPERTY);
        if (IsWindow(native_window->hwnd)) DestroyWindow(native_window->hwnd);
        native_window->hwnd = NULL;
    }
    wl_lui_release_gdi(native_window);
    free(native_window);
}

#else

typedef int wl_lui_win32_backend_not_selected;

#endif
