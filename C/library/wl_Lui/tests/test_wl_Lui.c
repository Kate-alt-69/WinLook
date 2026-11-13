#include "wl_Lui.h"

#include <assert.h>
#include <string.h>

typedef struct fake_backend_state {
    unsigned int create_calls;
    unsigned int theme_calls;
    unsigned int style_calls;
    unsigned int show_calls;
    unsigned int run_calls;
    unsigned int destroy_calls;
} fake_backend_state;

static wl_lui_result fake_create(
    void *user_data, const wl_lui_window_desc *description, void **native_handle
) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state == 0 || description == 0 || native_handle == 0) return WL_LUI_INVALID_ARGUMENT;
    state->create_calls++;
    *native_handle = user_data;
    return WL_LUI_OK;
}

static wl_lui_result fake_set_theme(
    void *user_data, void *native_handle, const wl_lui_theme *theme
) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state == 0 || native_handle != user_data || theme == 0) return WL_LUI_INVALID_ARGUMENT;
    state->theme_calls++;
    return WL_LUI_OK;
}

static wl_lui_result fake_set_style(
    void *user_data, void *native_handle, const wl_lui_style *style
) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state == 0 || native_handle != user_data || style == 0) return WL_LUI_INVALID_ARGUMENT;
    state->style_calls++;
    return WL_LUI_OK;
}

static wl_lui_result fake_show(void *user_data, void *native_handle) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state == 0 || native_handle != user_data) return WL_LUI_INVALID_ARGUMENT;
    state->show_calls++;
    return WL_LUI_OK;
}

static wl_lui_result fake_run(void *user_data, void *native_handle) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state == 0 || native_handle != user_data) return WL_LUI_INVALID_ARGUMENT;
    state->run_calls++;
    return WL_LUI_OK;
}

static void fake_destroy(void *user_data, void *native_handle) {
    fake_backend_state *state;
    state = (fake_backend_state *)user_data;
    if (state != 0 && native_handle == user_data) state->destroy_calls++;
}

static void test_custom_portable_backend(void) {
    fake_backend_state state;
    wl_lui_backend backend;
    wl_lui_window_desc description;
    wl_lui_window *window;
    wl_lui_style style;
    wl_lui_theme theme;

    memset(&state, 0, sizeof(state));
    memset(&backend, 0, sizeof(backend));
    backend.version = WL_LUI_BACKEND_VERSION;
    backend.struct_size = (unsigned int)sizeof(backend);
    backend.style_capabilities = WL_LUI_STYLE_OPACITY;
    backend.user_data = &state;
    backend.create = fake_create;
    backend.set_theme = fake_set_theme;
    backend.set_style = fake_set_style;
    backend.show = fake_show;
    backend.run = fake_run;
    backend.destroy = fake_destroy;

    description.title = "portable host";
    description.width = 800U;
    description.height = 600U;
    window = 0;
    assert(wl_lui_window_create_with_backend(&description, &backend, &window) == WL_LUI_OK);
    assert(window != 0);
    assert(wl_lui_window_style_capabilities(window) == WL_LUI_STYLE_OPACITY);
    assert(state.create_calls == 1U);
    assert(state.theme_calls == 1U);
    assert(state.style_calls == 1U);

    wl_lui_theme_init(&theme, WL_LUI_THEME_DARK);
    assert(wl_lui_window_set_theme(window, &theme) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_OPACITY;
    style.opacity_milli = 500U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_BORDER;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_UNSUPPORTED);
    assert(wl_lui_window_show(window) == WL_LUI_OK);
    assert(wl_lui_window_run(window) == WL_LUI_OK);
    wl_lui_window_destroy(window);
    assert(state.theme_calls == 2U);
    assert(state.style_calls == 2U);
    assert(state.show_calls == 1U);
    assert(state.run_calls == 1U);
    assert(state.destroy_calls == 1U);
}

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
    assert(strcmp(theme.font_family, "sans-serif") == 0);
    assert(theme.light.accent.red == 0U);
    assert(theme.dark.window.red == 24U);
    assert(theme.high_contrast.focus.red == 255U);
    assert(wl_lui_theme_init(&theme, (wl_lui_theme_scheme)99) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_style_init(0) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_style_init(&style) == WL_LUI_OK);
    assert(style.version == WL_LUI_STYLE_VERSION);
    assert(style.opacity_milli == 1000U);
    assert(wl_lui_window_create(0, &window) == WL_LUI_INVALID_ARGUMENT);
    assert(wl_lui_window_create(0, 0) == WL_LUI_INVALID_ARGUMENT);

    test_custom_portable_backend();

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
    theme.reduced_motion = 2U;
    assert(wl_lui_window_set_theme(window, &theme) == WL_LUI_INVALID_ARGUMENT);
    wl_lui_theme_init(&theme, WL_LUI_THEME_DARK);
    assert(wl_lui_window_set_theme(window, &theme) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_FOREGROUND;
    style.foreground.red = 220U;
    style.foreground.green = 224U;
    style.foreground.blue = 232U;
    style.foreground.alpha = 255U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_FONT;
    strcpy(style.font_family, "Segoe UI");
    style.font_size = 16U;
    style.font_weight = 600U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_FONT;
    style.font_family[0] = '\0';
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_INVALID_ARGUMENT);

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
    style.flags = WL_LUI_STYLE_OPACITY;
    style.opacity_milli = 1001U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_INVALID_ARGUMENT);
    wl_lui_style_init(&style);
    style.flags = WL_LUI_STYLE_FONT;
    style.font_size = 0U;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_INVALID_ARGUMENT);
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
    style.flags = 0UL;
    assert(wl_lui_window_set_style(window, &style) == WL_LUI_OK);
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
