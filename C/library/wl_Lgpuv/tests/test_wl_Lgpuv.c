#include <assert.h>
#include "wl_Lgpuv.h"

int main(void)
{
    wl_Lgpuv_governor governor;
    wl_Lgpuv_resource_slot resources[4];
    wl_Lgpuv_submission_slot submissions[2];
    unsigned long a;
    unsigned long b;

    assert(wl_Lgpuv_init(&governor, resources, 4, submissions, 2, 1000, 200, 2) == WL_LGPUV_OK);
    assert(wl_Lgpuv_available_for(&governor, WL_LGPUV_BACKGROUND) == 800);
    assert(wl_Lgpuv_reserve(&governor, WL_LGPUV_BACKGROUND, 800, &a) == WL_LGPUV_OK);
    assert(wl_Lgpuv_reserve(&governor, WL_LGPUV_INTERACTIVE, 1, &b) == WL_LGPUV_BUDGET_EXCEEDED);
    assert(wl_Lgpuv_reserve(&governor, WL_LGPUV_CRITICAL, 200, &b) == WL_LGPUV_OK);
    assert(wl_Lgpuv_get_pressure(&governor) == WL_LGPUV_PRESSURE_CRITICAL);
    assert(wl_Lgpuv_release(&governor, a) == WL_LGPUV_OK);
    assert(wl_Lgpuv_release(&governor, b) == WL_LGPUV_OK);
    assert(wl_Lgpuv_release(&governor, b) == WL_LGPUV_UNKNOWN_TICKET);

    assert(wl_Lgpuv_begin_submission(&governor, &a) == WL_LGPUV_OK);
    assert(wl_Lgpuv_begin_submission(&governor, &b) == WL_LGPUV_OK);
    assert(wl_Lgpuv_begin_submission(&governor, &b) == WL_LGPUV_QUEUE_FULL);
    assert(wl_Lgpuv_complete_submission(&governor, a) == WL_LGPUV_OK);
    assert(wl_Lgpuv_complete_submission(&governor, a) == WL_LGPUV_UNKNOWN_TICKET);
    assert(wl_Lgpuv_complete_submission(&governor, b) == WL_LGPUV_OK);

    assert(wl_Lgpuv_reserve(&governor, WL_LGPUV_INTERACTIVE, 600, &a) == WL_LGPUV_OK);
    assert(wl_Lgpuv_update_budget(&governor, 500, 100) == WL_LGPUV_OK);
    assert(wl_Lgpuv_get_pressure(&governor) == WL_LGPUV_PRESSURE_CRITICAL);
    assert(wl_Lgpuv_reserve(&governor, WL_LGPUV_CRITICAL, 1, &b) == WL_LGPUV_BUDGET_EXCEEDED);
    assert(wl_Lgpuv_release(&governor, a) == WL_LGPUV_OK);
    return 0;
}
