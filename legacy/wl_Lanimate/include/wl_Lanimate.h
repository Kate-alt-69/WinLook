#ifndef WL_LANIMATE_H
#define WL_LANIMATE_H

/*
 * Portable, allocation-free animation primitives for legacy WinLook builds.
 * Keep this header compatible with C89-era compilers. OS drawing/window APIs
 * belong in separate platform adapters.
 */

typedef enum wl_Lanimate_easing {
    WL_LANIMATE_LINEAR = 0,
    WL_LANIMATE_EASE_IN_QUAD = 1,
    WL_LANIMATE_EASE_OUT_QUAD = 2,
    WL_LANIMATE_EASE_IN_OUT_QUAD = 3,
    WL_LANIMATE_EASE_IN_CUBIC = 4,
    WL_LANIMATE_EASE_OUT_CUBIC = 5,
    WL_LANIMATE_EASE_IN_OUT_CUBIC = 6
} wl_Lanimate_easing;

typedef enum wl_Lanimate_state {
    WL_LANIMATE_PENDING = 0,
    WL_LANIMATE_RUNNING = 1,
    WL_LANIMATE_PAUSED = 2,
    WL_LANIMATE_FINISHED = 3,
    WL_LANIMATE_CANCELLED = 4
} wl_Lanimate_state;

typedef struct wl_Lanimate_tween {
    double from_value;
    double to_value;
    double duration_ms;
    double elapsed_ms;
    wl_Lanimate_easing easing;
    wl_Lanimate_state state;
} wl_Lanimate_tween;

typedef struct wl_Lanimate_point2 {
    double x;
    double y;
} wl_Lanimate_point2;

typedef struct wl_Lanimate_rgba {
    double r;
    double g;
    double b;
    double a;
} wl_Lanimate_rgba;

typedef struct wl_Lanimate_keyframe {
    double at_ms;
    double value;
    wl_Lanimate_easing easing_to_next;
} wl_Lanimate_keyframe;

typedef struct wl_Lanimate_track {
    const wl_Lanimate_keyframe *keyframes;
    unsigned int count;
} wl_Lanimate_track;

typedef enum wl_Lanimate_result {
    WL_LANIMATE_OK = 0,
    WL_LANIMATE_INVALID_ARGUMENT = 1,
    WL_LANIMATE_INVALID_KEYFRAME_ORDER = 2
} wl_Lanimate_result;

/* Returns eased progress. Input is clamped to [0, 1]. */
double wl_Lanimate_ease(wl_Lanimate_easing easing, double progress);

/* Value helpers do not allocate and do not clamp their output channels. */
wl_Lanimate_point2 wl_Lanimate_point2_lerp(wl_Lanimate_point2 from_value,
                                            wl_Lanimate_point2 to_value,
                                            double progress);
wl_Lanimate_rgba wl_Lanimate_rgba_lerp(wl_Lanimate_rgba from_value,
                                        wl_Lanimate_rgba to_value,
                                        double progress);

/* Caller owns the struct; no heap allocation is performed. */
void wl_Lanimate_tween_init(wl_Lanimate_tween *tween,
                            double from_value,
                            double to_value,
                            double duration_ms,
                            wl_Lanimate_easing easing);

double wl_Lanimate_tween_value(const wl_Lanimate_tween *tween);
double wl_Lanimate_tween_advance(wl_Lanimate_tween *tween, double delta_ms);
void wl_Lanimate_tween_pause(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_resume(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_cancel(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_restart(wl_Lanimate_tween *tween);

/*
 * A track references caller-owned, strictly increasing keyframes. The caller
 * must keep the array alive for the lifetime of the track. Sampling allocates
 * no memory and is safe for a host-controlled frame loop.
 */
wl_Lanimate_result wl_Lanimate_track_init(wl_Lanimate_track *track,
                                           const wl_Lanimate_keyframe *keyframes,
                                           unsigned int count);
double wl_Lanimate_track_sample(const wl_Lanimate_track *track, double at_ms);

#endif /* WL_LANIMATE_H */
