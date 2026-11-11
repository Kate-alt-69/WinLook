#include "wl_Lgpuv.h"

static int wl_Lgpuv_valid_class(wl_Lgpuv_work_class work_class)
{
    return (int)work_class >= (int)WL_LGPUV_CRITICAL && (int)work_class <= (int)WL_LGPUV_IDLE;
}

static int wl_Lgpuv_find_resource(const wl_Lgpuv_governor *governor, unsigned long ticket)
{
    unsigned int i;
    for (i = 0; i < governor->resource_capacity; ++i)
        if (governor->resources[i].active && governor->resources[i].id == ticket) return (int)i;
    return -1;
}

static int wl_Lgpuv_find_submission(const wl_Lgpuv_governor *governor, unsigned long ticket)
{
    unsigned int i;
    for (i = 0; i < governor->submission_capacity; ++i)
        if (governor->submissions[i].active && governor->submissions[i].id == ticket) return (int)i;
    return -1;
}

static wl_Lgpuv_result wl_Lgpuv_next_id(wl_Lgpuv_governor *governor, unsigned long *id)
{
    unsigned long candidate;
    candidate = governor->next_ticket;
    if (candidate == 0 || wl_Lgpuv_find_resource(governor, candidate) >= 0 ||
        wl_Lgpuv_find_submission(governor, candidate) >= 0) return WL_LGPUV_TICKET_EXHAUSTED;
    governor->next_ticket = candidate + 1;
    if (governor->next_ticket == 0) governor->next_ticket = 1;
    *id = candidate;
    return WL_LGPUV_OK;
}

wl_Lgpuv_result wl_Lgpuv_init(wl_Lgpuv_governor *governor,
    wl_Lgpuv_resource_slot *resources, unsigned int resource_capacity,
    wl_Lgpuv_submission_slot *submissions, unsigned int submission_capacity,
    unsigned long memory_budget_bytes, unsigned long critical_reserve_bytes,
    unsigned int max_in_flight_submissions)
{
    unsigned int i;
    if (governor == 0 || resources == 0 || submissions == 0 ||
        resource_capacity == 0 || submission_capacity == 0) return WL_LGPUV_INVALID_ARGUMENT;
    if (memory_budget_bytes == 0 || critical_reserve_bytes > memory_budget_bytes ||
        max_in_flight_submissions == 0 || max_in_flight_submissions > submission_capacity)
        return WL_LGPUV_INVALID_POLICY;
    governor->memory_budget_bytes = memory_budget_bytes;
    governor->critical_reserve_bytes = critical_reserve_bytes;
    governor->allocated_bytes = 0;
    governor->next_ticket = 1;
    governor->max_in_flight_submissions = max_in_flight_submissions;
    governor->active_submissions = 0;
    governor->resources = resources;
    governor->resource_capacity = resource_capacity;
    governor->submissions = submissions;
    governor->submission_capacity = submission_capacity;
    for (i = 0; i < resource_capacity; ++i) {
        resources[i].id = 0; resources[i].bytes = 0; resources[i].active = 0;
    }
    for (i = 0; i < submission_capacity; ++i) {
        submissions[i].id = 0; submissions[i].active = 0;
    }
    return WL_LGPUV_OK;
}

unsigned long wl_Lgpuv_available_for(const wl_Lgpuv_governor *governor,
    wl_Lgpuv_work_class work_class)
{
    unsigned long limit;
    if (governor == 0 || !wl_Lgpuv_valid_class(work_class)) return 0;
    limit = work_class == WL_LGPUV_CRITICAL
        ? governor->memory_budget_bytes
        : governor->memory_budget_bytes - governor->critical_reserve_bytes;
    return governor->allocated_bytes >= limit ? 0 : limit - governor->allocated_bytes;
}

wl_Lgpuv_result wl_Lgpuv_reserve(wl_Lgpuv_governor *governor,
    wl_Lgpuv_work_class work_class, unsigned long estimated_bytes, unsigned long *ticket)
{
    unsigned int i;
    unsigned long id;
    wl_Lgpuv_result result;
    if (governor == 0 || ticket == 0 || !wl_Lgpuv_valid_class(work_class)) return WL_LGPUV_INVALID_ARGUMENT;
    if (estimated_bytes == 0) return WL_LGPUV_INVALID_ESTIMATE;
    if (estimated_bytes > wl_Lgpuv_available_for(governor, work_class)) return WL_LGPUV_BUDGET_EXCEEDED;
    for (i = 0; i < governor->resource_capacity; ++i) if (!governor->resources[i].active) break;
    if (i == governor->resource_capacity) return WL_LGPUV_QUEUE_FULL;
    result = wl_Lgpuv_next_id(governor, &id);
    if (result != WL_LGPUV_OK) return result;
    governor->resources[i].id = id;
    governor->resources[i].bytes = estimated_bytes;
    governor->resources[i].active = 1;
    governor->allocated_bytes += estimated_bytes;
    *ticket = id;
    return WL_LGPUV_OK;
}

wl_Lgpuv_result wl_Lgpuv_release(wl_Lgpuv_governor *governor, unsigned long ticket)
{
    int index;
    wl_Lgpuv_resource_slot *slot;
    if (governor == 0 || ticket == 0) return WL_LGPUV_INVALID_ARGUMENT;
    index = wl_Lgpuv_find_resource(governor, ticket);
    if (index < 0) return WL_LGPUV_UNKNOWN_TICKET;
    slot = &governor->resources[index];
    governor->allocated_bytes -= slot->bytes;
    slot->id = 0; slot->bytes = 0; slot->active = 0;
    return WL_LGPUV_OK;
}

wl_Lgpuv_result wl_Lgpuv_begin_submission(wl_Lgpuv_governor *governor, unsigned long *ticket)
{
    unsigned int i;
    unsigned long id;
    wl_Lgpuv_result result;
    if (governor == 0 || ticket == 0) return WL_LGPUV_INVALID_ARGUMENT;
    if (governor->active_submissions >= governor->max_in_flight_submissions) return WL_LGPUV_QUEUE_FULL;
    for (i = 0; i < governor->submission_capacity; ++i) if (!governor->submissions[i].active) break;
    if (i == governor->submission_capacity) return WL_LGPUV_QUEUE_FULL;
    result = wl_Lgpuv_next_id(governor, &id);
    if (result != WL_LGPUV_OK) return result;
    governor->submissions[i].id = id;
    governor->submissions[i].active = 1;
    ++governor->active_submissions;
    *ticket = id;
    return WL_LGPUV_OK;
}

wl_Lgpuv_result wl_Lgpuv_complete_submission(wl_Lgpuv_governor *governor, unsigned long ticket)
{
    int index;
    if (governor == 0 || ticket == 0) return WL_LGPUV_INVALID_ARGUMENT;
    index = wl_Lgpuv_find_submission(governor, ticket);
    if (index < 0) return WL_LGPUV_UNKNOWN_TICKET;
    governor->submissions[index].id = 0;
    governor->submissions[index].active = 0;
    --governor->active_submissions;
    return WL_LGPUV_OK;
}

wl_Lgpuv_result wl_Lgpuv_update_budget(wl_Lgpuv_governor *governor,
    unsigned long memory_budget_bytes, unsigned long critical_reserve_bytes)
{
    if (governor == 0) return WL_LGPUV_INVALID_ARGUMENT;
    if (memory_budget_bytes == 0 || critical_reserve_bytes > memory_budget_bytes)
        return WL_LGPUV_INVALID_POLICY;
    governor->memory_budget_bytes = memory_budget_bytes;
    governor->critical_reserve_bytes = critical_reserve_bytes;
    return WL_LGPUV_OK;
}

wl_Lgpuv_pressure wl_Lgpuv_get_pressure(const wl_Lgpuv_governor *governor)
{
    if (governor == 0 || governor->allocated_bytes >= governor->memory_budget_bytes)
        return WL_LGPUV_PRESSURE_CRITICAL;
    if (governor->allocated_bytes >= governor->memory_budget_bytes - governor->memory_budget_bytes / 5)
        return WL_LGPUV_PRESSURE_CONSTRAINED;
    return WL_LGPUV_PRESSURE_NORMAL;
}
