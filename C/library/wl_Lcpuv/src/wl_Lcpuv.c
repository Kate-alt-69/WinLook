#include "wl_Lcpuv.h"

static unsigned long wl_Lcpuv_add_saturating(unsigned long a, unsigned long b)
{
    if (b > ((unsigned long)-1) - a) return (unsigned long)-1;
    return a + b;
}

static int wl_Lcpuv_valid_class(wl_Lcpuv_work_class work_class)
{
    return (int)work_class >= (int)WL_LCPUV_CRITICAL && (int)work_class <= (int)WL_LCPUV_IDLE;
}

static int wl_Lcpuv_find_slot(const wl_Lcpuv_governor *governor, unsigned long ticket)
{
    unsigned int i;
    for (i = 0; i < governor->slot_capacity; ++i) {
        if (governor->slots[i].active && governor->slots[i].id == ticket) return (int)i;
    }
    return -1;
}

wl_Lcpuv_result wl_Lcpuv_init(wl_Lcpuv_governor *governor,
    wl_Lcpuv_slot *slots, unsigned int slot_capacity,
    unsigned long window_budget, unsigned long critical_reserve,
    unsigned int max_in_flight)
{
    unsigned int i;
    if (governor == 0 || slots == 0 || slot_capacity == 0) return WL_LCPUV_INVALID_ARGUMENT;
    if (window_budget == 0 || critical_reserve > window_budget ||
        max_in_flight == 0 || max_in_flight > slot_capacity) return WL_LCPUV_INVALID_POLICY;
    governor->window_budget = window_budget;
    governor->critical_reserve = critical_reserve;
    governor->spent = 0;
    governor->reserved = 0;
    governor->next_ticket = 1;
    governor->max_in_flight = max_in_flight;
    governor->active_count = 0;
    governor->slots = slots;
    governor->slot_capacity = slot_capacity;
    for (i = 0; i < slot_capacity; ++i) {
        slots[i].id = 0;
        slots[i].estimate = 0;
        slots[i].active = 0;
    }
    return WL_LCPUV_OK;
}

unsigned long wl_Lcpuv_available_for(const wl_Lcpuv_governor *governor,
    wl_Lcpuv_work_class work_class)
{
    unsigned long limit;
    unsigned long used;
    if (governor == 0 || !wl_Lcpuv_valid_class(work_class)) return 0;
    limit = work_class == WL_LCPUV_CRITICAL
        ? governor->window_budget : governor->window_budget - governor->critical_reserve;
    used = wl_Lcpuv_add_saturating(governor->spent, governor->reserved);
    return used >= limit ? 0 : limit - used;
}

wl_Lcpuv_result wl_Lcpuv_try_acquire(wl_Lcpuv_governor *governor,
    wl_Lcpuv_work_class work_class, unsigned long estimate, unsigned long *ticket)
{
    unsigned int i;
    unsigned long id;
    if (governor == 0 || ticket == 0 || !wl_Lcpuv_valid_class(work_class)) return WL_LCPUV_INVALID_ARGUMENT;
    if (estimate == 0) return WL_LCPUV_INVALID_ESTIMATE;
    if (governor->active_count >= governor->max_in_flight) return WL_LCPUV_CONCURRENCY_LIMIT;
    if (estimate > wl_Lcpuv_available_for(governor, work_class)) return WL_LCPUV_BUDGET_EXCEEDED;
    for (i = 0; i < governor->slot_capacity; ++i) if (!governor->slots[i].active) break;
    if (i == governor->slot_capacity) return WL_LCPUV_CONCURRENCY_LIMIT;

    id = governor->next_ticket;
    if (id == 0 || wl_Lcpuv_find_slot(governor, id) >= 0) return WL_LCPUV_TICKET_EXHAUSTED;
    governor->next_ticket = id + 1;
    if (governor->next_ticket == 0) governor->next_ticket = 1;
    governor->slots[i].id = id;
    governor->slots[i].estimate = estimate;
    governor->slots[i].active = 1;
    governor->reserved = wl_Lcpuv_add_saturating(governor->reserved, estimate);
    ++governor->active_count;
    *ticket = id;
    return WL_LCPUV_OK;
}

wl_Lcpuv_result wl_Lcpuv_complete(wl_Lcpuv_governor *governor,
    unsigned long ticket, unsigned long actual, int *overrun)
{
    int index;
    wl_Lcpuv_slot *slot;
    if (governor == 0 || ticket == 0) return WL_LCPUV_INVALID_ARGUMENT;
    index = wl_Lcpuv_find_slot(governor, ticket);
    if (index < 0) return WL_LCPUV_UNKNOWN_TICKET;
    slot = &governor->slots[index];
    governor->reserved -= slot->estimate;
    governor->spent = wl_Lcpuv_add_saturating(governor->spent, actual);
    if (overrun != 0) *overrun = actual > slot->estimate;
    slot->active = 0;
    slot->id = 0;
    slot->estimate = 0;
    --governor->active_count;
    return WL_LCPUV_OK;
}

wl_Lcpuv_result wl_Lcpuv_cancel(wl_Lcpuv_governor *governor, unsigned long ticket)
{
    int index;
    wl_Lcpuv_slot *slot;
    if (governor == 0 || ticket == 0) return WL_LCPUV_INVALID_ARGUMENT;
    index = wl_Lcpuv_find_slot(governor, ticket);
    if (index < 0) return WL_LCPUV_UNKNOWN_TICKET;
    slot = &governor->slots[index];
    governor->reserved -= slot->estimate;
    slot->active = 0;
    slot->id = 0;
    slot->estimate = 0;
    --governor->active_count;
    return WL_LCPUV_OK;
}

void wl_Lcpuv_reset_window(wl_Lcpuv_governor *governor)
{
    if (governor != 0) governor->spent = 0;
}

wl_Lcpuv_pressure wl_Lcpuv_get_pressure(const wl_Lcpuv_governor *governor)
{
    unsigned long used;
    if (governor == 0) return WL_LCPUV_PRESSURE_CRITICAL;
    used = wl_Lcpuv_add_saturating(governor->spent, governor->reserved);
    if (used >= governor->window_budget) return WL_LCPUV_PRESSURE_CRITICAL;
    if (used >= governor->window_budget - governor->window_budget / 5)
        return WL_LCPUV_PRESSURE_CONSTRAINED;
    return WL_LCPUV_PRESSURE_NORMAL;
}
