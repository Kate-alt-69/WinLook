#ifndef WL_LANIMATE_H
#define WL_LANIMATE_H

/*
 * Portable animation primitives for WinLook.
 * Keep the public header and implementation compatible with C89.
 * OS drawing, scheduling and window APIs belong in separate adapters.
 */

#define WL_LANIMATE_VERSION_MAJOR 0
#define WL_LANIMATE_VERSION_MINOR 1
#define WL_LANIMATE_VERSION_PATCH 0
#define WL_LANIMATE_VERSION_PRERELEASE "alpha.2"

typedef enum wl_Lanimate_easing {
    WL_LANIMATE_LINEAR = 0,
    WL_LANIMATE_EASE_IN_QUAD = 1,
    WL_LANIMATE_EASE_OUT_QUAD = 2,
    WL_LANIMATE_EASE_IN_OUT_QUAD = 3,
    WL_LANIMATE_EASE_IN_CUBIC = 4,
    WL_LANIMATE_EASE_OUT_CUBIC = 5,
    WL_LANIMATE_EASE_IN_OUT_CUBIC = 6,
    WL_LANIMATE_EASE_IN_SINE = 7,
    WL_LANIMATE_EASE_OUT_SINE = 8,
    WL_LANIMATE_EASE_IN_OUT_SINE = 9,
    WL_LANIMATE_EASE_IN_QUART = 10,
    WL_LANIMATE_EASE_OUT_QUART = 11,
    WL_LANIMATE_EASE_IN_OUT_QUART = 12
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
    double delay_ms;
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
    WL_LANIMATE_INVALID_KEYFRAME_ORDER = 2,
    WL_LANIMATE_INVALID_KEYFRAME_TIME = 3
} wl_Lanimate_result;

/* Returns eased progress. Input is clamped to [0, 1]; NaN maps to zero. */
double wl_Lanimate_ease(wl_Lanimate_easing easing, double progress);

/* Value helpers interpolate all fields; RGBA channels are not clamped. */
wl_Lanimate_point2 wl_Lanimate_point2_lerp(wl_Lanimate_point2 from_value,
                                            wl_Lanimate_point2 to_value,
                                            double progress);
wl_Lanimate_rgba wl_Lanimate_rgba_lerp(wl_Lanimate_rgba from_value,
                                        wl_Lanimate_rgba to_value,
                                        double progress);

/* Caller owns the struct; no heap allocation is performed. elapsed_ms includes delay_ms. */
void wl_Lanimate_tween_init(wl_Lanimate_tween *tween,
                            double from_value,
                            double to_value,
                            double duration_ms,
                            wl_Lanimate_easing easing);
void wl_Lanimate_tween_init_delayed(wl_Lanimate_tween *tween,
                                    double from_value,
                                    double to_value,
                                    double duration_ms,
                                    double delay_ms,
                                    wl_Lanimate_easing easing);
double wl_Lanimate_tween_progress(const wl_Lanimate_tween *tween);
double wl_Lanimate_tween_value(const wl_Lanimate_tween *tween);
double wl_Lanimate_tween_advance(wl_Lanimate_tween *tween, double delta_ms);
void wl_Lanimate_tween_pause(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_resume(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_cancel(wl_Lanimate_tween *tween);
void wl_Lanimate_tween_restart(wl_Lanimate_tween *tween);

/*
 * A track references caller-owned, strictly increasing, non-negative keyframes.
 * The caller must keep the array alive while the track is used. Sampling is
 * allocation-free. A failed initialization clears the output track when possible.
 */
wl_Lanimate_result wl_Lanimate_track_init(wl_Lanimate_track *track,
                                           const wl_Lanimate_keyframe *keyframes,
                                           unsigned int count);
double wl_Lanimate_track_sample(const wl_Lanimate_track *track, double at_ms);

#endif /* WL_LANIMATE_H */
