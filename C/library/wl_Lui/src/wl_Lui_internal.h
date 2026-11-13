#ifndef WL_LUI_INTERNAL_H
#define WL_LUI_INTERNAL_H

#include "wl_Lui.h"

wl_lui_result wl_lui_platform_create(
    const wl_lui_window_desc *description,
    void **native_handle
);
wl_lui_result wl_lui_platform_show(void *native_handle);
wl_lui_result wl_lui_platform_run(void *native_handle);
void wl_lui_platform_destroy(void *native_handle);

#endif
