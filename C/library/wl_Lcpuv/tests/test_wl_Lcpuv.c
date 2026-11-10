#include <assert.h>
#include "wl_Lcpuv.h"

int main(void)
{
    wl_Lcpuv_governor governor;
    wl_Lcpuv_slot slots[4];
    unsigned long a;
    unsigned long b;
    int overrun;

    assert(wl_Lcpuv_init(&governor, slots, 4, 100, 20, 2) == WL_LCPUV_OK);
    assert(wl_Lcpuv_available_for(&governor, WL_LCPUV_BACKGROUND) == 80);
    assert(wl_Lcpuv_try_acquire(&governor, WL_LCPUV_BACKGROUND, 80, &a) == WL_LCPUV_OK);
    assert(wl_Lcpuv_try_acquire(&governor, WL_LCPUV_INTERACTIVE, 1, &b) == WL_LCPUV_BUDGET_EXCEEDED);
    assert(wl_Lcpuv_try_acquire(&governor, WL_LCPUV_CRITICAL, 20, &b) == WL_LCPUV_OK);
    assert(wl_Lcpuv_try_acquire(&governor, WL_LCPUV_IDLE, 1, &a) == WL_LCPUV_CONCURRENCY_LIMIT);
    assert(wl_Lcpuv_complete(&governor, a, 90, &overrun) == WL_LCPUV_OK);
    assert(overrun == 1);
    assert(wl_Lcpuv_cancel(&governor, b) == WL_LCPUV_OK);
    assert(wl_Lcpuv_cancel(&governor, b) == WL_LCPUV_UNKNOWN_TICKET);
    assert(governor.reserved == 0);
    wl_Lcpuv_reset_window(&governor);
    assert(governor.spent == 0);
    assert(wl_Lcpuv_get_pressure(&governor) == WL_LCPUV_PRESSURE_NORMAL);
    assert(wl_Lcpuv_init(&governor, slots, 4, 0, 0, 1) == WL_LCPUV_INVALID_POLICY);
    assert(wl_Lcpuv_init(&governor, slots, 4, 10, 11, 1) == WL_LCPUV_INVALID_POLICY);
    return 0;
}
