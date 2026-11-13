#include "wl_Lui.h"
#include "wl_Lui_internal.h"

#include <stdlib.h>
#include <string.h>

struct wl_lui_window {
    void *native_handle;
    wl_lui_theme theme;
    wl_lui_style style;
    wl_lui_backend backend;
};

static wl_lui_result wl_lui_default_create(
    void *user_data, const wl_lui_window_desc *description, void **native_handle
) {
    (void)user_data;
    return wl_lui_platform_create(description, native_handle);
}

static wl_lui_result wl_lui_default_set_theme(
    void *user_data, void *native_handle, const wl_lui_theme *theme
) {
    (void)user_data;
    return wl_lui_platform_set_theme(native_handle, theme);
}

static wl_lui_result wl_lui_default_set_style(
    void *user_data, void *native_handle, const wl_lui_style *style
) {
    (void)user_data;
    return wl_lui_platform_set_style(native_handle, style);
}

static wl_lui_result wl_lui_default_show(void *user_data, void *native_handle) {
    (void)user_data;
    return wl_lui_platform_show(native_handle);
}

static wl_lui_result wl_lui_default_run(void *user_data, void *native_handle) {
    (void)user_data;
    return wl_lui_platform_run(native_handle);
}

static void wl_lui_default_destroy(void *user_data, void *native_handle) {
    (void)user_data;
    wl_lui_platform_destroy(native_handle);
}

static const wl_lui_backend wl_lui_default_backend = {
    WL_LUI_BACKEND_VERSION,
    (unsigned int)sizeof(wl_lui_backend),
    0UL,
    NULL,
    wl_lui_default_create,
    wl_lui_default_set_theme,
    wl_lui_default_set_style,
    wl_lui_default_show,
    wl_lui_default_run,
    wl_lui_default_destroy
};

static wl_lui_color wl_lui_rgb(unsigned char red, unsigned char green, unsigned char blue) {
    wl_lui_color color;
    color.red = red;
    color.green = green;
    color.blue = blue;
    color.alpha = 255U;
    return color;
}

static wl_lui_color wl_lui_rgba(
    unsigned char red,
    unsigned char green,
    unsigned char blue,
    unsigned char alpha
) {
    wl_lui_color color;
    color.red = red;
    color.green = green;
    color.blue = blue;
    color.alpha = alpha;
    return color;
}

static void wl_lui_palette_light(wl_lui_palette *palette) {
    palette->window = wl_lui_rgb(248U, 249U, 251U);
    palette->surface = wl_lui_rgb(255U, 255U, 255U);
    palette->surface_alt = wl_lui_rgb(242U, 244U, 247U);
    palette->surface_elevated = wl_lui_rgb(255U, 255U, 255U);
    palette->text_primary = wl_lui_rgb(27U, 34U, 46U);
    palette->text_secondary = wl_lui_rgb(87U, 96U, 109U);
    palette->text_disabled = wl_lui_rgb(142U, 148U, 157U);
    palette->border = wl_lui_rgb(222U, 226U, 232U);
    palette->border_strong = wl_lui_rgb(190U, 197U, 207U);
    palette->accent = wl_lui_rgb(0U, 103U, 192U);
    palette->accent_hover = wl_lui_rgb(0U, 88U, 168U);
    palette->accent_pressed = wl_lui_rgb(0U, 72U, 140U);
    palette->accent_text = wl_lui_rgb(255U, 255U, 255U);
    palette->selection = wl_lui_rgb(218U, 238U, 255U);
    palette->selection_text = wl_lui_rgb(12U, 49U, 82U);
    palette->focus = wl_lui_rgb(0U, 103U, 192U);
    palette->success = wl_lui_rgb(16U, 124U, 16U);
    palette->success_surface = wl_lui_rgb(225U, 245U, 222U);
    palette->warning = wl_lui_rgb(157U, 93U, 0U);
    palette->warning_surface = wl_lui_rgb(255U, 244U, 206U);
    palette->danger = wl_lui_rgb(196U, 43U, 28U);
    palette->danger_surface = wl_lui_rgb(253U, 231U, 230U);
    palette->info = wl_lui_rgb(0U, 103U, 192U);
    palette->info_surface = wl_lui_rgb(226U, 241U, 255U);
    palette->shadow = wl_lui_rgba(0U, 0U, 0U, 48U);
}

static void wl_lui_palette_dark(wl_lui_palette *palette) {
    palette->window = wl_lui_rgb(24U, 24U, 28U);
    palette->surface = wl_lui_rgb(32U, 32U, 36U);
    palette->surface_alt = wl_lui_rgb(41U, 41U, 46U);
    palette->surface_elevated = wl_lui_rgb(49U, 49U, 55U);
    palette->text_primary = wl_lui_rgb(244U, 244U, 247U);
    palette->text_secondary = wl_lui_rgb(190U, 192U, 198U);
    palette->text_disabled = wl_lui_rgb(133U, 136U, 143U);
    palette->border = wl_lui_rgb(69U, 70U, 77U);
    palette->border_strong = wl_lui_rgb(96U, 98U, 107U);
    palette->accent = wl_lui_rgb(96U, 157U, 234U);
    palette->accent_hover = wl_lui_rgb(121U, 176U, 244U);
    palette->accent_pressed = wl_lui_rgb(65U, 130U, 215U);
    palette->accent_text = wl_lui_rgb(12U, 22U, 36U);
    palette->selection = wl_lui_rgb(45U, 80U, 115U);
    palette->selection_text = wl_lui_rgb(246U, 250U, 255U);
    palette->focus = wl_lui_rgb(124U, 181U, 246U);
    palette->success = wl_lui_rgb(108U, 200U, 120U);
    palette->success_surface = wl_lui_rgb(32U, 63U, 39U);
    palette->warning = wl_lui_rgb(243U, 190U, 90U);
    palette->warning_surface = wl_lui_rgb(77U, 55U, 20U);
    palette->danger = wl_lui_rgb(255U, 133U, 128U);
    palette->danger_surface = wl_lui_rgb(78U, 34U, 32U);
    palette->info = wl_lui_rgb(117U, 183U, 245U);
    palette->info_surface = wl_lui_rgb(30U, 51U, 77U);
    palette->shadow = wl_lui_rgba(0U, 0U, 0U, 112U);
}

static void wl_lui_palette_high_contrast(wl_lui_palette *palette) {
    palette->window = wl_lui_rgb(0U, 0U, 0U);
    palette->surface = wl_lui_rgb(0U, 0U, 0U);
    palette->surface_alt = wl_lui_rgb(18U, 18U, 18U);
    palette->surface_elevated = wl_lui_rgb(0U, 0U, 0U);
    palette->text_primary = wl_lui_rgb(255U, 255U, 255U);
    palette->text_secondary = wl_lui_rgb(255U, 255U, 255U);
    palette->text_disabled = wl_lui_rgb(190U, 190U, 190U);
    palette->border = wl_lui_rgb(255U, 255U, 255U);
    palette->border_strong = wl_lui_rgb(255U, 255U, 0U);
    palette->accent = wl_lui_rgb(255U, 255U, 0U);
    palette->accent_hover = wl_lui_rgb(255U, 255U, 0U);
    palette->accent_pressed = wl_lui_rgb(255U, 255U, 0U);
    palette->accent_text = wl_lui_rgb(0U, 0U, 0U);
    palette->selection = wl_lui_rgb(255U, 255U, 0U);
    palette->selection_text = wl_lui_rgb(0U, 0U, 0U);
    palette->focus = wl_lui_rgb(255U, 255U, 0U);
    palette->success = wl_lui_rgb(0U, 255U, 0U);
    palette->success_surface = wl_lui_rgb(0U, 0U, 0U);
    palette->warning = wl_lui_rgb(255U, 255U, 0U);
    palette->warning_surface = wl_lui_rgb(0U, 0U, 0U);
    palette->danger = wl_lui_rgb(255U, 128U, 128U);
    palette->danger_surface = wl_lui_rgb(0U, 0U, 0U);
    palette->info = wl_lui_rgb(0U, 255U, 255U);
    palette->info_surface = wl_lui_rgb(0U, 0U, 0U);
    palette->shadow = wl_lui_rgb(0U, 0U, 0U);
}

wl_lui_result wl_lui_theme_init(wl_lui_theme *theme, wl_lui_theme_scheme scheme) {
    if (theme == NULL
        || scheme < WL_LUI_THEME_SYSTEM
        || scheme > WL_LUI_THEME_HIGH_CONTRAST) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    memset(theme, 0, sizeof(*theme));
    theme->version = WL_LUI_THEME_VERSION;
    theme->struct_size = (unsigned int)sizeof(*theme);
    theme->scheme = scheme;
    wl_lui_palette_light(&theme->light);
    wl_lui_palette_dark(&theme->dark);
    wl_lui_palette_high_contrast(&theme->high_contrast);
    strcpy(theme->font_family, "sans-serif");
    theme->font_size = 14U;
    theme->font_weight = 400U;
    theme->spacing_unit = 4U;
    theme->corner_radius = 8U;
    theme->control_height = 34U;
    theme->focus_ring_width = 2U;
    theme->motion_fast_ms = 90U;
    theme->motion_normal_ms = 160U;
    theme->motion_slow_ms = 240U;
    theme->reduced_motion = 0U;
    return WL_LUI_OK;
}

wl_lui_result wl_lui_style_init(wl_lui_style *style) {
    if (style == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    memset(style, 0, sizeof(*style));
    style->version = WL_LUI_STYLE_VERSION;
    style->struct_size = (unsigned int)sizeof(*style);
    strcpy(style->font_family, "sans-serif");
    style->font_size = 14U;
    style->font_weight = 400U;
    style->opacity_milli = 1000U;
    return WL_LUI_OK;
}

wl_lui_result wl_lui_window_create(
    const wl_lui_window_desc *description,
    wl_lui_window **window_out
) {
    wl_lui_backend backend;
    backend = wl_lui_default_backend;
    backend.style_capabilities = wl_lui_style_capabilities();
    return wl_lui_window_create_with_backend(description, &backend, window_out);
}

wl_lui_result wl_lui_window_create_with_backend(
    const wl_lui_window_desc *description,
    const wl_lui_backend *backend,
    wl_lui_window **window_out
) {
    wl_lui_window *window;
    wl_lui_result result;

    if (window_out == NULL) return WL_LUI_INVALID_ARGUMENT;
    *window_out = NULL;
    if (description == NULL || description->title == NULL
        || description->title[0] == '\0'
        || description->width < 320U || description->height < 240U
        || description->width > 16384U || description->height > 16384U
        || backend == NULL
        || backend->version != WL_LUI_BACKEND_VERSION
        || backend->struct_size < (unsigned int)sizeof(wl_lui_backend)
        || backend->create == NULL || backend->show == NULL
        || backend->run == NULL || backend->destroy == NULL
        || (backend->style_capabilities & ~(WL_LUI_STYLE_BACKGROUND
            | WL_LUI_STYLE_FOREGROUND | WL_LUI_STYLE_BORDER
            | WL_LUI_STYLE_PADDING | WL_LUI_STYLE_FONT
            | WL_LUI_STYLE_OPACITY | WL_LUI_STYLE_CORNER_RADIUS)) != 0UL) {
        return WL_LUI_INVALID_ARGUMENT;
    }

    window = (wl_lui_window *)malloc(sizeof(wl_lui_window));
    if (window == NULL) return WL_LUI_OUT_OF_MEMORY;
    window->native_handle = NULL;
    window->backend = *backend;
    wl_lui_theme_init(&window->theme, WL_LUI_THEME_SYSTEM);
    wl_lui_style_init(&window->style);

    result = window->backend.create(
        window->backend.user_data, description, &window->native_handle
    );
    if (result != WL_LUI_OK || window->native_handle == NULL) {
        free(window);
        return result == WL_LUI_OK ? WL_LUI_PLATFORM_ERROR : result;
    }

    if (window->backend.set_theme != NULL) {
        result = window->backend.set_theme(
            window->backend.user_data, window->native_handle, &window->theme
        );
        if (result != WL_LUI_OK) {
            window->backend.destroy(window->backend.user_data, window->native_handle);
            free(window);
            return result;
        }
    }
    if (window->backend.set_style != NULL) {
        result = window->backend.set_style(
            window->backend.user_data, window->native_handle, &window->style
        );
        if (result != WL_LUI_OK) {
            window->backend.destroy(window->backend.user_data, window->native_handle);
            free(window);
            return result;
        }
    }

    *window_out = window;
    return WL_LUI_OK;
}

unsigned long wl_lui_window_style_capabilities(const wl_lui_window *window) {
    if (window == NULL) return 0UL;
    return window->backend.style_capabilities;
}

wl_lui_result wl_lui_window_set_theme(
    wl_lui_window *window,
    const wl_lui_theme *theme
) {
    wl_lui_result result;
    if (window == NULL || window->native_handle == NULL || theme == NULL
        || theme->version != WL_LUI_THEME_VERSION
        || theme->struct_size < (unsigned int)sizeof(*theme)
        || theme->scheme < WL_LUI_THEME_SYSTEM
        || theme->scheme > WL_LUI_THEME_HIGH_CONTRAST
        || theme->font_size == 0U || theme->font_size > 256U
        || theme->font_weight == 0U || theme->font_weight > 1000U
        || theme->spacing_unit == 0U || theme->spacing_unit > 128U
        || theme->corner_radius > 512U
        || theme->control_height == 0U || theme->control_height > 1024U
        || theme->focus_ring_width > 64U
        || theme->motion_fast_ms > 60000U
        || theme->motion_normal_ms > 60000U
        || theme->motion_slow_ms > 60000U
        || theme->reduced_motion > 1U
        || theme->font_family[0] == '\0'
        || memchr(theme->font_family, '\0', WL_LUI_FONT_FAMILY_CAPACITY) == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if (window->backend.set_theme == NULL) return WL_LUI_UNSUPPORTED;
    result = window->backend.set_theme(window->backend.user_data, window->native_handle, theme);
    if (result == WL_LUI_OK) {
        window->theme = *theme;
    }
    return result;
}

wl_lui_result wl_lui_window_set_style(
    wl_lui_window *window,
    const wl_lui_style *style
) {
    wl_lui_result result;
    if (window == NULL || window->native_handle == NULL || style == NULL
        || style->version != WL_LUI_STYLE_VERSION
        || style->struct_size < (unsigned int)sizeof(*style)
        || (style->flags & ~(WL_LUI_STYLE_BACKGROUND
            | WL_LUI_STYLE_FOREGROUND | WL_LUI_STYLE_BORDER
            | WL_LUI_STYLE_PADDING | WL_LUI_STYLE_FONT
            | WL_LUI_STYLE_OPACITY | WL_LUI_STYLE_CORNER_RADIUS)) != 0UL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & WL_LUI_STYLE_OPACITY) != 0UL
        && style->opacity_milli > 1000U) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & WL_LUI_STYLE_FONT) != 0UL
        && (style->font_size == 0U || style->font_size > 256U
            || style->font_weight == 0U || style->font_weight > 1000U
            || style->font_family[0] == '\0'
            || memchr(style->font_family, '\0', WL_LUI_FONT_FAMILY_CAPACITY) == NULL)) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & WL_LUI_STYLE_BORDER) != 0UL
        && style->border_width > 512U) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & WL_LUI_STYLE_PADDING) != 0UL
        && (style->padding_top > 16384U || style->padding_right > 16384U
            || style->padding_bottom > 16384U || style->padding_left > 16384U)) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & WL_LUI_STYLE_CORNER_RADIUS) != 0UL
        && style->corner_radius > 8192U) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    if ((style->flags & ~window->backend.style_capabilities) != 0UL
        || (style->flags != 0UL && window->backend.set_style == NULL)) {
        return WL_LUI_UNSUPPORTED;
    }
    if (window->backend.set_style == NULL) return WL_LUI_OK;
    result = window->backend.set_style(window->backend.user_data, window->native_handle, style);
    if (result == WL_LUI_OK) {
        window->style = *style;
    }
    return result;
}

wl_lui_result wl_lui_window_show(wl_lui_window *window) {
    if (window == NULL || window->native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    return window->backend.show(window->backend.user_data, window->native_handle);
}

wl_lui_result wl_lui_window_run(wl_lui_window *window) {
    if (window == NULL || window->native_handle == NULL) {
        return WL_LUI_INVALID_ARGUMENT;
    }
    return window->backend.run(window->backend.user_data, window->native_handle);
}

void wl_lui_window_destroy(wl_lui_window *window) {
    if (window == NULL) {
        return;
    }
    if (window->native_handle != NULL) {
        window->backend.destroy(window->backend.user_data, window->native_handle);
        window->native_handle = NULL;
    }
    free(window);
}

const char *wl_lui_result_string(wl_lui_result result) {
    switch (result) {
        case WL_LUI_OK: return "success";
        case WL_LUI_INVALID_ARGUMENT: return "invalid argument";
        case WL_LUI_OUT_OF_MEMORY: return "out of memory";
        case WL_LUI_UNSUPPORTED: return "native UI property is not supported on this platform";
        case WL_LUI_PLATFORM_ERROR: return "native platform operation failed";
        default: return "unknown wl_Lui result";
    }
}

#ifndef _WIN32
unsigned long wl_lui_style_capabilities(void) {
    return 0UL;
}

wl_lui_result wl_lui_platform_create(
    const wl_lui_window_desc *description,
    void **native_handle
) {
    (void)description;
    (void)native_handle;
    return WL_LUI_UNSUPPORTED;
}

wl_lui_result wl_lui_platform_set_theme(void *native_handle, const wl_lui_theme *theme) {
    (void)native_handle;
    (void)theme;
    return WL_LUI_UNSUPPORTED;
}

wl_lui_result wl_lui_platform_set_style(void *native_handle, const wl_lui_style *style) {
    (void)native_handle;
    (void)style;
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
