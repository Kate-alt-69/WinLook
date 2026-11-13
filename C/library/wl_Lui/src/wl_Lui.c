#include "wl_Lui.h"
#include "wl_Lui_internal.h"

#include <stdlib.h>

struct wl_lui_window {
    void *native_handle;
};

wl_lui_result wl_lui_window_create(
    const wl_lui_window_desc *description,
    wl_lui_window **window_out
) {
    wl_lui_window *window;
    wl_lui_result result;

    if (window_out == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    *window_out = NULL;

    if (description == NULL || description->title == NULL
        || description->title[0] == '\0'
        || description->width < 320U || description->height < 240U
        || description->width > 16384U || description->height > 16384U) {
        return WL_LUI_INVALID_ARGUMENT;
    }

    window = (wl_lui_window *)malloc(sizeof(wl_lui_window));
    if (window == NULL) {
        return WL_LUI_OUT_OF_MEMORY;
    }

    window->native_handle = NULL;
    result = wl_lui_platform_create(description, &window->native_handle);
    if (result != WL_LUI_OK) {
        free(window);
        return result;
    }

    *window_out = window;
    return WL_LUI_OK;
}

wl_lui_result wl_lui_window_show(wl_lui_window *window) {
    if (window == NULL || window->native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    return wl_lui_platform_show(window->native_handle);
}

wl_lui_result wl_lui_window_run(wl_lui_window *window) {
    if (window == NULL || window->native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    return wl_lui_platform_run(window->native_handle);
}

void wl_lui_window_destroy(wl_lui_window *window) {
    if (window == NULL) {
        return;
    }
    if (window->native_handle != NULL) {
        wl_lui_platform_destroy(window->native_handle);
        window->native_handle = NULL;
    }
    free(window);
}

const char *wl_lui_result_string(wl_lui_result result) {
    switch (result) {
        case WL_LUI_OK:
            return "success";
        case WL_LUI_INVALID_ARGUMENT:
            return "invalid argument";
        case WL_LUI_OUT_OF_MEMORY:
            return "out of memory";
        case WL_LUI_UNSUPPORTED:
            return "native UI is not supported on this platform";
        case WL_LUI_PLATFORM_ERROR:
            return "native platform operation failed";
        default:
            return "unknown wl_Lui result";
    }
}

#ifndef _WIN32
wl_lui_result wl_lui_platform_create(
    const wl_lui_window_desc *description,
    void **native_handle
) {
    (void)description;
    (void)native_handle;
    return WL_LUI_UNSUPPORTED;
}

wl_lui_result wl_lui_platform_show(void *native_handle) {
    (void)native_handle;
    return WL_LUI_UNSUPPORTED;
}

wl_lui_result wl_lui_platform_run(void *native_handle) {
    (void)native_handle;
    return WL_LUI_UNSUPPORTED;
}

void wl_lui_platform_destroy(void *native_handle) {
    (void)native_handle;
}
#endif
