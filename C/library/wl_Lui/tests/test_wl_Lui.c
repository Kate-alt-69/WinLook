#include "wl_Lui.h"

#include <assert.h>

int main(void) {
    wl_lui_window_desc description;
    wl_lui_window *window;
    wl_lui_result result;

    assert(wl_lui_result_string(WL_LUI_OK) != 0);
    assert(wl_lui_result_string(WL_LUI_UNSUPPORTED) != 0);
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
    wl_lui_window_destroy(window);
#else
    assert(result == WL_LUI_UNSUPPORTED);
    assert(window == 0);
#endif

    description.width = 100U;
    window = 0;
    assert(wl_lui_window_create(&description, &window) == WL_LUI_INVALID_ARGUMENT);

    return 0;
}
