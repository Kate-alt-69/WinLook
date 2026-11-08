#include "wl_Lanimate.h"

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

void wl_Lanimate_tween_init(wl_Lanimate_tween *tween,
                            double from_value,
                            double to_value,
                            double duration_ms,
                            wl_Lanimate_easing easing)
{
    if (tween == 0) return;
    tween->from_value = from_value;
    tween->to_value = to_value;
    tween->duration_ms = duration_ms > 0.0 ? duration_ms : 0.0;
    tween->elapsed_ms = 0.0;
    tween->easing = easing;
    tween->state = tween->duration_ms == 0.0 ? WL_LANIMATE_FINISHED : WL_LANIMATE_PENDING;
}

double wl_Lanimate_tween_value(const wl_Lanimate_tween *tween)
{
    double progress;
    if (tween == 0) return 0.0;
    if (tween->duration_ms <= 0.0 || tween->state == WL_LANIMATE_FINISHED) return tween->to_value;
    progress = tween->elapsed_ms / tween->duration_ms;
    return wl_Lanimate_lerp(tween->from_value, tween->to_value,
                            wl_Lanimate_ease(tween->easing, progress));
}

double wl_Lanimate_tween_advance(wl_Lanimate_tween *tween, double delta_ms)
{
    double remaining;
    if (tween == 0) return 0.0;
    if (tween->state == WL_LANIMATE_PENDING) tween->state = WL_LANIMATE_RUNNING;
    if (tween->state == WL_LANIMATE_RUNNING && delta_ms == delta_ms && delta_ms > 0.0) {
        remaining = tween->duration_ms - tween->elapsed_ms;
        if (delta_ms >= remaining) {
            tween->elapsed_ms = tween->duration_ms;
            tween->state = WL_LANIMATE_FINISHED;
        } else {
            tween->elapsed_ms += delta_ms;
        }
    }
    return wl_Lanimate_tween_value(tween);
}

void wl_Lanimate_tween_pause(wl_Lanimate_tween *tween)
{
    if (tween != 0 && tween->state == WL_LANIMATE_RUNNING) tween->state = WL_LANIMATE_PAUSED;
}

void wl_Lanimate_tween_resume(wl_Lanimate_tween *tween)
{
    if (tween != 0 && tween->state == WL_LANIMATE_PAUSED) tween->state = WL_LANIMATE_RUNNING;
}

void wl_Lanimate_tween_cancel(wl_Lanimate_tween *tween)
{
    if (tween != 0 && tween->state != WL_LANIMATE_FINISHED) tween->state = WL_LANIMATE_CANCELLED;
}

void wl_Lanimate_tween_restart(wl_Lanimate_tween *tween)
{
    if (tween == 0) return;
    tween->elapsed_ms = 0.0;
    tween->state = tween->duration_ms == 0.0 ? WL_LANIMATE_FINISHED : WL_LANIMATE_PENDING;
}
