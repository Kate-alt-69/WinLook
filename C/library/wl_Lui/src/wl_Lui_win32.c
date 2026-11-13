#ifdef _WIN32

#include "wl_Lui_internal.h"

#include <windows.h>
#include <stdlib.h>
#include <string.h>

#define WL_LUI_CLASS_NAME "WinLookLuiWindow"

typedef struct wl_lui_native_window {
    HWND hwnd;
} wl_lui_native_window;

static int wl_lui_class_registered = 0;

static LRESULT CALLBACK wl_lui_window_proc(
    HWND hwnd,
    UINT message,
    WPARAM wparam,
    LPARAM lparam
) {
    (void)wparam;
    (void)lparam;

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
    window_class.hbrBackground = (HBRUSH)(COLOR_BTNFACE + 1);
    window_class.lpszClassName = WL_LUI_CLASS_NAME;

    registered = RegisterClassA(&window_class);
    if (registered == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
        return WL_LUI_PLATFORM_ERROR;
    }

    wl_lui_class_registered = 1;
    return WL_LUI_OK;
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
        0,
        WL_LUI_CLASS_NAME,
        description->title,
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        (int)description->width,
        (int)description->height,
        NULL,
        NULL,
        instance,
        NULL
    );
    if (hwnd == NULL) {
        return WL_LUI_PLATFORM_ERROR;
    }

    native_window = (wl_lui_native_window *)malloc(sizeof(wl_lui_native_window));
    if (native_window == NULL) {
        DestroyWindow(hwnd);
        return WL_LUI_OUT_OF_MEMORY;
    }

    native_window->hwnd = hwnd;
    *native_handle = native_window;
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_show(void *native_handle) {
    wl_lui_native_window *native_window;

    if (native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }

    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd == NULL) {
        return WL_LUI_PLATFORM_ERROR;
    }

    ShowWindow(native_window->hwnd, SW_SHOW);
    UpdateWindow(native_window->hwnd);
    return WL_LUI_OK;
}

wl_lui_result wl_lui_platform_run(void *native_handle) {
    wl_lui_native_window *native_window;
    MSG message;
    BOOL result;

    if (native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }

    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd == NULL) {
        return WL_LUI_PLATFORM_ERROR;
    }

    while ((result = GetMessageA(&message, NULL, 0, 0)) > 0) {
        TranslateMessage(&message);
        DispatchMessageA(&message);
    }

    return result == -1 ? WL_LUI_PLATFORM_ERROR : WL_LUI_OK;
}

void wl_lui_platform_destroy(void *native_handle) {
    wl_lui_native_window *native_window;

    if (native_handle == NULL) {
        return;
    }

    native_window = (wl_lui_native_window *)native_handle;
    if (native_window->hwnd != NULL) {
        if (IsWindow(native_window->hwnd)) {
            DestroyWindow(native_window->hwnd);
        }
        native_window->hwnd = NULL;
    }
    free(native_window);
}

#else

/* Keep this translation unit valid under strict C89 builds on non-Windows hosts. */
typedef int wl_lui_win32_backend_not_selected;

#endif
