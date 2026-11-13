#include "wl_Lui.h"

#include <assert.h>
#include <string.h>

int main(void) {
    wl_lui_window_desc description;
    wl_lui_window *window;
    wl_lui_result result;
    wl_lui_theme theme;
    wl_lui_style style;

    assert(wl_lui_result_string(WL_LUI_OK) != 0);
    assert(wl_lui_result_string(WL_LUI_UNSUPPORTED) != 0);
    assert(wl_lui_theme_init(0, WL_LUI_THEME_SYSTEM) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_theme_init(&theme, WL_LUI_THEME_SYSTEM) == WL_LUI_OK);
    assert(theme.version == WL_LUI_THEME_VERSION);
    assert(strcmp(theme.font_family, "Segoe UI") == 0);
    assert(theme.light.accent.red == 0U);
    assert(theme.dark.window.red == 24U);
    assert(theme.high_contrast.focus.red == 255U);
    assert(wl_lui_theme_init(&theme, (wl_lui_theme_scheme)99) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_style_init(0) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_style_init(&style) == WL_LUI_OK);
    assert(style.version == WL_LUI_STYLE_VERSION);
    assert(style.opacity_milli == 1000U);
    style.flags = WL_LUI_STYLE_BACKGROUND;
    style.font_family[0] = '\0';
    style.font_size = 0U;
    style.font_weight = 0U;
    style.opacity_milli = 2000U;
    assert(style.version == WL_LUI_STYLE_VERSION);
    assert(wl_lui_style_init(&style) == WL_LUI_OK);
    assert(wl_lui_window_create(0, &window) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_window_create(0, 0) == WL_LUI_INVALID_ARGUMENT);

    description.title = "wl_Lui contract test";
    description.width = 800U;
    description.height = 600U;
    window = 0;
    result = wl_lui_window_create(&description, &window);

#ifdef _WIN32
    assert(result == WL_LUI_OK);
    assert(window != 0);
    assert(wl_lui_style_capabilities()
        == (WL_LUI_STYLE_BACKGROUND | WL_LUI_STYLE_FOREGROUND | WL_LUI_STYLE_FONT));
    wl_lui_theme_init(&theme, WL_LUI_THEME_DARK);
    assert(wl_lui_window_set_theme(window, &theme) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_BACKGROUND | WL_LUI_STYLE_FOREGROUND;
    style.background.red = 20U;
    style.background.green = 22U;
    style.background.blue = 28U;
    style.background.alpha = 255U;
    style.foreground.red = 240U;
    style.foreground.green = 240U;
    style.foreground.blue = 240U;
    style.foreground.alpha = 255U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_CORNER_RADIUS;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_UNSUPPORTED);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_BORDER | WL_LUI_STYLE_PADDING;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_UNSUPPORTED);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_OPACITY;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_UNSUPPORTED);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_BACKGROUND;
    style.font_family[0] = '\0';
    style.font_size = 0U;
    style.font_weight = 0U;
    style.opacity_milli = 2000U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
    wl_lui_window_destroy(window);
#else
    assert(result == WL_LUI_UNSUPPORTED);
    assert(window == 0);
    assert(wl_lui_style_capabilities() == 0UL);
#endif

    description.width = 100U;
    window = 0;
    assert(wl_lui_window_create(&description, &window) == WL_LUI_INVALID_ARGUMENT);

    return 0;
}
