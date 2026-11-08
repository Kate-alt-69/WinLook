#include "wl_Lanimate.h"

#include <float.h>

static double wl_Lanimate_clamp01(double value)
{
    if (value != value) return 0.0;
    if (value < 0.0) return 0.0;
    if (value > 1.0) return 1.0;
    return value;
}

static double wl_Lanimate_lerp(double from_value, double to_value, double progress)
{
    return from_value + (to_value - from_value) * wl_Lanimate_clamp01(progress);
}

static double wl_Lanimate_valid_duration(double value)
{
    if (!(value >= 0.0) || value > DBL_MAX) return 0.0;
    return value;
}

static double wl_Lanimate_total_time(const wl_Lanimate_tween *tween)
{
    if (tween->duration_ms > DBL_MAX - tween->delay_ms) return DBL_MAX;
    return tween->delay_ms + tween->duration_ms;
}

static void wl_Lanimate_refresh_state(wl_Lanimate_tween *tween)
{
    double active_elapsed;
    if (tween->elapsed_ms < tween->delay_ms) {
        tween->state = WL_LANIMATE_PENDING;
        return;
    }
    active_elapsed = tween->elapsed_ms - tween->delay_ms;
    if (tween->duration_ms <= 0.0 || active_elapsed >= tween->duration_ms) {
        tween->state = WL_LANIMATE_FINISHED;
    } else {
        tween->state = WL_LANIMATE_RUNNING;
    }
}

void wl_Lanimate_tween_init(wl_Lanimate_tween *tween,
                            double from_value,
                            double to_value,
                            double duration_ms,
                            wl_Lanimate_easing easing)
{
    wl_Lanimate_tween_init_delayed(tween, from_value, to_value, duration_ms,
                                   0.0, easing);
}

void wl_Lanimate_tween_init_delayed(wl_Lanimate_tween *tween,
                                    double from_value,
                                    double to_value,
                                    double duration_ms,
                                    double delay_ms,
                                    wl_Lanimate_easing easing)
{
    if (tween == 0) return;
    tween->from_value = from_value;
    tween->to_value = to_value;
    tween->duration_ms = wl_Lanimate_valid_duration(duration_ms);
    tween->delay_ms = wl_Lanimate_valid_duration(delay_ms);
    tween->elapsed_ms = 0.0;
    tween->easing = easing;
    tween->state = (tween->duration_ms == 0.0 && tween->delay_ms == 0.0)
        ? WL_LANIMATE_FINISHED : WL_LANIMATE_PENDING;
}

double wl_Lanimate_tween_progress(const wl_Lanimate_tween *tween)
{
    double active_elapsed;
    if (tween == 0) return 0.0;
    if (tween->elapsed_ms < tween->delay_ms) return 0.0;
    if (tween->duration_ms <= 0.0) return 1.0;
    active_elapsed = tween->elapsed_ms - tween->delay_ms;
    return wl_Lanimate_clamp01(active_elapsed / tween->duration_ms);
}

double wl_Lanimate_tween_value(const wl_Lanimate_tween *tween)
{
    if (tween == 0) return 0.0;
    if (tween->state == WL_LANIMATE_FINISHED) return tween->to_value;
    if (tween->elapsed_ms < tween->delay_ms) return tween->from_value;
    return wl_Lanimate_lerp(tween->from_value, tween->to_value,
                            wl_Lanimate_ease(tween->easing,
                                             wl_Lanimate_tween_progress(tween)));
}

double wl_Lanimate_tween_advance(wl_Lanimate_tween *tween, double delta_ms)
{
    double remaining;
    double total_time;
    if (tween == 0) return 0.0;
    if (tween->state == WL_LANIMATE_PENDING || tween->state == WL_LANIMATE_RUNNING) {
        if (delta_ms == delta_ms && delta_ms > 0.0 && delta_ms <= DBL_MAX) {
            total_time = wl_Lanimate_total_time(tween);
            remaining = total_time - tween->elapsed_ms;
            if (delta_ms >= remaining) tween->elapsed_ms = total_time;
            else tween->elapsed_ms += delta_ms;
        }
        wl_Lanimate_refresh_state(tween);
    }
    return wl_Lanimate_tween_value(tween);
}

void wl_Lanimate_tween_pause(wl_Lanimate_tween *tween)
{
    if (tween != 0 && (tween->state == WL_LANIMATE_PENDING ||
                       tween->state == WL_LANIMATE_RUNNING)) {
        tween->state = WL_LANIMATE_PAUSED;
    }
}

void wl_Lanimate_tween_resume(wl_Lanimate_tween *tween)
{
    if (tween != 0 && tween->state == WL_LANIMATE_PAUSED) {
        wl_Lanimate_refresh_state(tween);
    }
}

void wl_Lanimate_tween_cancel(wl_Lanimate_tween *tween)
{
    if (tween != 0 && tween->state != WL_LANIMATE_FINISHED) {
        tween->state = WL_LANIMATE_CANCELLED;
    }
}

void wl_Lanimate_tween_restart(wl_Lanimate_tween *tween)
{
    if (tween == 0) return;
    tween->elapsed_ms = 0.0;
    tween->state = (tween->duration_ms == 0.0 && tween->delay_ms == 0.0)
        ? WL_LANIMATE_FINISHED : WL_LANIMATE_PENDING;
}
