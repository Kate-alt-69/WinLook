#include "wl_Lanimate.h"

#include <float.h>
#include <limits.h>

static int wl_Lanimate_clock_valid_time(double value)
{
    return value >= 0.0 && value <= DBL_MAX;
}

wl_Lanimate_result wl_Lanimate_frame_clock_init(
    wl_Lanimate_frame_clock *clock,
    double target_interval_ms,
    double maximum_delta_ms
)
{
    if (clock == 0 || !(target_interval_ms >= 0.0)
        || target_interval_ms > DBL_MAX
        || !(maximum_delta_ms > 0.0)
        || maximum_delta_ms > DBL_MAX) {
        return WL_LANIMATE_INVALID_ARGUMENT;
    }

    clock->target_interval_ms = target_interval_ms;
    clock->maximum_delta_ms = maximum_delta_ms;
    clock->last_timestamp_ms = 0.0;
    clock->frame_index = 0UL;
    clock->has_last_timestamp = 0U;
    clock->paused = 0U;
    return WL_LANIMATE_OK;
}

wl_Lanimate_result wl_Lanimate_frame_clock_tick(
    wl_Lanimate_frame_clock *clock,
    double now_ms,
    wl_Lanimate_frame_tick *tick_out
)
{
    double elapsed_ms;

    if (clock == 0 || tick_out == 0 || !wl_Lanimate_clock_valid_time(now_ms)) {
        return WL_LANIMATE_INVALID_ARGUMENT;
    }
    tick_out->timestamp_ms = now_ms;
    tick_out->delta_ms = 0.0;
    tick_out->frame_index = clock->frame_index;

    if (clock->paused != 0U) return WL_LANIMATE_NO_FRAME;

    if (clock->has_last_timestamp == 0U) {
        clock->last_timestamp_ms = now_ms;
        clock->has_last_timestamp = 1U;
        if (clock->frame_index < ULONG_MAX) clock->frame_index++;
        return WL_LANIMATE_OK;
    }

    if (now_ms < clock->last_timestamp_ms) {
        clock->last_timestamp_ms = now_ms;
        return WL_LANIMATE_NO_FRAME;
    }

    elapsed_ms = now_ms - clock->last_timestamp_ms;
    if (elapsed_ms < clock->target_interval_ms) {
        return WL_LANIMATE_NO_FRAME;
    }

    clock->last_timestamp_ms = now_ms;
    tick_out->delta_ms = elapsed_ms > clock->maximum_delta_ms
        ? clock->maximum_delta_ms : elapsed_ms;
    tick_out->frame_index = clock->frame_index;
    if (clock->frame_index < ULONG_MAX) clock->frame_index++;
    return WL_LANIMATE_OK;
}

void wl_Lanimate_frame_clock_pause(wl_Lanimate_frame_clock *clock)
{
    if (clock != 0) clock->paused = 1U;
}

wl_Lanimate_result wl_Lanimate_frame_clock_resume(
    wl_Lanimate_frame_clock *clock,
    double now_ms
)
{
    if (clock == 0 || !wl_Lanimate_clock_valid_time(now_ms)) {
        return WL_LANIMATE_INVALID_ARGUMENT;
    }
    clock->last_timestamp_ms = now_ms;
    clock->has_last_timestamp = 1U;
    clock->paused = 0U;
    return WL_LANIMATE_OK;
}

void wl_Lanimate_frame_clock_reset(wl_Lanimate_frame_clock *clock)
{
    if (clock == 0) return;
    clock->last_timestamp_ms = 0.0;
    clock->frame_index = 0UL;
    clock->has_last_timestamp = 0U;
    clock->paused = 0U;
}
